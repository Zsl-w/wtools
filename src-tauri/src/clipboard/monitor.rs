use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use super::history::get_clipboard_history;

#[cfg(target_os = "windows")]
use winapi::um::winuser::{CF_HDROP};

/// 监控是否正在运行
static MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);

/// 监控是否应该停止
static MONITOR_STOP: AtomicBool = AtomicBool::new(false);

/// 启动剪贴板监听
pub fn start_clipboard_monitor() {
    // 如果已经在运行，直接返回
    if MONITOR_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    // 重置停止标志
    MONITOR_STOP.store(false, Ordering::SeqCst);

    thread::spawn(|| {
        let mut last_text: Option<String> = None;
        let mut last_image_hash: Option<u64> = None;
        let mut last_files_hash: Option<u64> = None;
        let mut error_count = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 10;

        loop {
            // 检查是否需要停止
            if MONITOR_STOP.load(Ordering::SeqCst) {
                println!("[clipboard] 监听线程收到停止信号，退出");
                MONITOR_RUNNING.store(false, Ordering::SeqCst);
                break;
            }

            // 先尝试读取图片（Windows API）
            #[cfg(target_os = "windows")]
            {
                match read_clipboard_image_win32() {
                    Ok(Some((image_data, hash))) => {
                        error_count = 0;
                        if Some(hash) != last_image_hash {
                            last_image_hash = Some(hash);
                            last_text = None;
                            last_files_hash = None;

                            if let Ok(mut history) = get_clipboard_history().lock() {
                                history.add_image(image_data);
                            }
                        }
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error_count += 1;
                        if error_count <= MAX_CONSECUTIVE_ERRORS {
                            eprintln!("[clipboard] 读取图片失败 ({}): {}", error_count, e);
                        }
                    }
                }

                // 尝试读取文件列表（CF_HDROP）
                match read_clipboard_files_win32() {
                    Ok(Some((files, hash))) => {
                        error_count = 0;
                        if Some(hash) != last_files_hash {
                            last_files_hash = Some(hash);
                            last_text = None;
                            last_image_hash = None;

                            if let Ok(mut history) = get_clipboard_history().lock() {
                                history.add_files(files);
                            }
                        }
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error_count += 1;
                        if error_count <= MAX_CONSECUTIVE_ERRORS {
                            eprintln!("[clipboard] 读取文件列表失败 ({}): {}", error_count, e);
                        }
                    }
                }
            }

            // 没有图片和文件，尝试读取文本
            match read_clipboard_text_win32() {
                Ok(Some(text)) => {
                    error_count = 0;
                    if text.trim().is_empty() {
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    if Some(&text) != last_text.as_ref() {
                        last_text = Some(text.clone());
                        last_image_hash = None;
                        last_files_hash = None;

                        if let Ok(mut history) = get_clipboard_history().lock() {
                            history.add_text(text);
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    error_count += 1;
                    if error_count <= MAX_CONSECUTIVE_ERRORS {
                        eprintln!("[clipboard] 读取文本失败 ({}): {}", error_count, e);
                    }
                }
            }

            // 错误过多时延长等待时间
            if error_count > MAX_CONSECUTIVE_ERRORS {
                eprintln!("[clipboard] 连续错误过多，暂停 5 秒");
                thread::sleep(Duration::from_secs(5));
                // 重置错误计数，给系统恢复的机会
                error_count = 0;
            } else {
                thread::sleep(Duration::from_millis(500));
            }
        }
    });
}

/// 停止剪贴板监听
pub fn stop_clipboard_monitor() {
    MONITOR_STOP.store(true, Ordering::SeqCst);
}

/// 检查监听是否正在运行
pub fn is_monitor_running() -> bool {
    MONITOR_RUNNING.load(Ordering::SeqCst)
}

#[cfg(target_os = "windows")]
fn read_clipboard_text_win32() -> Result<Option<String>, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use winapi::um::winuser::{OpenClipboard, CloseClipboard, GetClipboardData, CF_UNICODETEXT};
    use winapi::um::winbase::{GlobalLock, GlobalUnlock, GlobalSize};
    use winapi::shared::ntdef::HANDLE;

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            // 剪贴板可能被其他程序占用，这不是致命错误
            return Ok(None);
        }

        // 使用 RAII 确保剪贴板关闭
        struct ClipboardGuard;
        impl Drop for ClipboardGuard {
            fn drop(&mut self) {
                unsafe { CloseClipboard(); }
            }
        }
        let _guard = ClipboardGuard;

        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle.is_null() {
            return Ok(None);
        }

        let ptr = GlobalLock(handle as HANDLE);
        if ptr.is_null() {
            return Err("GlobalLock 返回空指针".to_string());
        }

        // 使用 RAII 确保内存解锁
        struct LockGuard(HANDLE);
        impl Drop for LockGuard {
            fn drop(&mut self) {
                unsafe { GlobalUnlock(self.0); }
            }
        }
        let _lock_guard = LockGuard(handle as HANDLE);

        // 获取实际数据大小
        let actual_size = GlobalSize(handle as HANDLE) as usize;
        if actual_size == 0 || actual_size > 10 * 1024 * 1024 {
            return Err(format!("剪贴板数据大小异常: {} 字节", actual_size));
        }

        // 计算字符数（每个 UTF-16 字符 2 字节）
        let max_chars = actual_size / 2;

        // 安全地查找 null 终止符
        let mut len = 0;
        let p = ptr as *const u16;
        while len < max_chars && *p.add(len) != 0 {
            len += 1;
        }

        if len == 0 {
            return Ok(None);
        }

        let slice = std::slice::from_raw_parts(ptr as *const u16, len);
        let text = OsString::from_wide(slice).to_string_lossy().to_string();

        Ok(Some(text))
    }
}

