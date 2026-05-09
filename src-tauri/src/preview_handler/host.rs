use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use base64::Engine;

use winapi::shared::guiddef::{CLSID, GUID};
use winapi::shared::windef::{HWND, RECT};
use winapi::shared::winerror::S_OK;
use winapi::um::combaseapi::{CoCreateInstance, CoInitializeEx, CoUninitialize};
use winapi::um::objbase::COINIT_APARTMENTTHREADED;
use winapi::um::winreg::{RegOpenKeyExW, RegQueryValueExW, HKEY_CLASSES_ROOT};
const KEY_READ: u32 = 0x20019;
use winapi::um::winuser::{
    CreateWindowExW, DestroyWindow, GetDC, ReleaseDC, ShowWindow, PrintWindow,
    InvalidateRect, UpdateWindow, EnumChildWindows, GetClientRect, FillRect,
    RegisterClassW, WNDCLASSW, CS_HREDRAW, CS_VREDRAW, SW_SHOW,
    PW_RENDERFULLCONTENT,
};
use winapi::um::wingdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject,
    BitBlt, SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, BI_RGB,
};
const CLSCTX_INPROC_SERVER: u32 = 0x1;

use super::com_interfaces::*;

// Preview Handler Shell Extension CLSID
const IID_PREVIEW_HANDLER: GUID = GUID {
    Data1: 0x8895b1c6,
    Data2: 0xb41f,
    Data3: 0x4c1c,
    Data4: [0xa5, 0x62, 0x0d, 0x56, 0x42, 0x50, 0x83, 0x6f],
};

const STGM_READ: u32 = 0x00000000;

/// 从注册表查找文件扩展名对应的 Preview Handler CLSID
pub fn find_handler_clsid(ext: &str) -> Option<CLSID> {
    // 尝试路径列表
    let search_paths = vec![
        format!(".{}\\shellex\\{{8895b1c6-b41f-4c1c-a562-0d564250836f}}", ext),
        format!("SystemFileAssociations\\.{}\\shellex\\{{8895b1c6-b41f-4c1c-a562-0d564250836f}}", ext),
    ];

    for path in &search_paths {
        eprintln!("[preview] 查找注册表: HKCR\\{}", path);
        if let Some(clsid) = read_clsid_from_registry(path) {
            return Some(clsid);
        }
    }

    // 还需要通过 ProgID 查找
    let prog_id = read_registry_default(&format!(".{}", ext));
    eprintln!("[preview] .{} ProgID: {:?}", ext, prog_id);
    if let Some(ref pid) = prog_id {
        let prog_path = format!(
            "{}\\shellex\\{{8895b1c6-b41f-4c1c-a562-0d564250836f}}",
            pid
        );
        eprintln!("[preview] 查找注册表: HKCR\\{}", prog_path);
        if let Some(clsid) = read_clsid_from_registry(&prog_path) {
            return Some(clsid);
        }
    }

    eprintln!("[preview] 所有注册表路径均未找到 handler");
    None
}

fn read_registry_default(key_path: &str) -> Option<String> {
    let key_w: Vec<u16> = OsStr::new(key_path).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        let mut hkey = ptr::null_mut();
        let result = RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            key_w.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        );
        if result != 0 {
            return None;
        }

        let mut buf = [0u16; 256];
        let mut buf_size = (buf.len() * 2) as u32;
        let mut val_type = 0u32;

        let result = RegQueryValueExW(
            hkey,
            ptr::null(), // 默认值
            ptr::null_mut(),
            &mut val_type,
            buf.as_mut_ptr() as *mut u8,
            &mut buf_size,
        );

        winapi::um::winreg::RegCloseKey(hkey);

        if result != 0 {
            return None;
        }

        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        if len == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..len]))
    }
}

