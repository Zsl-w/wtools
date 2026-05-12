use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use base64::Engine;

/// 应用缓存有效期（5分钟）
const CACHE_DURATION: Duration = Duration::from_secs(300);

/// 应用列表缓存
struct AppCache {
    apps: Vec<AppInfo>,
    last_updated: Instant,
}

/// 全局应用缓存
static APP_CACHE: OnceLock<Mutex<Option<AppCache>>> = OnceLock::new();

/// 获取应用数据目录
fn get_data_dir() -> Result<std::path::PathBuf, String> {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("wtools");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// 验证路径安全性，防止命令注入
fn validate_path(path: &str) -> Result<(), String> {
    let dangerous_chars = [';', '&', '|', '`', '$', '(', ')', '{', '}', '\n', '\r'];
    if path.chars().any(|c| dangerous_chars.contains(&c)) {
        return Err("路径包含非法字符".to_string());
    }
    if path.len() > 260 {
        return Err("路径过长".to_string());
    }
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

    let alias_map: &[(&[&str], &[&str])] = &[
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

    let initials: String = name
        .chars()
        .filter(|c| !c.is_ascii_punctuation() && !c.is_whitespace())
        .enumerate()
        .filter_map(|(_i, c)| {
            if c as u32 >= 0x4E00 && c as u32 <= 0x9FFF {
                None
            } else {
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
    pub aliases: Vec<String>,
}

pub fn get_installed_apps() -> Result<Vec<AppInfo>, String> {
    {
        let cache = APP_CACHE.get_or_init(|| Mutex::new(None)).lock().map_err(|e| e.to_string())?;
        if let Some(ref cached) = *cache {
            if cached.last_updated.elapsed() < CACHE_DURATION {
                return Ok(cached.apps.clone());
            }
        }
    }

    let mut apps = Vec::new();

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

    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let desktop = std::path::Path::new(&userprofile).join("Desktop");
        if desktop.exists() {
            scan_shortcuts(&desktop, &mut apps);
        }
        if let Ok(ref programdata) = std::env::var("PUBLIC") {
            let public_desktop = std::path::Path::new(programdata).join("Desktop");
            if public_desktop.exists() {
                scan_shortcuts(&public_desktop, &mut apps);
            }
        }
    }

    if let Ok(programfiles) = std::env::var("PROGRAMFILES") {
        scan_program_files(&std::path::Path::new(&programfiles), &mut apps);
    }
    if let Ok(programfiles_x86) = std::env::var("PROGRAMFILES(X86)") {
        scan_program_files(&std::path::Path::new(&programfiles_x86), &mut apps);
    }
    if let Ok(windir) = std::env::var("WINDIR") {
        let sys_tools = std::path::Path::new(&windir);
        if sys_tools.exists() {
            scan_system_tools(sys_tools, &mut apps);
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.name.to_lowercase() == b.name.to_lowercase());

    {
        let mut cache = APP_CACHE.get_or_init(|| Mutex::new(None)).lock().map_err(|e| e.to_string())?;
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
    pub last_used: i64,
}

fn record_usage(path: &str) -> Result<(), String> {
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    let data_dir = get_data_dir()?;
    let usage_path = data_dir.join("app_usage_v2.json");

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

pub fn launch_app(path: String) -> Result<(), String> {
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
    let _ = record_usage(&path);
    Ok(())
}

pub fn get_app_usage() -> Result<std::collections::HashMap<String, AppUsage>, String> {
    let data_dir = get_data_dir()?;
    let usage_path = data_dir.join("app_usage_v2.json");

    if !usage_path.exists() {
        let old_path = data_dir.join("app_usage.json");
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
        let mut info: SHFILEINFOW = std::mem::zeroed();
        let result = SHGetFileInfoW(
            wide_path.as_ptr(),
            0,
            &mut info,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            0x100,
        );

        if result == 0 || info.hIcon.is_null() {
            return Ok(None);
        }

        let hicon: HICON = info.hIcon;
        let icon_image = hicon_to_image(hicon);
        DestroyIcon(hicon);

        match icon_image {
            Some(img) => {
                use image::ImageEncoder;

                let resized = image::imageops::resize(&img, 32, 32, image::imageops::FilterType::Lanczos3);

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

                let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
                Ok(Some(format!("data:image/png;base64,{}", b64)))
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

    let mut icon_info: winapi::um::winuser::ICONINFO = std::mem::zeroed();
    if GetIconInfo(hicon, &mut icon_info) == 0 {
        return None;
    }

    let hdc: HDC = CreateCompatibleDC(null_mut());
    if hdc.is_null() {
        DeleteObject(icon_info.hbmColor as *mut _);
        DeleteObject(icon_info.hbmMask as *mut _);
        return None;
    }

    let mut bmi: BITMAPINFO = std::mem::zeroed();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    bmi.bmiHeader.biWidth = 32;
    bmi.bmiHeader.biHeight = -32;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

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

    DeleteDC(hdc);
    DeleteObject(icon_info.hbmColor as *mut _);
    DeleteObject(icon_info.hbmMask as *mut _);

    if res == 0 {
        return None;
    }

    for chunk in pixels.chunks_exact_mut(4) {
        let b = chunk[0];
        chunk[0] = chunk[2];
        chunk[2] = b;
    }

    image::RgbaImage::from_raw(32, 32, pixels)
}

// ── Custom App Management ─────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomApp {
    pub name: String,
    pub path: String,
}

fn read_custom_apps() -> Result<Vec<CustomApp>, String> {
    let data_dir = get_data_dir()?;
    let path = data_dir.join("custom_apps.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn write_custom_apps(apps: &[CustomApp]) -> Result<(), String> {
    let data_dir = get_data_dir()?;
    let path = data_dir.join("custom_apps.json");
    let json = serde_json::to_string_pretty(apps).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

pub fn get_custom_apps() -> Result<Vec<CustomApp>, String> {
    read_custom_apps()
}

pub fn add_custom_app(name: String, path: String) -> Result<(), String> {
    let mut apps = read_custom_apps()?;
    apps.retain(|a| a.path != path);
    apps.push(CustomApp { name, path });
    write_custom_apps(&apps)
}

pub fn remove_custom_app(path: String) -> Result<(), String> {
    let mut apps = read_custom_apps()?;
    apps.retain(|a| a.path != path);
    write_custom_apps(&apps)
}

// ── Autostart ─────────────────────────────────────────────

pub fn is_autostart_enabled() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use winapi::shared::minwindef::HKEY;
        use winapi::um::winreg::RegOpenKeyExW;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        const HKEY_CURRENT_USER: HKEY = 0x80000001isize as HKEY;
        const KEY_READ: u32 = 0x20019;

        let subkey: Vec<u16> = OsStr::new("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .encode_wide().chain(std::iter::once(0)).collect();
        let value: Vec<u16> = OsStr::new("wTools")
            .encode_wide().chain(std::iter::once(0)).collect();

        unsafe {
            let mut hkey: HKEY = std::ptr::null_mut();
            if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
                let mut data_size = 0u32;
                let result = winapi::um::winreg::RegQueryValueExW(
                    hkey, value.as_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), &mut data_size);
                winapi::um::winreg::RegCloseKey(hkey);
                Ok(result == 0)
            } else {
                Ok(false)
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(false)
    }
}

pub fn set_autostart(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use winapi::shared::minwindef::HKEY;
        use winapi::um::winreg::{RegOpenKeyExW, RegSetValueExW, RegDeleteValueW};
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        const HKEY_CURRENT_USER: HKEY = 0x80000001isize as HKEY;
        const KEY_WRITE: u32 = 0x20006;
        const REG_SZ: u32 = 1;

        let subkey: Vec<u16> = OsStr::new("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .encode_wide().chain(std::iter::once(0)).collect();
        let value: Vec<u16> = OsStr::new("wTools")
            .encode_wide().chain(std::iter::once(0)).collect();

        unsafe {
            let mut hkey: HKEY = std::ptr::null_mut();
            if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_WRITE, &mut hkey) != 0 {
                return Err("无法打开注册表".to_string());
            }

            let result = if enabled {
                let exe = std::env::current_exe().map_err(|e| e.to_string())?;
                // 校验 exe 路径是否有效——避免开发构建的缓存路径写入注册表
                let exe_str = exe.to_string_lossy();
                let exe_lower = exe_str.to_lowercase();
                if !exe_lower.ends_with("lib.exe") &&
                   !exe_lower.ends_with("wtools.exe") &&
                   !exe_lower.ends_with(".exe") {
                    return Err("可执行文件路径无效".to_string());
                }
                // 检查路径中是否包含开发构建缓存目录特征
                if exe_lower.contains("build\\windows") || exe_lower.contains("flutter_build") {
                    return Err("检测到开发构建路径，请使用安装版设置自启动".to_string());
                }
                let mut wide_exe: Vec<u16> = OsStr::new(&*exe_str).encode_wide().collect();
                wide_exe.push(0);
                let byte_len = wide_exe.len() * 2;
                RegSetValueExW(hkey, value.as_ptr(), 0, REG_SZ, wide_exe.as_ptr() as *const u8, byte_len as u32)
            } else {
                RegDeleteValueW(hkey, value.as_ptr())
            };

            winapi::um::winreg::RegCloseKey(hkey);

            if result == 0 {
                Ok(())
            } else {
                Err("注册表操作失败".to_string())
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("仅支持 Windows".to_string())
    }
}