#[cfg(target_os = "windows")]
fn read_clipboard_image_win32() -> Result<Option<(Vec<u8>, u64)>, String> {
    use winapi::um::winuser::{OpenClipboard, CloseClipboard, GetClipboardData, CF_DIB};
    use winapi::shared::ntdef::HANDLE;
    use winapi::um::winbase::{GlobalLock, GlobalUnlock, GlobalSize};
    use winapi::um::wingdi::BITMAPINFOHEADER;
    use std::mem::size_of;

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Ok(None);
        }

        // 使用 RAII 确保剪贴板关闭
        struct ClipboardGuard;
        impl Drop for ClipboardGuard {
            fn drop(&mut self) {
                unsafe { CloseClipboard(); }
            }
        }
        let _guard = ClipboardGuard;

        let handle = GetClipboardData(CF_DIB);
        if handle.is_null() {
            return Ok(None);
        }

        let ptr = GlobalLock(handle as HANDLE);
        if ptr.is_null() {
            return Err("GlobalLock 返回空指针".to_string());
        }

        // 使用 RAII 确保内存解锁
        struct LockGuard(HANDLE);
        impl Drop for LockGuard {
            fn drop(&mut self) {
                unsafe { GlobalUnlock(self.0); }
            }
        }
        let _lock_guard = LockGuard(handle as HANDLE);

        let size = GlobalSize(handle as HANDLE) as usize;

        // 安全检查：限制最大图片大小（20MB，降低以提升性能）
        const MAX_IMAGE_SIZE: usize = 20 * 1024 * 1024;
        if size > MAX_IMAGE_SIZE {
            return Err(format!("图片过大 ({} MB)，已跳过", size / 1024 / 1024));
        }

        if size < size_of::<BITMAPINFOHEADER>() {
            return Err("DIB 数据过小".to_string());
        }

        let data = std::slice::from_raw_parts(ptr as *const u8, size);

        // 解析 BITMAPINFOHEADER
        let header = &*(data.as_ptr() as *const BITMAPINFOHEADER);

        // 验证图片尺寸合理性
        let width = header.biWidth.unsigned_abs();
        let height = header.biHeight.unsigned_abs();
        if width == 0 || height == 0 || width > 16384 || height > 16384 {
            return Err(format!("图片尺寸不合理: {}x{}", width, height));
        }

        let bit_count = header.biBitCount;

        // 计算像素数据偏移
        let header_size = header.biSize as usize;
        let pixel_offset = header_size + if bit_count <= 8 { 256 * 4 } else { 0 };

        if data.len() < pixel_offset {
            return Err("DIB 数据过短".to_string());
        }

        let pixel_data = &data[pixel_offset..];
        let hash = calculate_hash(pixel_data);

        // 转换为 PNG
        let png_data = dib_to_png(data, width, height, bit_count)?;

        Ok(Some((png_data, hash)))
    }
}

