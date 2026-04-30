//! Everything HTTP API 客户端
//! 通过 Everything 内置 HTTP 服务器搜索文件，避免 DLL 权限问题

use serde::Deserialize;
use super::log;
use reqwest::Client;
use std::sync::OnceLock;

const DEFAULT_PORT: u16 = 18080;

fn get_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client")
    })
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub result_type: String,
}

#[derive(Deserialize)]
struct EverythingResponse {
    #[serde(rename = "totalResults")]
    #[allow(dead_code)]
    total_results: u64,
    results: Vec<EverythingResult>,
}

#[derive(Deserialize)]
struct EverythingResult {
    #[serde(rename = "type")]
    result_type: String,
    name: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    size: serde_json::Value,
}

/// 通过 Everything HTTP API 搜索文件（异步）
pub async fn search(query: &str, limit: u32) -> Result<Vec<SearchResult>, String> {
    log::log(&format!("--- HTTP 搜索: '{}' ---", query));

    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let url = format!(
        "http://localhost:{}/?search={}&j=1&path_column=1&size_column=1&count={}",
        DEFAULT_PORT,
        urlencoding::encode(query),
        limit.min(100)
    );

    log::log(&format!("请求: {}", url));

    let resp = match get_client().get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            log::log(&format!("HTTP 请求失败: {}", e));
            return Err(format!("HTTP 请求失败: {}。请确保 Everything HTTP 服务器已启用。", e));
        }
    };

    if !resp.status().is_success() {
        log::log(&format!("HTTP 请求返回错误: {}", resp.status()));
        return Err(format!("HTTP 请求返回错误: {}", resp.status()));
    }

    let body = resp.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    log::log(&format!("响应长度: {} 字节", body.len()));

    let data: EverythingResponse = serde_json::from_str(&body)
        .map_err(|e| {
            let preview: String = body.chars().take(200).collect();
            log::log(&format!("解析 JSON 失败: {}。响应前 200 字符: {}", e, preview));
            format!("解析 JSON 失败: {}", e)
        })?;

    log::log(&format!("找到 {} 个结果", data.total_results));

    let results: Vec<SearchResult> = data
        .results
        .into_iter()
        .map(|r| {
            let size = match &r.size {
                serde_json::Value::String(s) => s.parse::<u64>().unwrap_or(0),
                serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
                _ => 0,
            };
            let full_path = if r.path.is_empty() {
                r.name.clone()
            } else {
                format!("{}\\{}", r.path, r.name)
            };
            SearchResult {
                name: r.name,
                path: full_path,
                size,
                result_type: if r.result_type == "folder" { "folder" } else { "file" }.to_string(),
            }
        })
        .collect();

    log::log(&format!("返回 {} 个结果", results.len()));
    Ok(results)
}
