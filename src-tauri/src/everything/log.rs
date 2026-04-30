//! 日志工具 - 写入文件以便在生产环境诊断问题

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn get_log_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("wtools_debug.log");
        }
    }
    PathBuf::from("wtools_debug.log")
}

pub fn log(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(get_log_path())
    {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
}

