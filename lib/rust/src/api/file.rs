use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::Path;
use base64::Engine;

use crate::everything::search as sdk_search;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileResult {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
    #[serde(rename = "type")]
    pub result_type: String,
}

/// 使用 Everything HTTP API 搜索文件
pub async fn search_files(query: String, limit: u32) -> Result<Vec<FileResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    match sdk_search::sdk_search(&query, limit).await {
        Ok(results) => Ok(results
            .into_iter()
            .map(|r| FileResult {
                name: r.name,
                path: r.path,
                size: r.size,
                modified: r.modified,
                result_type: r.result_type,
            })
            .collect()),
        Err(e) => {
            eprintln!("[search_files] 搜索失败: {}", e);
            Err(format!("文件搜索失败: {}", e))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilePreviewResult {
    pub preview_type: String,
    pub content: Option<String>,
    pub total_lines: Option<usize>,
    pub size: u64,
    pub modified: String,
    pub extension: String,
}

/// 获取文件预览
pub fn get_file_preview(path: String) -> Option<FilePreviewResult> {
    let file_path = Path::new(&path);
    let metadata = fs::metadata(&file_path).ok()?;
    let size = metadata.len();

    let modified = metadata.modified().ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            let secs = d.as_secs() as i64;
            let secs_adj = secs + 8 * 3600;
            let days = secs_adj / 86400;
            let time_of_day = secs_adj % 86400;
            let h = time_of_day / 3600;
            let m = (time_of_day % 3600) / 60;
            let s = time_of_day % 60;

            let mut y = 1970i64;
            let mut remaining_days = days;
            loop {
                let days_in_year = if is_leap_year(y) { 366 } else { 365 };
                if remaining_days < days_in_year { break; }
                remaining_days -= days_in_year;
                y += 1;
            }
            let leap = is_leap_year(y);
            let days_in_month = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut mo = 0usize;
            for (i, &dim) in days_in_month.iter().enumerate() {
                if remaining_days < dim { mo = i; break; }
                remaining_days -= dim;
            }
            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo + 1, remaining_days + 1, h, m, s)
        })
        .unwrap_or_default();

    let extension = file_path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let image_extensions = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "ico", "svg"];
    if image_extensions.contains(&extension.as_str()) {
        return Some(FilePreviewResult {
            preview_type: "image".to_string(),
            content: None,
            total_lines: None,
            size,
            modified,
            extension,
        });
    }

    let preview_handler_exts = ["pdf", "docx", "xlsx", "pptx"];
    if preview_handler_exts.contains(&extension.as_str()) {
        let doc_result = match extension.as_str() {
            "pdf" => extract_pdf_text(&path),
            "docx" => extract_docx_text(&path),
            "xlsx" => extract_xlsx_text(&path),
            "pptx" => extract_pptx_text(&path),
            _ => None,
        };
        if let Some(text) = doc_result {
            let total_lines = text.lines().count();
            let preview: Vec<&str> = text.lines().take(80).collect();
            return Some(FilePreviewResult {
                preview_type: "text".to_string(),
                content: Some(preview.join("\n")),
                total_lines: Some(total_lines),
                size,
                modified,
                extension,
            });
        }
    }

    let text_extensions = [
        "txt", "md", "markdown", "json", "xml", "yaml", "yml", "toml", "ini", "cfg", "conf",
        "js", "ts", "jsx", "tsx", "vue", "svelte",
        "rs", "py", "rb", "go", "java", "c", "cpp", "h", "hpp", "cs", "swift", "kt",
        "html", "htm", "css", "scss", "sass", "less",
        "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd",
        "sql", "graphql", "proto",
        "log", "csv", "env", "gitignore", "dockerignore",
        "makefile", "cmake", "gradle",
        "lock", "editorconfig", "prettierrc", "eslintrc",
    ];

    if text_extensions.contains(&extension.as_str()) || is_likely_text(&path) {
        if size > 2 * 1024 * 1024 {
            return Some(FilePreviewResult {
                preview_type: "binary".to_string(),
                content: None,
                total_lines: None,
                size,
                modified,
                extension,
            });
        }

        if let Ok(content) = fs::read_to_string(&file_path) {
            let total_lines = content.lines().count();
            let preview_lines: Vec<&str> = content.lines().take(80).collect();
            return Some(FilePreviewResult {
                preview_type: "text".to_string(),
                content: Some(preview_lines.join("\n")),
                total_lines: Some(total_lines),
                size,
                modified,
                extension,
            });
        }
    }

    Some(FilePreviewResult {
        preview_type: "binary".to_string(),
        content: None,
        total_lines: None,
        size,
        modified,
        extension,
    })
}