fn read_clsid_from_registry(key_path: &str) -> Option<CLSID> {
    let key_w: Vec<u16> = OsStr::new(key_path).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        let mut hkey = ptr::null_mut();
        let result = RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            key_w.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        );
        if result != 0 {
            return None;
        }

        let mut buf = [0u16; 64];
        let mut buf_size = (buf.len() * 2) as u32;
        let mut val_type = 0u32;

        let result = RegQueryValueExW(
            hkey,
            ptr::null(), // 默认值
            ptr::null_mut(),
            &mut val_type,
            buf.as_mut_ptr() as *mut u8,
            &mut buf_size,
        );

        winapi::um::winreg::RegCloseKey(hkey);

        if result != 0 {
            return None;
        }

        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        if len == 0 {
            return None;
        }

        let clsid_str = String::from_utf16_lossy(&buf[..len]);
        parse_clsid(&clsid_str)
    }
}

fn parse_clsid(s: &str) -> Option<CLSID> {
    let s = s.trim();
    let s = s.trim_start_matches('{').trim_end_matches('}');
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return None;
    }

    let data1 = u32::from_str_radix(parts[0], 16).ok()?;
    let data2 = u16::from_str_radix(parts[1], 16).ok()?;
    let data3 = u16::from_str_radix(parts[2], 16).ok()?;
    let data4_hi = u16::from_str_radix(parts[3], 16).ok()?;

    // 解析最后 12 个十六进制字符为 6 字节
    let hex_str = parts[4];
    if hex_str.len() != 12 {
        return None;
    }
    let mut data4_lo = [0u8; 6];
    for i in 0..6 {
        data4_lo[i] = u8::from_str_radix(&hex_str[i * 2..i * 2 + 2], 16).ok()?;
    }

    Some(CLSID {
        Data1: data1,
        Data2: data2,
        Data3: data3,
        Data4: [
            (data4_hi >> 8) as u8,
            (data4_hi & 0xFF) as u8,
            data4_lo[0], data4_lo[1], data4_lo[2],
            data4_lo[3], data4_lo[4], data4_lo[5],
        ],
    })
}

// ── 离屏渲染 ──────────────────────────────────────────────────

/// 使用 Preview Handler 渲染文件并返回 PNG base64
///
/// `width` / `height`: 渲染目标尺寸（像素）
pub fn render_preview(path: &str, width: i32, height: i32) -> Option<String> {
    let ext = std::path::Path::new(path)
        .extension()?
        .to_string_lossy()
        .to_lowercase();

    eprintln!("[preview] 尝试渲染: {} (扩展名: {})", path, ext);

    let clsid = match find_handler_clsid(&ext) {
        Some(c) => {
            eprintln!("[preview] 找到 handler CLSID: {:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                c.Data1, c.Data2, c.Data3,
                c.Data4[0], c.Data4[1], c.Data4[2], c.Data4[3], c.Data4[4], c.Data4[5], c.Data4[6], c.Data4[7]);
            c
        }
        None => {
            eprintln!("[preview] 未找到 {} 的 Preview Handler", ext);
            return None;
        }
    };

    let path_w: Vec<u16> = OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();

    unsafe {
        // 初始化 COM
        let hr = CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED);
        if hr != S_OK && hr != 0x00000001 /* S_FALSE (already initialized) */ {
            return None;
        }

        let result = do_render(&clsid, path_w.as_ptr(), width, height);

        CoUninitialize();
        result
    }
}

