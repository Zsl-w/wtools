use crate::clipboard::history::{ClipboardItem, CLIPBOARD_HISTORY};

/// 获取剪贴板历史记录
#[tauri::command]
pub fn get_clipboard_history() -> Vec<ClipboardItem> {
    CLIPBOARD_HISTORY.lock().unwrap().get_items().to_vec()
}

/// 删除指定剪贴板记录
#[tauri::command]
pub fn delete_clipboard_item(id: String) {
    CLIPBOARD_HISTORY.lock().unwrap().remove_item(&id);
}

/// 清空剪贴板历史
#[tauri::command]
pub fn clear_clipboard_history() {
    CLIPBOARD_HISTORY.lock().unwrap().clear();
}

/// 复制内容到剪贴板
#[tauri::command]
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
        
        EmptyClipboard();
        
        // 转换为 UTF-16
        let wide: Vec<u16> = OsStr::new(&content).encode_wide().chain(std::iter::once(0)).collect();
        let byte_len = wide.len() * 2;
        
        let handle = GlobalAlloc(GHND, byte_len);
        if handle.is_null() {
            CloseClipboard();
            return Err("内存分配失败".to_string());
        }
        
        let ptr = GlobalLock(handle as HANDLE);
        if ptr.is_null() {
            CloseClipboard();
            return Err("锁定内存失败".to_string());
        }
        
        std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, ptr as *mut u8, byte_len);
        
        GlobalUnlock(handle as HANDLE);
        
        if SetClipboardData(CF_UNICODETEXT, handle as HANDLE).is_null() {
            CloseClipboard();
            return Err("设置剪贴板数据失败".to_string());
        }
        
        CloseClipboard();
        Ok(())
    }
}

/// 复制图片到剪贴板（传入 PNG base64 数据）
#[tauri::command]
pub fn copy_image_to_clipboard(base64_data: String) -> Result<(), String> {
    use winapi::um::winuser::{OpenClipboard, CloseClipboard, SetClipboardData, EmptyClipboard, CF_DIB};
    use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
    use winapi::shared::ntdef::HANDLE;
    
    // base64 解码为 PNG
    let png_bytes = base64::decode(&base64_data).map_err(|e| format!("base64 解码失败: {}", e))?;
    
    // PNG 转 BMP (DIB)
    let image = image::load_from_memory(&png_bytes).map_err(|e| format!("图片解码失败: {}", e))?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    
    // DIB 结构: BITMAPINFOHEADER + 像素数据（BGR, 每行 4 字节对齐）
    let row_size = ((width * 3 + 3) & !3) as usize;
    let pixel_data_size = row_size * height as usize;
    let header_size = 40;
    let dib_size = header_size + pixel_data_size;
    
    let mut dib = vec![0u8; dib_size];
    
    // BITMAPINFOHEADER
    dib[0..4].copy_from_slice(&(header_size as u32).to_le_bytes());
    dib[4..8].copy_from_slice(&(width as i32).to_le_bytes());
    dib[8..12].copy_from_slice(&(height as i32).to_le_bytes()); // 正高度 = bottom-up
    dib[12..14].copy_from_slice(&1u16.to_le_bytes()); // planes
    dib[14..16].copy_from_slice(&24u16.to_le_bytes()); // bits per pixel
    
    // RGBA -> BGR, bottom-up
    let raw_pixels = rgba.as_raw();
    for y in 0..height {
        let src_row = ((height - 1 - y) as usize) * width as usize * 4;
        let dst_row = y as usize * row_size;
        for x in 0..width {
            let src_idx = src_row + x as usize * 4;
            let dst_idx = dst_row + x as usize * 3;
            dib[header_size + dst_idx] = raw_pixels[src_idx + 2];     // B
            dib[header_size + dst_idx + 1] = raw_pixels[src_idx + 1]; // G
            dib[header_size + dst_idx + 2] = raw_pixels[src_idx];     // R
        }
    }
    
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err("无法打开剪贴板".to_string());
        }
        
        EmptyClipboard();
        
        let handle = GlobalAlloc(GHND, dib_size);
        if handle.is_null() {
            CloseClipboard();
            return Err("内存分配失败".to_string());
        }
        
        let ptr = GlobalLock(handle as HANDLE);
        if ptr.is_null() {
            CloseClipboard();
            return Err("锁定内存失败".to_string());
        }
        
        std::ptr::copy_nonoverlapping(dib.as_ptr(), ptr as *mut u8, dib_size);
        
        GlobalUnlock(handle as HANDLE);
        
        if SetClipboardData(CF_DIB, handle as HANDLE).is_null() {
            CloseClipboard();
            return Err("设置剪贴板数据失败".to_string());
        }
        
        CloseClipboard();
        Ok(())
    }
}
