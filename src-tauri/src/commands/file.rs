use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::everything::search as sdk_search;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileResult {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
    #[serde(rename = "type")]
    pub result_type: String, // "file" 或 "folder"
}

// 使用 Everything SDK 搜索文件
#[tauri::command]
pub async fn search_files(query: String, limit: u32) -> Result<Vec<FileResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // 使用 Everything HTTP API 搜索
    match sdk_search::sdk_search(&query, limit).await {
        Ok(results) => Ok(results
            .into_iter()
            .map(|r| FileResult {
                name: r.name,
                path: r.path,
                size: r.size,
                modified: String::new(),
                result_type: r.result_type,
            })
            .collect()),
        Err(e) => {
            // 返回错误信息，让前端能够显示问题
            eprintln!("[search_files] 搜索失败: {}", e);
            Err(format!("文件搜索失败: {}", e))
        }
    }
}

// 获取图片缩略图（返回 base64）
#[tauri::command]
pub fn get_image_thumbnail(path: String) -> Option<String> {
    let file_path = Path::new(&path);
    
    if !file_path.exists() {
        return None;
    }
    
    // 检查扩展名
    let extension = file_path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    
    let image_extensions = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "ico", "svg"];
    if !image_extensions.contains(&extension.as_str()) {
        return None;
    }
    
    // 限制文件大小（最大 5MB）
    let metadata = fs::metadata(&path).ok()?;
    if metadata.len() > 5 * 1024 * 1024 {
        return None;
    }
    
    // 读取图片为 base64
    let bytes = fs::read(&path).ok()?;
    let base64_content = base64::encode(&bytes);
    
    let mime_type = match extension.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "svg" => "image/svg+xml",
        _ => "image/png",
    };
    
    Some(format!("data:{};base64,{}", mime_type, base64_content))
}