unsafe fn do_render(clsid: &CLSID, path_ptr: *const u16, width: i32, height: i32) -> Option<String> {
    // 创建 Preview Handler 实例
    let mut handler_ptr: *mut IPreviewHandler = ptr::null_mut();
    let hr = CoCreateInstance(
        clsid,
        ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_PREVIEW_HANDLER as *const GUID as *const _,
        &mut handler_ptr as *mut *mut _ as *mut *mut _,
    );
    if hr != S_OK || handler_ptr.is_null() {
        eprintln!("[preview] CoCreateInstance 失败: hr=0x{:08X}", hr);
        return None;
    }
    eprintln!("[preview] CoCreateInstance 成功");

    let handler = &*handler_ptr;

    // 尝试通过 IInitializeWithFile 初始化
    let mut init_file_ptr: *mut IInitializeWithFile = ptr::null_mut();
    let iid_init_file = GUID {
        Data1: 0xb7d14566,
        Data2: 0x0509,
        Data3: 0x4cce,
        Data4: [0xa7, 0x1f, 0x0a, 0x55, 0x42, 0x33, 0xbd, 0x9b],
    };

    let hr = handler.query_interface(
        &iid_init_file as *const GUID as *const _,
        &mut init_file_ptr as *mut *mut _ as *mut *mut _,
    );

    if hr != S_OK || init_file_ptr.is_null() {
        eprintln!("[preview] QueryInterface(IInitializeWithFile) 失败: hr=0x{:08X}", hr);
        handler.release();
        return None;
    }
    eprintln!("[preview] IInitializeWithFile 接口获取成功");

    let init_file = &*init_file_ptr;
    let hr = init_file.initialize(path_ptr, STGM_READ);
    if hr != S_OK {
        eprintln!("[preview] Initialize 失败: hr=0x{:08X}", hr);
        init_file.release();
        handler.release();
        return None;
    }

    eprintln!("[preview] Initialize 成功");

    // 创建隐藏的宿主窗口
    let hwnd_host = create_host_window(width, height);
    if hwnd_host.is_null() {
        eprintln!("[preview] 创建宿主窗口失败");
        init_file.release();
        handler.release();
        return None;
    }
    eprintln!("[preview] 宿主窗口创建成功");

    // 创建 Site 对象并设置给 handler（部分 handler 需要通过 IOleWindow 获取宿主窗口句柄）
    let site = PreviewHandlerSite::new(hwnd_host);
    let iid_object_with_site = GUID {
        Data1: 0xfc4801a3,
        Data2: 0x2ba9,
        Data3: 0x11cf,
        Data4: [0xa2, 0x29, 0x00, 0xaa, 0x00, 0x3d, 0x73, 0x52],
    };
    let mut site_holder_ptr: *mut IObjectWithSite = ptr::null_mut();
    let hr = handler.query_interface(
        &iid_object_with_site as *const GUID as *const _,
        &mut site_holder_ptr as *mut *mut _ as *mut *mut _,
    );
    if hr == S_OK && !site_holder_ptr.is_null() {
        let site_holder = &*site_holder_ptr;
        let hr = site_holder.set_site(site as *mut IUnknown);
        eprintln!("[preview] SetSite 结果: hr=0x{:08X}", hr);
        site_holder.release();
    } else {
        eprintln!("[preview] QueryInterface(IObjectWithSite) 失败: hr=0x{:08X}（部分 handler 可能不需要）", hr);
        // 释放未使用的 site
        site_release(site);
    }

    // 设置 handler 的窗口和区域
    let rect = RECT {
        left: 0,
        top: 0,
        right: width,
        bottom: height,
    };

    let hr = handler.set_window(hwnd_host, &rect);
    if hr != S_OK {
        eprintln!("[preview] SetWindow 失败: hr=0x{:08X}", hr);
        DestroyWindow(hwnd_host);
        init_file.release();
        handler.release();
        return None;
    }

    let hr = handler.set_rect(&rect);
    if hr != S_OK {
        eprintln!("[preview] SetRect 失败: hr=0x{:08X}", hr);
        handler.unload();
        DestroyWindow(hwnd_host);
        init_file.release();
        handler.release();
        return None;
    }

    // 将窗口移到屏幕外并显示（handler 需要可见窗口，但不能让用户看到闪烁）
    use winapi::um::winuser::SetWindowPos;
    SetWindowPos(hwnd_host, ptr::null_mut(), -width - 100, -height - 100, 0, 0,
        winapi::um::winuser::SWP_NOSIZE | winapi::um::winuser::SWP_NOZORDER);
    ShowWindow(hwnd_host, SW_SHOW);

    // 执行预览渲染
    eprintln!("[preview] 调用 DoPreview...");
    let hr = handler.do_preview();
    if hr != S_OK {
        eprintln!("[preview] DoPreview 失败: hr=0x{:08X}", hr);
        handler.unload();
        DestroyWindow(hwnd_host);
        init_file.release();
        handler.release();
        return None;
    }
    eprintln!("[preview] DoPreview 成功");

    // 等待渲染完成（Office handler 可能需要更长时间异步渲染）
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 强制窗口重绘
    InvalidateRect(hwnd_host, ptr::null(), 1);
    UpdateWindow(hwnd_host);

    // 截取渲染结果
    eprintln!("[preview] 截取渲染结果...");
    let mut png_data = capture_window_to_png(hwnd_host, width, height);

    // 如果截取失败或图像为空白，等待更长时间后重试
    if png_data.is_none() {
        eprintln!("[preview] 首次截取失败，等待 1 秒后重试...");
        std::thread::sleep(std::time::Duration::from_millis(1000));
        InvalidateRect(hwnd_host, ptr::null(), 1);
        UpdateWindow(hwnd_host);
        png_data = capture_window_to_png(hwnd_host, width, height);
    }

    match &png_data {
        Some(data) => eprintln!("[preview] 截取成功, base64 长度: {}", data.len()),
        None => eprintln!("[preview] 截取失败 (返回 None)"),
    }

    // 清理
    handler.unload();
    DestroyWindow(hwnd_host);
    init_file.release();
    handler.release();

    png_data
}