/// 获取图片缩略图（返回 base64）
pub fn get_image_thumbnail(path: String) -> Option<String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return None;
    }

    let extension = file_path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let image_extensions = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "ico", "svg"];
    if !image_extensions.contains(&extension.as_str()) {
        return None;
    }

    let metadata = fs::metadata(&path).ok()?;
    if metadata.len() > 5 * 1024 * 1024 {
        return None;
    }

    let bytes = fs::read(&path).ok()?;
    let base64_content = base64::engine::general_purpose::STANDARD.encode(&bytes);

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

// ── Helpers ──────────────────────────────────────────────

fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn is_likely_text(path: &str) -> bool {
    let Ok(bytes) = fs::read(path) else { return false };
    let sample = &bytes[..bytes.len().min(512)];
    let null_count = sample.iter().filter(|&&b| b == 0).count();
    null_count < sample.len() / 10
}

fn extract_pdf_text(path: &str) -> Option<String> {
    use lopdf::Document;

    let doc = Document::load(path).ok()?;
    let pages = doc.get_pages();
    let mut pages_text = Vec::new();
    let max_pages = 10usize;

    for (i, (&_page_num, &page_id)) in pages.iter().enumerate() {
        if i >= max_pages { break; }
        if let Ok(content) = doc.get_page_content(page_id) {
            let text = extract_text_from_stream(&content);
            if !text.trim().is_empty() {
                pages_text.push(format!("── 第 {} 页 ──\n{}", i + 1, text));
            }
        }
    }

    if pages_text.is_empty() {
        None
    } else {
        Some(pages_text.join("\n\n"))
    }
}

fn extract_text_from_stream(content: &[u8]) -> String {
    let mut result = String::new();
    let mut in_text = false;

    let text = String::from_utf8_lossy(content);
    let mut chars = text.chars().peekable();
    let mut current_token = String::new();

    while let Some(c) = chars.next() {
        match c {
            'B' => {
                current_token.push(c);
                if current_token.ends_with("BT") {
                    in_text = true;
                    current_token.clear();
                    result.push('\n');
                }
            }
            'E' => {
                current_token.push(c);
                if current_token.ends_with("ET") {
                    in_text = false;
                    current_token.clear();
                }
            }
            '(' if in_text => {
                let mut depth = 1i32;
                let mut text_buf = String::new();
                let mut escaped = false;
                while let Some(tc) = chars.next() {
                    if escaped {
                        match tc {
                            'n' => text_buf.push('\n'),
                            'r' => text_buf.push('\r'),
                            't' => text_buf.push('\t'),
                            '\\' => text_buf.push('\\'),
                            '(' => text_buf.push('('),
                            ')' => text_buf.push(')'),
                            _ => text_buf.push(tc),
                        }
                        escaped = false;
                    } else if tc == '\\' {
                        escaped = true;
                    } else if tc == '(' {
                        depth += 1;
                        text_buf.push(tc);
                    } else if tc == ')' {
                        depth -= 1;
                        if depth <= 0 { break; }
                        text_buf.push(tc);
                    } else {
                        text_buf.push(tc);
                    }
                }
                result.push_str(&text_buf);
            }
            '[' if in_text => {
                let mut depth = 1i32;
                while let Some(tc) = chars.next() {
                    if tc == '[' { depth += 1; }
                    else if tc == ']' { depth -= 1; if depth <= 0 { break; } }
                    else if tc == '(' {
                        let mut text_buf = String::new();
                        let mut escaped = false;
                        while let Some(sc) = chars.next() {
                            if escaped {
                                escaped = false;
                                text_buf.push(sc);
                            } else if sc == '\\' {
                                escaped = true;
                            } else if sc == ')' {
                                break;
                            } else {
                                text_buf.push(sc);
                            }
                        }
                        result.push_str(&text_buf);
                    }
                }
            }
            'T' => {
                current_token.push(c);
                if current_token.ends_with("T*") || current_token.ends_with("Td") || current_token.ends_with("TD") {
                    result.push('\n');
                    current_token.clear();
                }
            }
            _ if !c.is_ascii_digit() && c != ' ' && c != '\n' && c != '\r' && c != '[' && c != ']' => {
                if in_text {
                    current_token.push(c);
                }
            }
            _ => {
                current_token.push(c);
            }
        }
        if current_token.len() > 20 {
            current_token.clear();
        }
    }

    result.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_docx_text(path: &str) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let mut xml_content = String::new();
    if let Ok(mut f) = archive.by_name("word/document.xml") {
        f.read_to_string(&mut xml_content).ok()?;
    } else {
        return None;
    }

    extract_xml_text_content(&xml_content, "w:t")
}

