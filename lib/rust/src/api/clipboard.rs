use crate::clipboard::history::{ClipboardItem, get_clipboard_history as get_history};
use base64::Engine;

/// 获取剪贴板历史记录
pub fn get_clipboard_history() -> Vec<ClipboardItem> {
    let history = get_history();
    let items = history.lock().map(|h| h.get_items().to_vec()).unwrap_or_default();
    // 将图片文件转为缩略图 base64 供前端显示 (max 256px, 大幅节省内存)
    items.into_iter().map(|mut item| {
        if item.content_type == "image" {
            if let Some(ref path) = item.content {
                if let Ok(bytes) = std::fs::read(path) {
                    let b64 = if let Ok(img) = image::load_from_memory(&bytes) {
                        let thumb = img.thumbnail(256, 256);
                        let mut buf = std::io::Cursor::new(Vec::new());
                        if thumb.write_to(&mut buf, image::ImageFormat::Png).is_ok() {
                            base64::engine::general_purpose::STANDARD.encode(buf.get_ref())
                        } else {
                            base64::engine::general_purpose::STANDARD.encode(&bytes)
                        }
                    } else {
                        base64::engine::general_purpose::STANDARD.encode(&bytes)
                    };
                    item.content = Some(format!("data:image/png;base64,{}", b64));
                }
            }
        }
        item
    }).collect()
}

/// 获取剪贴板图片的原始大小 base64（用于复制粘贴）
pub fn get_clipboard_image_full_base64(id: String) -> Option<String> {
    let history = get_history();
    let items = history.lock().ok()?;
    let item = items.get_items().iter().find(|i| i.id == id)?;
    if item.content_type == "image" {
        if let Some(ref path) = item.content {
            if let Ok(bytes) = std::fs::read(path) {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                return Some(format!("data:image/png;base64,{}", b64));
            }
        }
    }
    None
}

/// 删除指定剪贴板记录
pub fn delete_clipboard_item(id: String) {
    let history = get_history();
    if let Ok(mut h) = history.lock() {
        h.remove_item(&id);
    }
}

/// 清空剪贴板历史
pub fn clear_clipboard_history() {
    let history = get_history();
    if let Ok(mut h) = history.lock() {
        h.clear();
    }
}

/// 切换固定状态
pub fn toggle_pin_clipboard_item(id: String) {
    let history = get_history();
    if let Ok(mut h) = history.lock() {
        h.toggle_pin(&id);
    }
}

/// 复制内容到剪贴板
pub fn copy_to_clipboard(content: String) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::{OpenClipboard, CloseClipboard, SetClipboardData, EmptyClipboard, CF_UNICODETEXT};
    use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
    use winapi::shared::ntdef::HANDLE;

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err("无法打开剪贴板".to_string());
        }

        struct ClipboardGuard;
        impl Drop for ClipboardGuard {
            fn drop(&mut self) {
                unsafe { CloseClipboard(); }
            }
        }
        let _guard = ClipboardGuard;

        EmptyClipboard();

        let wide: Vec<u16> = OsStr::new(&content).encode_wide().chain(std::iter::once(0)).collect();
        let byte_len = wide.len() * 2;

        let handle = GlobalAlloc(GHND, byte_len);
        if handle.is_null() {
            return Err("内存分配失败".to_string());
        }

        let ptr = GlobalLock(handle as HANDLE);
        if ptr.is_null() {
            return Err("锁定内存失败".to_string());
        }

        struct LockGuard(HANDLE);
        impl Drop for LockGuard {
            fn drop(&mut self) {
                unsafe { GlobalUnlock(self.0); }
            }
        }
        let _lock_guard = LockGuard(handle as HANDLE);

        std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, ptr as *mut u8, byte_len);

        if SetClipboardData(CF_UNICODETEXT, handle as HANDLE).is_null() {
            return Err("设置剪贴板数据失败".to_string());
        }

        Ok(())
    }
}

/// 复制图片到剪贴板（传入 PNG base64 数据）
pub fn copy_image_to_clipboard(base64_data: String) -> Result<(), String> {
    use winapi::um::winuser::{OpenClipboard, CloseClipboard, SetClipboardData, EmptyClipboard, CF_DIB};
    use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
    use winapi::shared::ntdef::HANDLE;

    let png_bytes = base64::engine::general_purpose::STANDARD.decode(&base64_data)
        .map_err(|e| format!("base64 解码失败: {}", e))?;

    let image = image::load_from_memory(&png_bytes)
        .map_err(|e| format!("图片解码失败: {}", e))?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();

    let row_size = ((width * 3 + 3) & !3) as usize;
    let pixel_data_size = row_size * height as usize;
    let header_size = 40;
    let dib_size = header_size + pixel_data_size;

    let mut dib = vec![0u8; dib_size];

    dib[0..4].copy_from_slice(&(header_size as u32).to_le_bytes());
    dib[4..8].copy_from_slice(&(width as i32).to_le_bytes());
    dib[8..12].copy_from_slice(&(height as i32).to_le_bytes());
    dib[12..14].copy_from_slice(&1u16.to_le_bytes());
    dib[14..16].copy_from_slice(&24u16.to_le_bytes());

    let raw_pixels = rgba.as_raw();
    for y in 0..height {
        let src_row = ((height - 1 - y) as usize) * width as usize * 4;
        let dst_row = y as usize * row_size;
        for x in 0..width {
            let src_idx = src_row + x as usize * 4;
            let dst_idx = dst_row + x as usize * 3;
            dib[header_size + dst_idx] = raw_pixels[src_idx + 2];
            dib[header_size + dst_idx + 1] = raw_pixels[src_idx + 1];
            dib[header_size + dst_idx + 2] = raw_pixels[src_idx];
        }
    }

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err("无法打开剪贴板".to_string());
        }

        struct ClipboardGuard;
        impl Drop for ClipboardGuard {
            fn drop(&mut self) {
                unsafe { CloseClipboard(); }
            }
        }
        let _guard = ClipboardGuard;

        EmptyClipboard();

        let handle = GlobalAlloc(GHND, dib_size);
        if handle.is_null() {
            return Err("内存分配失败".to_string());
        }

        let ptr = GlobalLock(handle as HANDLE);
        if ptr.is_null() {
            return Err("锁定内存失败".to_string());
        }

        struct LockGuard(HANDLE);
        impl Drop for LockGuard {
            fn drop(&mut self) {
                unsafe { GlobalUnlock(self.0); }
            }
        }
        let _lock_guard = LockGuard(handle as HANDLE);

        std::ptr::copy_nonoverlapping(dib.as_ptr(), ptr as *mut u8, dib_size);

        if SetClipboardData(CF_DIB, handle as HANDLE).is_null() {
            return Err("设置剪贴板数据失败".to_string());
        }

        Ok(())
    }
}