/// 创建一个隐藏的弹出窗口作为 Preview Handler 的宿主
unsafe fn create_host_window(width: i32, height: i32) -> HWND {
    let class_name: Vec<u16> = OsStr::new("wToolsPreviewHost")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // 先尝试注册窗口类（如果已注册则忽略错误）
    let wc = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(def_window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: winapi::um::libloaderapi::GetModuleHandleW(ptr::null()),
        hIcon: ptr::null_mut(),
        hCursor: ptr::null_mut(),
        hbrBackground: ptr::null_mut(),
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };
    RegisterClassW(&wc);

    // 使用 WS_POPUP（无需父窗口），创建后立即隐藏
    let hwnd = CreateWindowExW(
        0,
        class_name.as_ptr(),
        ptr::null(),
        winapi::um::winuser::WS_POPUP,
        0, 0, width, height,
        ptr::null_mut(),
        ptr::null_mut(),
        winapi::um::libloaderapi::GetModuleHandleW(ptr::null()),
        ptr::null_mut(),
    );

    if hwnd.is_null() {
        eprintln!("[preview] CreateWindowExW 失败, GetLastError={}", winapi::um::errhandlingapi::GetLastError());
    }

    hwnd
}

unsafe extern "system" fn def_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    winapi::um::winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn get_window_class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = winapi::um::winuser::GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    String::from_utf16_lossy(&buf[..len as usize])
}

