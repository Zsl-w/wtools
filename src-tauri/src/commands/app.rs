use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;

/// 应用缓存有效期（5分钟）
const CACHE_DURATION: Duration = Duration::from_secs(300);

/// 应用列表缓存
struct AppCache {
    apps: Vec<AppInfo>,
    last_updated: Instant,
}

lazy_static! {
    static ref APP_CACHE: Mutex<Option<AppCache>> = Mutex::new(None);
}

/// 验证路径安全性，防止命令注入
fn validate_path(path: &str) -> Result<(), String> {
    // 检查路径是否包含危险字符
    let dangerous_chars = [';', '&', '|', '`', '$', '(', ')', '{', '}', '\n', '\r'];
    if path.chars().any(|c| dangerous_chars.contains(&c)) {
        return Err("路径包含非法字符".to_string());
    }

    // 检查路径长度
    if path.len() > 260 {
        return Err("路径过长".to_string());
    }

    // 验证路径是有效的文件系统路径
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err("路径不存在".to_string());
    }

    Ok(())
}

/// 获取应用别名（拼音/首字母）
fn get_app_aliases(name: &str) -> Vec<String> {
    let name_lower = name.to_lowercase();
    let mut aliases = Vec::new();
    
    // 常见应用别名映射
    let alias_map: &[(&[&str], &[&str])] = &[
        // (匹配名, 别名列表)
        (&["微信", "wechat"], &["weixin", "wx"]),
        (&["qq"], &["qq"]),
        (&["钉钉", "dingtalk"], &["dingding", "dd"]),
        (&["企业微信"], &["qiyeweixin", "qywx", "qiye", "qy"]),
        (&["飞书", "lark"], &["feishu", "fs"]),
        (&["wps"], &["wps"]),
        (&["word"], &["word", "wd"]),
        (&["excel"], &["excel", "xls"]),
        (&["powerpoint", "ppt"], &["ppt", "powerpoint"]),
        (&["记事本", "notepad"], &["jishiben", "jsb", "notepad"]),
        (&["计算器", "calculator"], &["jisuanqi", "jsq", "calc"]),
        (&["画图", "paint", "mspaint"], &["huatu", "ht", "paint"]),
        (&["浏览器", "chrome", "edge", "firefox"], &["liulanqi", "llq", "browser"]),
        (&["谷歌浏览器", "chrome"], &["chrome", "gg"]),
        (&["火狐", "firefox"], &["firefox", "huohu", "hh"]),
        (&["微软Edge", "microsoft edge", "edge"], &["edge", "llq"]),
        (&["vs code", "vscode", "code"], &["vscode", "code", "vs"]),
        (&["idea", "intellij"], &["idea", "ij"]),
        (&["终端", "terminal", "cmd", "命令提示符"], &["zhongduan", "zd", "cmd", "terminal"]),
        (&["控制面板", "control panel"], &["kongzhi", "kz", "control"]),
        (&["任务管理器", "task manager"], &["renwu", "rw", "taskmgr"]),
        (&["资源管理器", "explorer"], &["ziyuan", "zy", "explorer"]),
        (&["设置", "settings"], &["shezhi", "sz", "settings"]),
        (&["照片", "photos"], &["zhaopian", "zp", "photos"]),
        (&["音乐", "music", "groove"], &["yinle", "yl", "music"]),
        (&["视频", "video"], &["shipin", "sp", "video"]),
        (&["网易云音乐", "netease music"], &["wangyiyun", "wyy", "cloudmusic"]),
        (&["qq音乐", "qqmusic"], &["qqyinyue", "qqyy"]),
        (&["酷狗", "kugou"], &["kugou", "kg"]),
        (&["迅雷", "thunder"], &["xunlei", "xl", "thunder"]),
        (&["百度网盘", "baidunetdisk"], &["baidu", "bd", "baiduyun", "bdy"]),
        (&["阿里云盘", "aliyundrive"], &["aliyun", "aly", "aliyundrive"]),
        (&["腾讯会议", "tencent meeting"], &["tengxun", "tx", "wemeet"]),
        (&["zoom"], &["zoom", "zm"]),
        (&["steam"], &["steam"]),
        (&["wegame"], &["wegame", "wg"]),
        (&["爱奇艺", "iqiyi"], &["aiqiyi", "aqy", "iqiyi"]),
        (&["优酷", "youku"], &["youku", "yk"]),
        (&["哔哩哔哩", "bilibili", "b站"], &["bilibili", "blbl", "bili", "bzhan"]),
        (&["腾讯视频", "tencent video"], &["tengxun", "tx", "txsp"]),
        (&["抖音", "douyin", "tiktok"], &["douyin", "dy"]),
        (&["淘宝", "taobao"], &["taobao", "tb"]),
        (&["京东", "jd"], &["jingdong", "jd"]),
        (&["拼多多", "pdd"], &["pinduoduo", "pdd"]),
        (&["美团", "meituan"], &["meituan", "mt"]),
    ];
    
    for (names, alias_list) in alias_map {
        if names.iter().any(|n| name_lower.contains(&n.to_lowercase())) {
            for alias in *alias_list {
                if !aliases.contains(&alias.to_string()) {
                    aliases.push(alias.to_string());
                }
            }
        }
    }
    
    // 为所有应用生成首字母缩写（中文取每个字首字母，英文取每个单词首字母）
    let initials: String = name
        .chars()
        .filter(|c| !c.is_ascii_punctuation() && !c.is_whitespace())
        .enumerate()
        .filter_map(|(_i, c)| {
            // 如果是中文字符，且是词语开头（前一个字是英文/数字，或这是第一个字符）
            if c as u32 >= 0x4E00 && c as u32 <= 0x9FFF {
                // 简化为：每个汉字都取拼音首字母（这里用字母本身，因为已经映射过常见应用）
                // 对于未知中文，不提供首字母，依赖上面的映射
                None
            } else {
                // 英文字母/数字，取小写
                Some(c.to_ascii_lowercase())
            }
        })
        .collect();
    
    if initials.len() >= 2 && initials.len() <= 6 && !aliases.contains(&initials) {
        aliases.push(initials);
    }
    
    aliases
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppInfo {
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub aliases: Vec<String>, // 拼音/首字母别名，如 ["weixin", "wx"]
}

#[tauri::command]
pub fn get_installed_apps() -> Result<Vec<AppInfo>, String> {
    // 检查缓存是否有效
    {
        let cache = APP_CACHE.lock().map_err(|e| e.to_string())?;
        if let Some(ref cached) = *cache {
            if cached.last_updated.elapsed() < CACHE_DURATION {
                return Ok(cached.apps.clone());
            }
        }
    }

    // 缓存过期或不存在，重新扫描
    let mut apps = Vec::new();

    // 1. 扫描用户开始菜单
    if let Ok(appdata) = std::env::var("APPDATA") {
        let user_start_menu = std::path::Path::new(&appdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        if user_start_menu.exists() {
            scan_shortcuts(&user_start_menu, &mut apps);
        }
    }

    // 2. 扫描系统开始菜单
    if let Ok(programdata) = std::env::var("PROGRAMDATA") {
        let sys_start_menu = std::path::Path::new(&programdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        if sys_start_menu.exists() {
            scan_shortcuts(&sys_start_menu, &mut apps);
        }
    }

    // 3. 扫描桌面
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let desktop = std::path::Path::new(&userprofile).join("Desktop");
        if desktop.exists() {
            scan_shortcuts(&desktop, &mut apps);
        }
        // 公共桌面
        if let Ok(ref programdata) = std::env::var("PUBLIC") {
            let public_desktop = std::path::Path::new(programdata).join("Desktop");
            if public_desktop.exists() {
                scan_shortcuts(&public_desktop, &mut apps);
            }
        }
    }

    // 4. 扫描常见安装目录
    if let Ok(programfiles) = std::env::var("PROGRAMFILES") {
        scan_program_files(&std::path::Path::new(&programfiles), &mut apps);
    }
    if let Ok(programfiles_x86) = std::env::var("PROGRAMFILES(X86)") {
        scan_program_files(&std::path::Path::new(&programfiles_x86), &mut apps);
    }
    // 扫描 Windows 系统工具
    if let Ok(windir) = std::env::var("WINDIR") {
        let sys_tools = std::path::Path::new(&windir);
        if sys_tools.exists() {
            scan_system_tools(sys_tools, &mut apps);
        }
    }

    // 去重
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.name.to_lowercase() == b.name.to_lowercase());

    // 更新缓存
    {
        let mut cache = APP_CACHE.lock().map_err(|e| e.to_string())?;
        *cache = Some(AppCache {
            apps: apps.clone(),
            last_updated: Instant::now(),
        });
    }

    Ok(apps)
}

fn scan_program_files(dir: &std::path::Path, apps: &mut Vec<AppInfo>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                scan_exe_in_dir(&path, apps, 2);
            }
        }
    }
}

