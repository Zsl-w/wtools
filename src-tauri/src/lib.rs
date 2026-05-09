use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{Emitter, Manager};
use tauri::menu::{MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

mod commands;
mod everything;
mod clipboard;
mod preview_handler;

/// 防抖间隔（毫秒）
const DEBOUNCE_MS: u64 = 300;

/// 上次触发时间戳（毫秒），使用 AtomicU64 避免锁竞争
static LAST_TRIGGER_MS: AtomicU64 = AtomicU64::new(0);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::window::show_window,
            commands::window::hide_window,
            commands::app::get_installed_apps,
            commands::app::launch_app,
            commands::app::open_file,
            commands::app::show_in_folder,
            commands::app::get_app_icon_base64,
            commands::app::get_app_usage,
            commands::app::get_custom_apps,
            commands::app::add_custom_app,
            commands::app::remove_custom_app,
            commands::file::search_files,
            commands::file::get_image_thumbnail,
            commands::file::get_file_preview,
            commands::clipboard::get_clipboard_history,
            commands::clipboard::delete_clipboard_item,
            commands::clipboard::clear_clipboard_history,
            commands::clipboard::toggle_pin_clipboard_item,
            commands::clipboard::copy_to_clipboard,
            commands::clipboard::copy_image_to_clipboard,
        ])
        .setup(|app| {
            use tauri_plugin_autostart::ManagerExt;

            // 启动剪贴板监听
            clipboard::start_clipboard_monitor();

            // 检查当前自动启动状态
            let autostart_manager = app.autolaunch();
            let is_autostart_enabled = autostart_manager.is_enabled().unwrap_or(false);
            let autostart_label = if is_autostart_enabled { "✓ 开机自启动" } else { "开机自启动" };

            let show_item = MenuItem::with_id(app, "show", "显示搜索", true, None::<&str>)?;
            let autostart_item = MenuItem::with_id(app, "autostart", autostart_label, true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = tauri::menu::MenuBuilder::new(app)
                .item(&show_item)
                .item(&separator)
                .item(&autostart_item)
                .item(&quit_item)
                .build()?;

            TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("wTools - 快速搜索")
                .on_menu_event(move |app: &tauri::AppHandle, event| {
                    match event.id.as_ref() {
                        "show" => {
                            toggle_window(app);
                        }
                        "autostart" => {
                            toggle_autostart(app);
                        }
                        "quit" => {
                            // 停止剪贴板监听
                            clipboard::stop_clipboard_monitor();
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        let app = tray.app_handle().clone();
                        toggle_window(&app);
                    }
                })
                .build(app)?;

            // 尝试注册 Alt+Space，如果失败则使用 Alt+`
            let app_handle = app.handle().clone();
            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            let result = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
                if !should_process_trigger() {
                    return;
                }
                println!("Alt+Space 触发");
                toggle_window(&app_handle);
            });

            if result.is_err() {
                // Alt+Space 被占用，回退到 Alt+`
                let app_handle2 = app.handle().clone();
                let fallback = Shortcut::new(Some(Modifiers::ALT), Code::Backquote);
                app.global_shortcut().on_shortcut(fallback, move |_app, _shortcut, _event| {
                    if !should_process_trigger() {
                        return;
                    }
                    println!("Alt+` 触发");
                    toggle_window(&app_handle2);
                })?;
                println!("Alt+Space 已被占用，使用 Alt+` 作为替代");
            }

            // 获取主窗口
            if let Some(main_window) = app.get_webview_window("main") {
                // Windows: 使用DWM API移除系统边框
                #[cfg(target_os = "windows")]
                apply_borderless_style(&main_window);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 检查是否应该处理触发（防抖）
fn should_process_trigger() -> bool {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let last_ms = LAST_TRIGGER_MS.load(Ordering::SeqCst);

    if last_ms > 0 && now_ms.saturating_sub(last_ms) < DEBOUNCE_MS {
        println!("忽略重复触发 (间隔: {}ms)", now_ms.saturating_sub(last_ms));
        return false;
    }

    LAST_TRIGGER_MS.store(now_ms, Ordering::SeqCst);
    true
}

#[cfg(target_os = "windows")]
fn apply_borderless_style(window: &tauri::WebviewWindow) {
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{GetWindowLongW, SetWindowLongW, SetWindowPos,
        GWL_STYLE, GWL_EXSTYLE,
        WS_EX_WINDOWEDGE, WS_EX_CLIENTEDGE, WS_EX_STATICEDGE, WS_EX_LAYERED,
        WS_CAPTION, WS_THICKFRAME, WS_SYSMENU,
        SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_NOACTIVATE,
        WS_POPUP};
    use winapi::um::dwmapi::{DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DwmEnableBlurBehindWindow};
    use winapi::um::uxtheme::MARGINS;

    if let Ok(hwnd) = window.hwnd() {
        let hwnd = hwnd.0 as HWND;
        unsafe {
            // 设置 WS_POPUP 并显式移除所有可能导致原生按钮的样式
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
}

/// 仅应用 DWM 属性，不调用 SetWindowPos（避免窗口闪烁）
#[cfg(target_os = "windows")]
fn apply_dwm_attributes(window: &tauri::WebviewWindow) {
    use winapi::shared::windef::HWND;
    use winapi::um::dwmapi::{DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DwmEnableBlurBehindWindow};
    use winapi::um::uxtheme::MARGINS;

    if let Ok(hwnd) = window.hwnd() {
        let hwnd = hwnd.0 as HWND;
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
}

fn toggle_window(app_handle: &tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        match window.is_visible() {
            Ok(visible) => {
                println!("窗口当前状态: visible={}", visible);
                if visible {
                    println!("执行隐藏窗口");
                    if let Err(e) = window.hide() {
                        eprintln!("隐藏窗口失败: {}", e);
                    }
                } else {
                    println!("执行显示窗口");
                    #[cfg(target_os = "windows")]
                    apply_borderless_style(&window);
                    if let Err(e) = window.show() {
                        eprintln!("显示窗口失败: {}", e);
                    }
                    #[cfg(target_os = "windows")]
                    apply_dwm_attributes(&window);
                    if let Err(e) = window.set_focus() {
                        eprintln!("设置焦点失败: {}", e);
                    }
                    // 通知前端窗口已就绪，可以安全聚焦输入框
                    let _ = window.emit("window-shown", ());
                }
            }
            Err(e) => {
                eprintln!("窗口状态检查失败: {}", e);
            }
        }
    }
}

fn toggle_autostart(app_handle: &tauri::AppHandle) {
    use tauri_plugin_autostart::ManagerExt;

    let autostart_manager = app_handle.autolaunch();
    match autostart_manager.is_enabled() {
        Ok(enabled) => {
            let result = if enabled {
                println!("禁用开机自启动");
                autostart_manager.disable()
            } else {
                println!("启用开机自启动");
                autostart_manager.enable()
            };

            match result {
                Ok(_) => {
                    let new_enabled = !enabled;
                    let new_label = if new_enabled { "✓ 开机自启动" } else { "开机自启动" };
                    println!("自启动设置已更新: {}", new_label);

                    // 更新托盘菜单
                    if let Some(tray) = app_handle.tray_by_id("tray") {
                        // 创建新的菜单项，使用 expect 而不是 unwrap 避免恐慌
                        let show_item = match MenuItem::with_id(app_handle, "show", "显示搜索", true, None::<&str>) {
                            Ok(item) => item,
                            Err(e) => {
                                eprintln!("创建菜单项失败: {}", e);
                                return;
                            }
                        };
                        let autostart_item = match MenuItem::with_id(app_handle, "autostart", new_label, true, None::<&str>) {
                            Ok(item) => item,
                            Err(e) => {
                                eprintln!("创建菜单项失败: {}", e);
                                return;
                            }
                        };
                        let separator = match PredefinedMenuItem::separator(app_handle) {
                            Ok(item) => item,
                            Err(e) => {
                                eprintln!("创建分隔符失败: {}", e);
                                return;
                            }
                        };
                        let quit_item = match MenuItem::with_id(app_handle, "quit", "退出", true, None::<&str>) {
                            Ok(item) => item,
                            Err(e) => {
                                eprintln!("创建菜单项失败: {}", e);
                                return;
                            }
                        };

                        let menu = match tauri::menu::MenuBuilder::new(app_handle)
                            .item(&show_item)
                            .item(&separator)
                            .item(&autostart_item)
                            .item(&quit_item)
                            .build() {
                                Ok(m) => m,
                                Err(e) => {
                                    eprintln!("构建菜单失败: {}", e);
                                    return;
                                }
                            };
                        let _ = tray.set_menu(Some(menu));
                    }
                }
                Err(e) => {
                    eprintln!("设置自启动失败: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("检查自启动状态失败: {}", e);
        }
    }
}