/// 从窗口 DC 截取像素并编码为 PNG base64
unsafe fn capture_window_to_png(hwnd: HWND, width: i32, height: i32) -> Option<String> {
    let hdc_window = GetDC(hwnd);
    if hdc_window.is_null() {
        return None;
    }

    let hdc_mem = CreateCompatibleDC(hdc_window);
    if hdc_mem.is_null() {
        ReleaseDC(hwnd, hdc_window);
        return None;
    }

    // 创建 DIB section 用于读取像素
    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [winapi::um::wingdi::RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }; 1],
    };

    let mut pixels: *mut u8 = ptr::null_mut();
    let hbitmap = CreateDIBSection(
        hdc_mem,
        &bmi,
        DIB_RGB_COLORS,
        &mut pixels as *mut *mut u8 as *mut *mut _,
        ptr::null_mut(),
        0,
    );

    if hbitmap.is_null() {
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_window);
        return None;
    }

    let old_bmp = SelectObject(hdc_mem, hbitmap as _);

    // 枚举子窗口，找到 Preview Handler 的实际渲染窗口
    let mut child_windows: Vec<HWND> = Vec::new();
    unsafe extern "system" fn enum_children(hwnd: HWND, lparam: isize) -> i32 {
        let children = &mut *(lparam as *mut Vec<HWND>);
        children.push(hwnd);
        1 // continue enumeration
    }
    EnumChildWindows(hwnd, Some(enum_children), &mut child_windows as *mut _ as _);

    let mut captured = false;

    if !child_windows.is_empty() {
        // 找到最大的子窗口（通常是 handler 的渲染窗口）
        let mut best_hwnd: HWND = ptr::null_mut();
        let mut best_area = 0i32;
        for &child in &child_windows {
            let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            GetClientRect(child, &mut rc);
            let area = (rc.right - rc.left) * (rc.bottom - rc.top);
            let class_name = get_window_class_name(child);
            eprintln!("[preview] 子窗口: hwnd={:?}, class='{}', area={}", child, class_name, area);
            if area > best_area {
                best_area = area;
                best_hwnd = child;
            }
        }

        if !best_hwnd.is_null() && best_area > 0 {
            eprintln!("[preview] 尝试从最大子窗口捕获 (area={})...", best_area);
            let hdc_child = GetDC(best_hwnd);
            if !hdc_child.is_null() {
                let mut child_rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                GetClientRect(best_hwnd, &mut child_rc);
                let cw = child_rc.right - child_rc.left;
                let ch = child_rc.bottom - child_rc.top;

                if cw > 0 && ch > 0 {
                    // 先用白色填充背景
                    let white_brush = winapi::um::wingdi::CreateSolidBrush(0x00FFFFFF);
                    let fill_rc = RECT { left: 0, top: 0, right: width, bottom: height };
                    FillRect(hdc_mem, &fill_rc, white_brush);
                    winapi::um::wingdi::DeleteObject(white_brush as _);

                    // 从子窗口 DC 拷贝
                    BitBlt(
                        hdc_mem,
                        0, 0, cw.min(width), ch.min(height),
                        hdc_child,
                        0, 0,
                        SRCCOPY,
                    );
                    captured = true;
                    eprintln!("[preview] 子窗口 BitBlt 完成 ({}x{})", cw.min(width), ch.min(height));
                }
                ReleaseDC(best_hwnd, hdc_child);
            }
        }
    } else {
        eprintln!("[preview] 没有子窗口");
    }

    if !captured {
        // 回退：用白色填充背景后 PrintWindow
        let white_brush = winapi::um::wingdi::CreateSolidBrush(0x00FFFFFF);
        let fill_rc = RECT { left: 0, top: 0, right: width, bottom: height };
        FillRect(hdc_mem, &fill_rc, white_brush);
        winapi::um::wingdi::DeleteObject(white_brush as _);

        let pw_result = PrintWindow(hwnd, hdc_mem, PW_RENDERFULLCONTENT);
        if pw_result == 0 {
            eprintln!("[preview] PrintWindow 失败, 回退到 BitBlt");
            BitBlt(hdc_mem, 0, 0, width, height, hdc_window, 0, 0, SRCCOPY);
        } else {
            eprintln!("[preview] PrintWindow 成功");
        }
    }

    // 读取像素数据
    let total_pixels = (width * height) as usize;
    let pixel_slice = std::slice::from_raw_parts(pixels, total_pixels * 4);

    // 诊断：检查像素数据
    let non_zero_count = pixel_slice.iter().filter(|&&b| b != 0).count();
    eprintln!("[preview] 像素数据: 总字节={}, 非零字节={}", pixel_slice.len(), non_zero_count);

    // BGRA -> RGBA
    let mut rgba_data = vec![0u8; total_pixels * 4];
    for i in 0..total_pixels {
        let offset = i * 4;
        rgba_data[offset] = pixel_slice[offset + 2];     // R
        rgba_data[offset + 1] = pixel_slice[offset + 1]; // G
        rgba_data[offset + 2] = pixel_slice[offset];     // B
        rgba_data[offset + 3] = pixel_slice[offset + 3]; // A
    }

    // 编码为 PNG
    let png_data = encode_rgba_to_png(&rgba_data, width as u32, height as u32)?;

    // 清理
    SelectObject(hdc_mem, old_bmp);
    DeleteObject(hbitmap as _);
    DeleteDC(hdc_mem);
    ReleaseDC(hwnd, hdc_window);

    // 调试：保存 PNG 到临时文件
    let debug_path = std::env::temp_dir().join("wtools_preview_debug.png");
    if let Err(e) = std::fs::write(&debug_path, &png_data) {
        eprintln!("[preview] 保存调试 PNG 失败: {}", e);
    } else {
        eprintln!("[preview] 调试 PNG 已保存到: {}", debug_path.display());
    }

    // 转 base64
    Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&png_data)))
}

fn encode_rgba_to_png(rgba: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    use image::ImageEncoder;
    use image::codecs::png::PngEncoder;

    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(rgba, width, height, image::ColorType::Rgba8).ok()?;
    Some(png_data)
}