fn scan_exe_in_dir(dir: &std::path::Path, apps: &mut Vec<AppInfo>, depth: usize) {
    if depth == 0 {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()).take(50) {
            let path = entry.path();
            if path.is_dir() {
                scan_exe_in_dir(&path, apps, depth - 1);
            } else if let Some(ext) = path.extension() {
                if ext == "exe" {
                    if let Some(name) = path.file_stem() {
                        let name_str = name.to_string_lossy().to_string();
                        if !is_system_exe(&name_str) {
                            let aliases = get_app_aliases(&name_str);
                            apps.push(AppInfo {
                                name: name_str,
                                path: path.to_string_lossy().to_string(),
                                icon: None,
                                aliases,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn scan_system_tools(windir: &std::path::Path, apps: &mut Vec<AppInfo>) {
    let tools = vec![
        ("记事本", "notepad.exe"),
        ("计算器", "System32\\calc.exe"),
        ("命令提示符", "System32\\cmd.exe"),
        ("资源管理器", "explorer.exe"),
        ("任务管理器", "System32\\taskmgr.exe"),
        ("画图", "System32\\mspaint.exe"),
        ("截图工具", "System32\\SnippingTool.exe"),
    ];
    for (name, exe) in tools {
        let path = windir.join(exe);
        if path.exists() {
            let aliases = get_app_aliases(name);
            apps.push(AppInfo {
                name: name.to_string(),
                path: path.to_string_lossy().to_string(),
                icon: None,
                aliases,
            });
        }
    }
}

fn is_system_exe(name: &str) -> bool {
    let system_names = [
        "setup", "install", "uninstall", "update", "crashpad",
        "helper", "renderer", "gpu-process", "plugin-host"
    ];
    let lower = name.to_lowercase();
    system_names.iter().any(|&s| lower.contains(s))
}

fn scan_shortcuts(dir: &std::path::Path, apps: &mut Vec<AppInfo>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                scan_shortcuts(&path, apps);
            } else if let Some(ext) = path.extension() {
                if ext == "lnk" || ext == "exe" {
                    if let Some(name) = path.file_stem() {
                        let name_str = name.to_string_lossy().to_string();
                        if !name_str.contains("卸载") && !name_str.contains("Uninstall") {
                            let aliases = get_app_aliases(&name_str);
                            apps.push(AppInfo {
                                name: name_str,
                                path: path.to_string_lossy().to_string(),
                                icon: None,
                                aliases,
                            });
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppUsage {
    pub count: u32,
    pub last_used: i64, // Unix timestamp
}

#[tauri::command]
fn get_usage_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    use tauri::Manager;
    let app_dir = app.path().app_data_dir().map_err(|e: tauri::Error| e.to_string())?;
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    Ok(app_dir.join("app_usage_v2.json"))
}

fn record_usage_internal(app: &tauri::AppHandle, path: &str) -> Result<(), String> {
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let usage_path = get_usage_path(app)?;
    let mut usage: HashMap<String, AppUsage> = if usage_path.exists() {
        let content = std::fs::read_to_string(&usage_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    let entry = usage.entry(path.to_string()).or_default();
    entry.count += 1;
    entry.last_used = now;
    
    let json = serde_json::to_string_pretty(&usage).map_err(|e| e.to_string())?;
    std::fs::write(usage_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn launch_app(app: tauri::AppHandle, path: String) -> Result<(), String> {
    validate_path(&path)?;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        Command::new("cmd")
            .args(["/C", "start", "", &path])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    // 记录使用
    let _ = record_usage_internal(&app, &path);
    Ok(())
}

#[tauri::command]
pub fn get_app_usage(app: tauri::AppHandle) -> Result<std::collections::HashMap<String, AppUsage>, String> {
    use tauri::Manager;
    let usage_path = get_usage_path(&app)?;
    if !usage_path.exists() {
        // 尝试从旧版本迁移
        let app_dir: std::path::PathBuf = app.path().app_data_dir().map_err(|e: tauri::Error| e.to_string())?;
        let old_path = app_dir.join("app_usage.json");
        if old_path.exists() {
            let content = std::fs::read_to_string(&old_path).map_err(|e| e.to_string())?;
            let old_usage: std::collections::HashMap<String, u32> =
                serde_json::from_str(&content).unwrap_or_default();
            let migrated: std::collections::HashMap<String, AppUsage> = old_usage.into_iter()
                .map(|(k, v)| (k, AppUsage { count: v, last_used: 0 }))
                .collect();
            return Ok(migrated);
        }
        return Ok(std::collections::HashMap::new());
    }
    let content = std::fs::read_to_string(&usage_path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    validate_path(&path)?;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        Command::new("explorer")
            .arg(&path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_in_folder(path: String) -> Result<(), String> {
    validate_path(&path)?;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// 获取应用图标 - 使用 image crate 直接处理
#[tauri::command]
pub fn get_app_icon_base64(path: String) -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        extract_icon(&path)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn extract_icon(path: &str) -> Result<Option<String>, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::shellapi::SHGetFileInfoW;
    use winapi::um::shellapi::SHFILEINFOW;
    use winapi::um::winuser::DestroyIcon;
    use winapi::shared::windef::HICON;
    
    let wide_path: Vec<u16> = OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();
    
    unsafe {
        // 获取系统图标
        let mut info: SHFILEINFOW = std::mem::zeroed();
        let result = SHGetFileInfoW(
            wide_path.as_ptr(),
            0,
            &mut info,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            0x100, // SHGFI_ICON
        );
        
        if result == 0 || info.hIcon.is_null() {
            return Ok(None);
        }
        
        let hicon: HICON = info.hIcon;
        
        // 使用 image crate 从 HICON 创建图像
        let icon_image = hicon_to_image(hicon);
        DestroyIcon(hicon);
        
        match icon_image {
                Some(img) => {
                    use image::ImageEncoder;
                    
                    // 缩放到 32x32
                    let resized = image::imageops::resize(&img, 32, 32, image::imageops::FilterType::Lanczos3);
                    
                    // 编码为 PNG
                    let mut png_data: Vec<u8> = Vec::new();
                    {
                        use image::codecs::png::PngEncoder;
                        let encoder = PngEncoder::new(&mut png_data);
                        let _ = encoder.write_image(
                            resized.as_raw(),
                            32,
                            32,
                            image::ColorType::Rgba8
                        );
                    }
                
                if png_data.len() < 100 {
                    return Ok(None);
                }
                
                let b64 = base64::encode(&png_data);
                Ok(Some(format!("data:image/png;base64,{}" , b64)))
            }
            None => Ok(None),
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn hicon_to_image(hicon: winapi::shared::windef::HICON) -> Option<image::RgbaImage> {
    use winapi::um::winuser::GetIconInfo;
    use winapi::um::wingdi::{GetDIBits, CreateCompatibleDC, DeleteDC, DeleteObject};
    use winapi::um::wingdi::{BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, BI_RGB};
    use winapi::shared::windef::HDC;
    use winapi::shared::minwindef::DWORD;
    use winapi::ctypes::c_void;
    use std::ptr::null_mut;
    
    // 获取图标信息
    let mut icon_info: winapi::um::winuser::ICONINFO = std::mem::zeroed();
    if GetIconInfo(hicon, &mut icon_info) == 0 {
        return None;
    }
    
    // 获取位图信息
    let hdc: HDC = CreateCompatibleDC(null_mut());
    if hdc.is_null() {
        DeleteObject(icon_info.hbmColor as *mut _);
        DeleteObject(icon_info.hbmMask as *mut _);
        return None;
    }
    
    // 设置 BITMAPINFO
    let mut bmi: BITMAPINFO = std::mem::zeroed();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    bmi.bmiHeader.biWidth = 32;
    bmi.bmiHeader.biHeight = -32;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;
    
    // 读取颜色位图
    let mut pixels: Vec<u8> = vec![0; 32 * 32 * 4];
    let res = GetDIBits(
        hdc,
        icon_info.hbmColor,
        0,
        32,
        pixels.as_mut_ptr() as *mut c_void,
        &mut bmi,
        DIB_RGB_COLORS,
    );
    
    // 清理
    DeleteDC(hdc);
    DeleteObject(icon_info.hbmColor as *mut _);
    DeleteObject(icon_info.hbmMask as *mut _);
    
    if res == 0 {
        return None;
    }
    
    // BGRA -> RGBA
    for chunk in pixels.chunks_exact_mut(4) {
        let b = chunk[0];
        chunk[0] = chunk[2];
        chunk[2] = b;
    }
    
    // 创建 RgbaImage
    image::RgbaImage::from_raw(32, 32, pixels)
}

// 自定义应用管理
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomApp {
    pub name: String,
    pub path: String,
}

fn get_custom_apps_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    let app_dir = app.path().app_data_dir().map_err(|e: tauri::Error| e.to_string())?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    Ok(app_dir.join("custom_apps.json"))
}

fn read_custom_apps(app: &tauri::AppHandle) -> Result<Vec<CustomApp>, String> {
    let path = get_custom_apps_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn write_custom_apps(app: &tauri::AppHandle, apps: &[CustomApp]) -> Result<(), String> {
    let path = get_custom_apps_path(app)?;
    let json = serde_json::to_string_pretty(apps).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_custom_apps(app: tauri::AppHandle) -> Result<Vec<CustomApp>, String> {
    read_custom_apps(&app)
}

#[tauri::command]
pub fn add_custom_app(app: tauri::AppHandle, name: String, path: String) -> Result<(), String> {
    let mut apps = read_custom_apps(&app)?;
    apps.retain(|a| a.path != path);
    apps.push(CustomApp { name, path });
    write_custom_apps(&app, &apps)
}

#[tauri::command]
pub fn remove_custom_app(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let mut apps = read_custom_apps(&app)?;
    apps.retain(|a| a.path != path);
    write_custom_apps(&app, &apps)
}

// 简化版：根据扩展名返回emoji
#[tauri::command]
pub fn get_file_icon_type(path: String) -> Result<String, String> {
    let path_lower = path.to_lowercase();
    if path_lower.ends_with(".exe") || path_lower.ends_with(".lnk") {
        Ok("app".to_string())
    } else if path_lower.ends_with(".doc") || path_lower.ends_with(".docx") {
        Ok("doc".to_string())
    } else if path_lower.ends_with(".xls") || path_lower.ends_with(".xlsx") {
        Ok("xls".to_string())
    } else if path_lower.ends_with(".ppt") || path_lower.ends_with(".pptx") {
        Ok("ppt".to_string())
    } else if path_lower.ends_with(".pdf") {
        Ok("pdf".to_string())
    } else if path_lower.ends_with(".zip") || path_lower.ends_with(".rar") || path_lower.ends_with(".7z") {
        Ok("zip".to_string())
    } else if path_lower.ends_with(".jpg") || path_lower.ends_with(".png") || path_lower.ends_with(".gif") {
        Ok("image".to_string())
    } else if path_lower.ends_with(".mp3") || path_lower.ends_with(".mp4") || path_lower.ends_with(".avi") {
        Ok("media".to_string())
    } else {
        Ok("file".to_string())
    }
}