#[cfg(target_os = "windows")]
fn dib_to_png(data: &[u8], width: u32, height: u32, bit_count: u16) -> Result<Vec<u8>, String> {
    use image::{ImageBuffer, Rgba, RgbaImage};
    use winapi::um::wingdi::BITMAPINFOHEADER;

    let pixel_offset = unsafe {
        let header = &*(data.as_ptr() as *const BITMAPINFOHEADER);
        let h_size = header.biSize as usize;
        h_size + if bit_count <= 8 { 256 * 4 } else { 0 }
    };

    if pixel_offset >= data.len() {
        return Err("像素数据偏移超出数据长度".to_string());
    }

    let pixel_data = &data[pixel_offset..];
    let row_size = ((bit_count as u32 * width + 31) / 32) * 4;

    // 验证预期的像素数据大小
    let expected_size = row_size as usize * height as usize;
    if pixel_data.len() < expected_size {
        return Err(format!("像素数据不足: 预期 {} 字节，实际 {} 字节", expected_size, pixel_data.len()));
    }

    let mut img: RgbaImage = ImageBuffer::new(width, height);

    match bit_count {
        32 => {
            for y in 0..height {
                for x in 0..width {
                    let src_idx = ((height - 1 - y) * row_size + x * 4) as usize;
                    // 安全检查已在上面完成，这里可以安全访问
                    let b = pixel_data[src_idx];
                    let g = pixel_data[src_idx + 1];
                    let r = pixel_data[src_idx + 2];
                    let a = pixel_data[src_idx + 3];
                    img.put_pixel(x, y, Rgba([r, g, b, a]));
                }
            }
        }
        24 => {
            for y in 0..height {
                for x in 0..width {
                    let src_idx = ((height - 1 - y) * row_size + x * 3) as usize;
                    let b = pixel_data[src_idx];
                    let g = pixel_data[src_idx + 1];
                    let r = pixel_data[src_idx + 2];
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
        }
        _ => return Err(format!("不支持的位深度: {}", bit_count)),
    }

    let mut png_data = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageOutputFormat::Png)
        .map_err(|e| format!("PNG 编码失败: {}", e))?;

    Ok(png_data)
}

fn calculate_hash(data: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

// 非 Windows 平台的占位实现
#[cfg(not(target_os = "windows"))]
fn read_clipboard_text_win32() -> Result<Option<String>, String> {
    Ok(None)
}

#[cfg(not(target_os = "windows"))]
fn read_clipboard_image_win32() -> Result<Option<(Vec<u8>, u64)>, String> {
    Ok(None)
}

#[cfg(not(target_os = "windows"))]
fn read_clipboard_files_win32() -> Result<Option<(Vec<String>, u64)>, String> {
    Ok(None)
}

#[cfg(target_os = "windows")]
fn read_clipboard_files_win32() -> Result<Option<(Vec<String>, u64)>, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use winapi::um::winuser::{OpenClipboard, CloseClipboard, GetClipboardData};
    use winapi::um::shellapi::{DragQueryFileW, DragFinish, HDROP};
    use winapi::shared::ntdef::HANDLE;
    use winapi::um::winbase::GlobalLock;

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Ok(None);
        }

        // 使用 RAII 确保剪贴板关闭
        struct ClipboardGuard;
        impl Drop for ClipboardGuard {
            fn drop(&mut self) {
                unsafe { CloseClipboard(); }
            }
        }
        let _guard = ClipboardGuard;

        let handle = GetClipboardData(CF_HDROP);
        if handle.is_null() {
            return Ok(None);
        }

        let hdrop = GlobalLock(handle as HANDLE) as HDROP;
        if hdrop.is_null() {
            return Err("GlobalLock 返回空指针".to_string());
        }

        // 使用 RAII 确保 HDROP 资源释放
        struct DropGuard(HDROP);
        impl Drop for DropGuard {
            fn drop(&mut self) {
                unsafe { DragFinish(self.0); }
            }
        }
        let _drop_guard = DropGuard(hdrop);

        let file_count = DragQueryFileW(hdrop, 0xFFFFFFFF, std::ptr::null_mut(), 0);
        if file_count == 0 {
            return Ok(None);
        }

        // 限制文件数量，防止恶意剪贴板数据
        const MAX_FILES: u32 = 100;
        let limited_count = file_count.min(MAX_FILES);

        let mut files = Vec::new();
        for i in 0..limited_count {
            let len = DragQueryFileW(hdrop, i, std::ptr::null_mut(), 0) as usize;
            if len == 0 {
                continue;
            }
            // 限制路径长度
            const MAX_PATH_LEN: usize = 260;
            if len > MAX_PATH_LEN {
                continue;
            }
            let mut buf = vec![0u16; len + 1];
            DragQueryFileW(hdrop, i, buf.as_mut_ptr(), (len + 1) as u32);
            buf.pop(); // 移除 null terminator
            let path = OsString::from_wide(&buf).to_string_lossy().to_string();
            files.push(path);
        }

        if files.is_empty() {
            return Ok(None);
        }

        // 计算 hash 用于去重
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        files.hash(&mut hasher);
        let hash = hasher.finish();

        Ok(Some((files, hash)))
    }
}
