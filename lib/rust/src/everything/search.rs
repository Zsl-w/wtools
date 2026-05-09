//! Everything 搜索接口

use super::{http, SearchResult};

/// 使用 Everything HTTP API 搜索文件
pub async fn sdk_search(query: &str, limit: u32) -> Result<Vec<SearchResult>, String> {
    http::search(query, limit).await
}
