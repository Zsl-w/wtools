use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_HISTORY: usize = 100;
const MAX_TEXT_PREVIEW: usize = 200;
const IMAGE_DIR_NAME: &str = "clipboard_images";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content_type: String,      // "text", "image", "files", "unknown"
    pub preview: String,           // 预览文本
    pub content: Option<String>,   // 文本内容或图片文件路径
    pub timestamp: u64,
    pub size: usize,
    #[serde(default)]
    pub pinned: bool,
}

#[derive(Debug, Clone)]
pub struct ClipboardHistory {
    items: Vec<ClipboardItem>,
    file_path: PathBuf,
    image_dir: PathBuf,
}

impl ClipboardHistory {
    pub fn new() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wtools");

        fs::create_dir_all(&data_dir).ok();
        let file_path = data_dir.join("clipboard_history.json");
        let image_dir = data_dir.join(IMAGE_DIR_NAME);
        fs::create_dir_all(&image_dir).ok();

        let items = Self::load_from_file(&file_path).unwrap_or_default();

        Self { items, file_path, image_dir }
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
            pinned: false,
        };
        
        self.items.insert(0, item);
        self.items.truncate(MAX_HISTORY);
        self.save_to_file();
    }
    
    pub fn add_image(&mut self, image_data: Vec<u8>) {
        if image_data.is_empty() {
            return;
        }

        let id = generate_id();
        let size = image_data.len();

        // 保存图片到文件而非内联 base64
        let image_file = self.image_dir.join(format!("{}.png", id));
        if let Err(e) = fs::write(&image_file, &image_data) {
            eprintln!("[clipboard] 保存图片文件失败: {}", e);
            return;
        }

        let item = ClipboardItem {
            id,
            content_type: "image".to_string(),
            preview: format!("图片 ({} KB)", size / 1024),
            content: Some(image_file.to_string_lossy().to_string()),
            timestamp: current_timestamp(),
            size,
            pinned: false,
        };

        self.items.insert(0, item);
        self.items.truncate(MAX_HISTORY);
        self.cleanup_images();
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
            pinned: false,
        };
        
        self.items.insert(0, item);
        self.items.truncate(MAX_HISTORY);
        self.save_to_file();
    }
    
    pub fn get_items(&self) -> &[ClipboardItem] {
        &self.items
    }
    
    pub fn toggle_pin(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.pinned = !item.pinned;
        }
        // 固定项排在最前面
        self.items.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.timestamp.cmp(&a.timestamp)));
        self.save_to_file();
    }

    pub fn remove_item(&mut self, id: &str) {
        // 删除关联的图片文件
        if let Some(item) = self.items.iter().find(|i| i.id == id) {
            if item.content_type == "image" {
                if let Some(ref path) = item.content {
                    fs::remove_file(path).ok();
                }
            }
        }
        self.items.retain(|item| item.id != id);
        self.save_to_file();
    }

    pub fn clear(&mut self) {
        // 清理所有图片文件
        for item in &self.items {
            if item.content_type == "image" {
                if let Some(ref path) = item.content {
                    fs::remove_file(path).ok();
                }
            }
        }
        self.items.clear();
        self.save_to_file();
    }

    /// 清理不在历史记录中的孤立图片文件
    fn cleanup_images(&self) {
        let existing_ids: std::collections::HashSet<&str> = self.items
            .iter()
            .filter(|i| i.content_type == "image")
            .map(|i| i.id.as_str())
            .collect();

        if let Ok(entries) = fs::read_dir(&self.image_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(id) = name.strip_suffix(".png") {
                    if !existing_ids.contains(id) {
                        fs::remove_file(entry.path()).ok();
                    }
                }
            }
        }
    }
}

fn generate_id() -> String {
    format!("{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())
}

fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

// 全局历史记录实例
pub static CLIPBOARD_HISTORY: OnceLock<Arc<Mutex<ClipboardHistory>>> = OnceLock::new();

/// 获取剪贴板历史实例
pub fn get_clipboard_history() -> &'static Arc<Mutex<ClipboardHistory>> {
    CLIPBOARD_HISTORY.get_or_init(|| Arc::new(Mutex::new(ClipboardHistory::new())))
}
