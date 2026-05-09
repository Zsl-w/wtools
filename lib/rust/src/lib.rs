mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

mod api;
#[path = "clipboard/mod.rs"]
mod clipboard_impl;
mod everything;

pub(crate) use clipboard_impl as clipboard;
pub use api::*;

/// 防抖间隔（毫秒）
const DEBOUNCE_MS: u64 = 300;

/// 上次触发时间戳（毫秒）
static LAST_TRIGGER_MS: AtomicU64 = AtomicU64::new(0);

/// 热键触发标志 — Dart 端轮询此值
pub static HOTKEY_TOGGLE_FLAG: AtomicBool = AtomicBool::new(false);

/// 初始化后端：启动剪贴板监听、注册全局热键
pub fn init_backend() {
    clipboard::start_clipboard_monitor();
    start_hotkey_listener();
}

/// 关闭后端：停止剪贴板监听
pub fn shutdown_backend() {
    clipboard::stop_clipboard_monitor();
}

/// Dart 端轮询：是否有热键触发事件待处理
pub fn consume_hotkey_flag() -> bool {
    HOTKEY_TOGGLE_FLAG.swap(false, Ordering::AcqRel)
}

/// 检查是否应该处理触发（防抖）
pub fn should_process_trigger() -> bool {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let last_ms = LAST_TRIGGER_MS.load(Ordering::SeqCst);

    if last_ms > 0 && now_ms.saturating_sub(last_ms) < DEBOUNCE_MS {
        return false;
    }

    LAST_TRIGGER_MS.store(now_ms, Ordering::SeqCst);
    true
}

// ── 热键监听 ─────────────────────────────────────────────

#[cfg(target_os = "windows")]
pub(crate) fn start_hotkey_listener() {
    use std::thread;
    use winapi::um::winuser::{RegisterHotKey, GetMessageW, MOD_ALT, VK_SPACE, WM_HOTKEY};

    thread::spawn(move || {
        unsafe {
            // 注册 Alt+Space
            RegisterHotKey(std::ptr::null_mut(), 1, MOD_ALT as u32, VK_SPACE as u32);

            // 注册备用 Alt+`
            RegisterHotKey(std::ptr::null_mut(), 2, MOD_ALT as u32, 0xC0u32); // VK_OEM_3

            let mut msg: winapi::um::winuser::MSG = std::mem::zeroed();
            loop {
                let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
                if ret <= 0 {
                    break;
                }
                if msg.message == WM_HOTKEY {
                    if should_process_trigger() {
                        HOTKEY_TOGGLE_FLAG.store(true, Ordering::SeqCst);
                    }
                }
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn start_hotkey_listener() {
    // Non-Windows: no-op
}

// ── DWM / Window Chrome ──────────────────────────────────

/// 应用 DWM 无边框毛玻璃样式
/// `hwnd_value` 是 Flutter 窗口的原生 HWND 的 isize 表示
#[cfg(target_os = "windows")]
pub(crate) fn apply_dwm_borderless_impl(hwnd_value: isize) {
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{GetWindowLongW, SetWindowLongW, SetWindowPos,
        GWL_STYLE, GWL_EXSTYLE,
        WS_EX_WINDOWEDGE, WS_EX_CLIENTEDGE, WS_EX_STATICEDGE, WS_EX_LAYERED,
        WS_CAPTION, WS_THICKFRAME, WS_SYSMENU,
        SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_NOACTIVATE,
        WS_POPUP};
    use winapi::um::dwmapi::{DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DwmEnableBlurBehindWindow};
    use winapi::um::uxtheme::MARGINS;

    let hwnd = hwnd_value as HWND;
    unsafe {
        SetWindowLongW(hwnd, GWL_STYLE,
            (WS_POPUP as i32) & !(WS_CAPTION as i32 | WS_THICKFRAME as i32 | WS_SYSMENU as i32));

        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(hwnd, GWL_EXSTYLE,
            (ex_style | WS_EX_LAYERED as i32) &
            !(WS_EX_WINDOWEDGE as i32 | WS_EX_CLIENTEDGE as i32 | WS_EX_STATICEDGE as i32));

        let margins = MARGINS { cxLeftWidth: -1, cxRightWidth: -1, cyTopHeight: -1, cyBottomHeight: -1 };
        DwmExtendFrameIntoClientArea(hwnd, &margins);

        let color: u32 = 0xFFFFFFFE;
        let _ = DwmSetWindowAttribute(hwnd, 34, &color as *const _ as *const _, std::mem::size_of::<u32>() as u32);

        let corner_pref: u32 = 2;
        let _ = DwmSetWindowAttribute(hwnd, 33, &corner_pref as *const _ as *const _, std::mem::size_of::<u32>() as u32);

        let bb = winapi::um::dwmapi::DWM_BLURBEHIND {
            dwFlags: 0x00000001 | 0x00000002,
            fEnable: 1,
            hRgnBlur: std::ptr::null_mut(),
            fTransitionOnMaximized: 0,
        };
        let _ = DwmEnableBlurBehindWindow(hwnd, &bb);

        SetWindowPos(hwnd, std::ptr::null_mut(), 0, 0, 0, 0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
    }
}

/// 仅应用 DWM 属性，不调用 SetWindowPos（避免窗口闪烁）
#[cfg(target_os = "windows")]
pub(crate) fn refresh_dwm_attributes_impl(hwnd_value: isize) {
    use winapi::shared::windef::HWND;
    use winapi::um::dwmapi::{DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DwmEnableBlurBehindWindow};
    use winapi::um::uxtheme::MARGINS;

    let hwnd = hwnd_value as HWND;
    unsafe {
        let margins = MARGINS { cxLeftWidth: -1, cxRightWidth: -1, cyTopHeight: -1, cyBottomHeight: -1 };
        DwmExtendFrameIntoClientArea(hwnd, &margins);

        let color: u32 = 0xFFFFFFFE;
        let _ = DwmSetWindowAttribute(hwnd, 34, &color as *const _ as *const _, std::mem::size_of::<u32>() as u32);

        let corner_pref: u32 = 2;
        let _ = DwmSetWindowAttribute(hwnd, 33, &corner_pref as *const _ as *const _, std::mem::size_of::<u32>() as u32);

        let bb = winapi::um::dwmapi::DWM_BLURBEHIND {
            dwFlags: 0x00000001 | 0x00000002,
            fEnable: 1,
            hRgnBlur: std::ptr::null_mut(),
            fTransitionOnMaximized: 0,
        };
        let _ = DwmEnableBlurBehindWindow(hwnd, &bb);
    }
}

// Stubs for non-Windows
#[cfg(not(target_os = "windows"))]
pub(crate) fn apply_dwm_borderless_impl(_hwnd_value: isize) {}
#[cfg(not(target_os = "windows"))]
pub(crate) fn refresh_dwm_attributes_impl(_hwnd_value: isize) {}
