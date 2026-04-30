use tauri::Manager;
use tauri::menu::{MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

mod commands;
mod everything;
mod clipboard;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::window::show_window,
            commands::window::hide_window,
            commands::window::center_window,
            commands::app::get_installed_apps,
            commands::app::launch_app,
            commands::app::open_file,
            commands::app::show_in_folder,
            commands::app::get_file_icon_type,
            commands::app::get_app_icon_base64,
            commands::app::get_app_usage,
            commands::app::get_custom_apps,
            commands::app::add_custom_app,
            commands::app::remove_custom_app,
            commands::file::search_files,
            commands::file::get_image_thumbnail,
            commands::clipboard::get_clipboard_history,
            commands::clipboard::delete_clipboard_item,
            commands::clipboard::clear_clipboard_history,
            commands::clipboard::copy_to_clipboard,
            commands::clipboard::copy_image_to_clipboard,
        ])
        .setup(|app| {
            use std::sync::Arc;
            use std::sync::Mutex;
            use std::time::{Duration, Instant};
            use tauri_plugin_autostart::ManagerExt;

            // 启动剪贴板监听
            clipboard::start_clipboard_monitor();

            // 检查是否为开机自启动（通过 --hidden 参数）
            let args: Vec<String> = std::env::args().collect();
            let is_autostart = args.contains(&"--hidden".to_string());

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

            // 使用 Mutex 保护最后触发时间
            let last_trigger: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));

            let app_handle = app.handle().clone();
            let last_trigger_clone = last_trigger.clone();

            // 尝试注册 Alt+Space，如果失败则使用 Alt+`
            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            let result = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
                // 检查时间间隔，防止重复触发
                let now = Instant::now();
                let mut last = last_trigger_clone.lock().unwrap();
                if let Some(last_time) = *last {
                    if now.duration_since(last_time) < Duration::from_millis(500) {
                        println!("忽略重复触发");
                        return;
                    }
                }
                *last = Some(now);
                drop(last);

                println!("Alt+Space 触发");
                toggle_window(&app_handle);
            });

            if result.is_err() {
                // Alt+Space 被占用，回退到 Alt+`
                let fallback = Shortcut::new(Some(Modifiers::ALT), Code::Backquote);
                let app_handle2 = app.handle().clone();
                let last_trigger2 = last_trigger.clone();
                app.global_shortcut().on_shortcut(fallback, move |_app, _shortcut, _event| {
                    let now = Instant::now();
                    let mut last = last_trigger2.lock().unwrap();
                    if let Some(last_time) = *last {
                        if now.duration_since(last_time) < Duration::from_millis(500) {
                            return;
                        }
                    }
                    *last = Some(now);
                    drop(last);

                    println!("Alt+` 触发");
                    toggle_window(&app_handle2);
                })?;
                println!("Alt+Space 已被占用，使用 Alt+` 作为替代");
            }

            // 获取主窗口
            let main_window = app.get_webview_window("main").unwrap();

            // 如果是开机自启动，隐藏窗口
            if is_autostart {
                let _ = main_window.hide();
            }

            // Windows: 使用DWM API移除系统边框
            #[cfg(target_os = "windows")]
            {
                if let Ok(hwnd) = main_window.hwnd() {
                    let hwnd = hwnd.0;
                    
                    unsafe {
                        use winapi::shared::windef::HWND;
                        use winapi::um::winuser::{GetWindowLongW, SetWindowLongW, SetWindowPos,
                            GWL_STYLE, GWL_EXSTYLE, 
                            WS_EX_WINDOWEDGE, WS_EX_CLIENTEDGE, WS_EX_STATICEDGE, WS_EX_LAYERED,
                            SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_NOACTIVATE,
                            WS_POPUP};
                        use winapi::um::dwmapi::{DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DwmEnableBlurBehindWindow};
                        use winapi::um::uxtheme::MARGINS;
                        
                        let hwnd = hwnd as HWND;
                        
                        // 1. 先设置基础样式为 WS_POPUP（无边框窗口）
                        SetWindowLongW(hwnd, GWL_STYLE, WS_POPUP as i32);
                        
                        // 2. 移除所有扩展边框样式
                        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                        SetWindowLongW(hwnd, GWL_EXSTYLE, 
                            (ex_style | WS_EX_LAYERED as i32) & 
                            !(WS_EX_WINDOWEDGE as i32 | WS_EX_CLIENTEDGE as i32 | WS_EX_STATICEDGE as i32));
                        
                        // 3. 将DWM玻璃框架扩展到整个窗口，覆盖掉系统边框
                        let margins = MARGINS { cxLeftWidth: -1, cxRightWidth: -1, cyTopHeight: -1, cyBottomHeight: -1 };
                        DwmExtendFrameIntoClientArea(hwnd, &margins);
                        
                        // 4. Windows 11: 尝试将边框颜色设为透明（属性34 = DWMWA_BORDER_COLOR）
                        let color: u32 = 0xFFFFFFFF; // DWMWA_COLOR_NONE
                        let _ = DwmSetWindowAttribute(hwnd, 34, &color as *const _ as *const _, std::mem::size_of::<u32>() as u32);
                        
                        // 5. Windows 11: 设置圆角（属性33 = DWMWA_WINDOW_CORNER_PREFERENCE）
                        let corner_pref: u32 = 2; // DWMWCP_ROUND
                        let _ = DwmSetWindowAttribute(hwnd, 33, &corner_pref as *const _ as *const _, std::mem::size_of::<u32>() as u32);
                        
                        // 6. 启用模糊背景
                        let bb = winapi::um::dwmapi::DWM_BLURBEHIND {
                            dwFlags: 0x00000001 | 0x00000002, // DWM_BB_ENABLE | DWM_BB_BLURREGION
                            fEnable: 1,
                            hRgnBlur: std::ptr::null_mut(),
                            fTransitionOnMaximized: 0,
                        };
                        let _ = DwmEnableBlurBehindWindow(hwnd, &bb);
                        
                        // 7. 强制窗口重绘边框
                        SetWindowPos(hwnd, std::ptr::null_mut(), 0, 0, 0, 0,
                            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
                    if let Err(e) = window.center() {
                        eprintln!("居中窗口失败: {}", e);
                    }
                    if let Err(e) = window.show() {
                        eprintln!("显示窗口失败: {}", e);
                    }
                    if let Err(e) = window.set_focus() {
                        eprintln!("设置焦点失败: {}", e);
                    }
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
                        let show_item = MenuItem::with_id(app_handle, "show", "显示搜索", true, None::<&str>).unwrap();
                        let autostart_item = MenuItem::with_id(app_handle, "autostart", new_label, true, None::<&str>).unwrap();
                        let separator = PredefinedMenuItem::separator(app_handle).unwrap();
                        let quit_item = MenuItem::with_id(app_handle, "quit", "退出", true, None::<&str>).unwrap();
                        let menu = tauri::menu::MenuBuilder::new(app_handle)
                            .item(&show_item)
                            .item(&separator)
                            .item(&autostart_item)
                            .item(&quit_item)
                            .build().unwrap();
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
