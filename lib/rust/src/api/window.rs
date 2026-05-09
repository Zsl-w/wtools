/// 初始化后端：启动剪贴板监听、注册全局热键
pub fn init_backend() {
    crate::clipboard::start_clipboard_monitor();
    crate::start_hotkey_listener();
}

/// 关闭后端：停止剪贴板监听
pub fn shutdown_backend() {
    crate::clipboard::stop_clipboard_monitor();
}

/// Dart 端轮询：是否有热键触发事件待处理
pub fn consume_hotkey_flag() -> bool {
    crate::HOTKEY_TOGGLE_FLAG.swap(false, std::sync::atomic::Ordering::AcqRel)
}

/// 应用 DWM 无边框毛玻璃样式
/// `hwnd_value` 是 Flutter 窗口的原生 HWND 的 isize 表示
#[cfg(target_os = "windows")]
pub fn apply_dwm_borderless(hwnd_value: isize) {
    crate::apply_dwm_borderless_impl(hwnd_value);
}

/// 仅应用 DWM 属性，不调用 SetWindowPos（避免窗口闪烁）
#[cfg(target_os = "windows")]
pub fn refresh_dwm_attributes(hwnd_value: isize) {
    crate::refresh_dwm_attributes_impl(hwnd_value);
}

// Stubs for non-Windows
#[cfg(not(target_os = "windows"))]
pub fn apply_dwm_borderless(_hwnd_value: isize) {}
#[cfg(not(target_os = "windows"))]
pub fn refresh_dwm_attributes(_hwnd_value: isize) {}
