use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_HISTORY: usize = 100;
const MAX_TEXT_PREVIEW: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content_type: String,      // "text", "image", "files", "unknown"
    pub preview: String,           // 预览文本
    pub content: Option<String>,   // 文本内容或图片 base64
    pub timestamp: u64,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct ClipboardHistory {
    items: Vec<ClipboardItem>,
    file_path: PathBuf,
}

impl ClipboardHistory {
    pub fn new() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wtools");
        
        fs::create_dir_all(&data_dir).ok();
        let file_path = data_dir.join("clipboard_history.json");
        
        let items = Self::load_from_file(&file_path).unwrap_or_default();
        
        Self { items, file_path }
    }
    
    fn load_from_file(path: &PathBuf) -> Option<Vec<ClipboardItem>> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }
    
    fn save_to_file(&self) {
        match serde_json::to_string_pretty(&self.items) {
            Ok(content) => {
                if let Err(e) = fs::write(&self.file_path, content) {
                    eprintln!("[clipboard] 保存历史记录失败: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[clipboard] 序列化历史记录失败: {}", e);
            }
        }
    }
    
    pub fn add_text(&mut self, text: String) {
        if text.trim().is_empty() {
            return;
        }
        
        // 检查是否与最近一条重复
        if let Some(last) = self.items.first() {
            if last.content_type == "text" && last.content.as_ref() == Some(&text) {
                return;
            }
        }
        
        let preview = if text.len() > MAX_TEXT_PREVIEW {
            format!("{}...", &text[..MAX_TEXT_PREVIEW])
        } else {
            text.clone()
        };
        
        let item = ClipboardItem {
            id: generate_id(),
            content_type: "text".to_string(),
            preview,
            content: Some(text),
            timestamp: current_timestamp(),
            size: 0,
        };
        
        self.items.insert(0, item);
        self.items.truncate(MAX_HISTORY);
        self.save_to_file();
    }
    
    pub fn add_image(&mut self, image_data: Vec<u8>) {
        if image_data.is_empty() {
            return;
        }
        
        let size = image_data.len();
        let base64 = base64::encode(&image_data);
        
        let item = ClipboardItem {
            id: generate_id(),
            content_type: "image".to_string(),
            preview: format!("图片 ({} KB)", size / 1024),
            content: Some(format!("data:image/png;base64,{}", base64)),
            timestamp: current_timestamp(),
            size,
        };
        
        self.items.insert(0, item);
        self.items.truncate(MAX_HISTORY);
        self.save_to_file();
    }
    
    pub fn add_files(&mut self, files: Vec<String>) {
        if files.is_empty() {
            return;
        }
        
        let preview = if files.len() == 1 {
            files[0].clone()
        } else {
            format!("{} 个文件", files.len())
        };
        
        let item = ClipboardItem {
            id: generate_id(),
            content_type: "files".to_string(),
            preview,
            content: Some(files.join("\n")),
            timestamp: current_timestamp(),
            size: files.len(),
        };
        
        self.items.insert(0, item);
        self.items.truncate(MAX_HISTORY);
        self.save_to_file();
    }
    
    pub fn get_items(&self) -> &[ClipboardItem] {
        &self.items
    }
    
    pub fn remove_item(&mut self, id: &str) {
        self.items.retain(|item| item.id != id);
        self.save_to_file();
    }
    
    pub fn clear(&mut self) {
        self.items.clear();
        self.save_to_file();
    }
}

fn generate_id() -> String {
    format!("{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())
}

fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

// 全局历史记录实例
lazy_static::lazy_static! {
    pub static ref CLIPBOARD_HISTORY: Arc<Mutex<ClipboardHistory>> = Arc::new(Mutex::new(ClipboardHistory::new()));
}
