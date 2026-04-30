use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use super::history::CLIPBOARD_HISTORY;

static MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn start_clipboard_monitor() {
    if MONITOR_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(|| {
        let mut last_text: Option<String> = None;
        let mut last_image_hash: Option<u64> = None;
        let mut error_count = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 10;

        loop {
            // 先尝试读取图片（Windows API）
            #[cfg(target_os = "windows")]
            {
                match read_clipboard_image_win32() {
                    Ok(Some((image_data, hash))) => {
                        error_count = 0; // 重置错误计数
                        if Some(hash) != last_image_hash {
                            last_image_hash = Some(hash);
                            last_text = None;

                            if let Ok(mut history) = CLIPBOARD_HISTORY.lock() {
                                history.add_image(image_data);
                            } else {
                                eprintln!("[clipboard] 获取历史记录锁失败");
                            }
                        }
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    Ok(None) => {} // 没有图片
                    Err(e) => {
                        error_count += 1;
                        if error_count <= MAX_CONSECUTIVE_ERRORS {
                            eprintln!("[clipboard] 读取图片失败 ({}): {}", error_count, e);
                        }
                    }
                }
            }

            // 没有图片，尝试读取文本
            match read_clipboard_text_win32() {
                Ok(Some(text)) => {
                    error_count = 0; // 重置错误计数
                    if text.trim().is_empty() {
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    if Some(&text) != last_text.as_ref() {
                        last_text = Some(text.clone());
                        last_image_hash = None;

                        if let Ok(mut history) = CLIPBOARD_HISTORY.lock() {
                            history.add_text(text);
                        } else {
                            eprintln!("[clipboard] 获取历史记录锁失败");
                        }
                    }
                }
                Ok(None) => {} // 空剪贴板
                Err(e) => {
                    error_count += 1;
                    if error_count <= MAX_CONSECUTIVE_ERRORS {
                        eprintln!("[clipboard] 读取文本失败 ({}): {}", error_count, e);
                    }
                }
            }

            // 如果连续错误过多，增加等待时间避免日志刷屏
            if error_count > MAX_CONSECUTIVE_ERRORS {
                thread::sleep(Duration::from_secs(5));
            } else {
                thread::sleep(Duration::from_millis(500));
            }
        }
    });
}

#[cfg(target_os = "windows")]
fn read_clipboard_text_win32() -> Result<Option<String>, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use winapi::um::winuser::{OpenClipboard, CloseClipboard, GetClipboardData, CF_UNICODETEXT};
    use winapi::um::winbase::GlobalLock;
    use winapi::shared::ntdef::HANDLE;

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            // 剪贴板可能被其他程序占用，这不是致命错误
            return Ok(None);
        }

        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle.is_null() {
            CloseClipboard();
            return Ok(None);
        }

        let ptr = GlobalLock(handle as HANDLE);
        if ptr.is_null() {
            CloseClipboard();
            return Err("GlobalLock 返回空指针".to_string());
        }

        // 计算长度，添加安全检查
        let mut len = 0;
        let mut p = ptr as *const u16;
        let max_len = 1024 * 1024; // 最大 1MB 文本
        while *p != 0 && len < max_len {
            len += 1;
            p = p.offset(1);
        }

        if len >= max_len {
            winapi::um::winbase::GlobalUnlock(handle as HANDLE);
            CloseClipboard();
            return Err("剪贴板文本过长，已跳过".to_string());
        }

        let slice = std::slice::from_raw_parts(ptr as *const u16, len);
        let text = OsString::from_wide(slice).to_string_lossy().to_string();

        winapi::um::winbase::GlobalUnlock(handle as HANDLE);
        CloseClipboard();

        Ok(Some(text))
    }
}

#[cfg(target_os = "windows")]
fn read_clipboard_image_win32() -> Result<Option<(Vec<u8>, u64)>, String> {
    use winapi::um::winuser::{OpenClipboard, CloseClipboard, GetClipboardData, CF_DIB};
    use winapi::shared::ntdef::HANDLE;
    use winapi::um::winbase::{GlobalLock, GlobalSize};
    use winapi::um::wingdi::BITMAPINFOHEADER;
    use std::mem::size_of;

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Ok(None);
        }

        let handle = GetClipboardData(CF_DIB);
        if handle.is_null() {
            CloseClipboard();
            return Ok(None);
        }

        let ptr = GlobalLock(handle as HANDLE);
        if ptr.is_null() {
            CloseClipboard();
            return Err("GlobalLock 返回空指针".to_string());
        }

        let size = GlobalSize(handle as HANDLE) as usize;

        // 安全检查：限制最大图片大小（50MB）
        const MAX_IMAGE_SIZE: usize = 50 * 1024 * 1024;
        if size > MAX_IMAGE_SIZE {
            winapi::um::winbase::GlobalUnlock(handle as HANDLE);
            CloseClipboard();
            return Err(format!("图片过大 ({} MB)，已跳过", size / 1024 / 1024));
        }

        if size < size_of::<BITMAPINFOHEADER>() {
            winapi::um::winbase::GlobalUnlock(handle as HANDLE);
            CloseClipboard();
            return Err("DIB 数据过小".to_string());
        }

        let data = std::slice::from_raw_parts(ptr as *const u8, size);

        // 解析 BITMAPINFOHEADER
        let header = &*(data.as_ptr() as *const BITMAPINFOHEADER);

        // 验证图片尺寸合理性
        let width = header.biWidth.unsigned_abs();
        let height = header.biHeight.unsigned_abs();
        if width == 0 || height == 0 || width > 16384 || height > 16384 {
            winapi::um::winbase::GlobalUnlock(handle as HANDLE);
            CloseClipboard();
            return Err(format!("图片尺寸不合理: {}x{}", width, height));
        }

        let bit_count = header.biBitCount;

        // 计算像素数据偏移
        let header_size = header.biSize as usize;
        let pixel_offset = header_size + if bit_count <= 8 { 256 * 4 } else { 0 };

        if data.len() < pixel_offset {
            winapi::um::winbase::GlobalUnlock(handle as HANDLE);
            CloseClipboard();
            return Err("DIB 数据过短".to_string());
        }

        let pixel_data = &data[pixel_offset..];
        let hash = calculate_hash(pixel_data);

        // 转换为 PNG
        let png_data = dib_to_png(data, width, height, bit_count)?;

        winapi::um::winbase::GlobalUnlock(handle as HANDLE);
        CloseClipboard();

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