fn extract_xlsx_text(path: &str) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let shared_strings = {
        let mut xml = String::new();
        if let Ok(mut f) = archive.by_name("xl/sharedStrings.xml") {
            f.read_to_string(&mut xml).ok()?;
        }
        parse_shared_strings(&xml)
    };

    let mut sheets = Vec::new();
    let sheet_names: Vec<String> = archive.file_names()
        .filter(|n| n.starts_with("xl/worksheets/sheet") && n.ends_with(".xml"))
        .map(|n| n.to_string())
        .collect();

    for sheet_name in &sheet_names {
        let mut xml = String::new();
        if let Ok(mut f) = archive.by_name(sheet_name) {
            f.read_to_string(&mut xml).ok()?;
        }
        if let Some(sheet_text) = parse_xlsx_sheet(&xml, &shared_strings) {
            sheets.push(sheet_text);
        }
    }

    if sheets.is_empty() {
        None
    } else {
        Some(sheets.join("\n\n"))
    }
}

fn parse_shared_strings(xml: &str) -> Vec<String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    let mut strings = Vec::new();
    let mut current = String::new();
    let mut in_si = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"si" => {
                in_si = true;
                current.clear();
            }
            Ok(Event::Text(ref e)) if in_si => {
                current.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"si" => {
                strings.push(current.clone());
                in_si = false;
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }
    strings
}

fn parse_xlsx_sheet(xml: &str, shared_strings: &[String]) -> Option<String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut cell_value = String::new();
    let mut in_value = false;
    let mut cell_type_is_shared = false;
    let mut in_row = false;
    let mut row_count = 0usize;
    let max_rows = 50;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"row" => {
                in_row = true;
                current_row.clear();
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"c" => {
                cell_type_is_shared = e.attributes()
                    .filter_map(|a| a.ok())
                    .any(|a| a.key.as_ref() == b"t" && a.value.as_ref() == b"s");
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"v" && in_row => {
                in_value = true;
                cell_value.clear();
            }
            Ok(Event::Text(ref e)) if in_value => {
                cell_value.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"v" => {
                in_value = false;
                if cell_type_is_shared {
                    if let Ok(idx) = cell_value.parse::<usize>() {
                        if idx < shared_strings.len() {
                            current_row.push(shared_strings[idx].clone());
                        } else {
                            current_row.push(cell_value.clone());
                        }
                    } else {
                        current_row.push(cell_value.clone());
                    }
                } else {
                    current_row.push(cell_value.clone());
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"row" && in_row => {
                in_row = false;
                if !current_row.is_empty() {
                    rows.push(current_row.join("\t"));
                    row_count += 1;
                    if row_count >= max_rows {
                        rows.push("... (更多行省略)".to_string());
                        break;
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }

    if rows.is_empty() {
        None
    } else {
        Some(rows.join("\n"))
    }
}

fn extract_pptx_text(path: &str) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let slide_names: Vec<String> = archive.file_names()
        .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
        .map(|n| n.to_string())
        .collect();

    let mut slides = Vec::new();
    for (i, name) in slide_names.iter().enumerate() {
        let mut xml = String::new();
        if let Ok(mut f) = archive.by_name(name) {
            f.read_to_string(&mut xml).ok()?;
        }
        if let Some(text) = extract_xml_text_content(&xml, "a:t") {
            if !text.trim().is_empty() {
                slides.push(format!("── 幻灯片 {} ──\n{}", i + 1, text));
            }
        }
    }

    if slides.is_empty() {
        None
    } else {
        Some(slides.join("\n\n"))
    }
}

fn extract_xml_text_content(xml: &str, tag: &str) -> Option<String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    let mut texts = Vec::new();
    let mut current = String::new();
    let mut capturing = false;
    let tag_bytes = tag.as_bytes();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == tag_bytes => {
                capturing = true;
                current.clear();
            }
            Ok(Event::Text(ref e)) if capturing => {
                current.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == tag_bytes => {
                if capturing {
                    let trimmed = current.trim();
                    if !trimmed.is_empty() {
                        texts.push(trimmed.to_string());
                    }
                    capturing = false;
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }

    if texts.is_empty() {
        None
    } else {
        Some(texts.join(""))
    }
}
