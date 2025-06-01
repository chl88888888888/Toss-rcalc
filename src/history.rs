use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

// 历史记录条目
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: f64,
    pub timestamp: String, // 使用字符串存储时间戳，便于序列化
}

// 历史记录管理
pub struct HistoryManager {
    file_path: String,
    max_entries: usize,
}

impl HistoryManager {
    // 创建新的历史管理器
    pub fn new(file_path: &str, max_entries: usize) -> Self {
        // 确保目录存在
        if let Some(parent) = Path::new(file_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create history directory");
            }
        }
        
        HistoryManager {
            file_path: file_path.to_string(),
            max_entries,
        }
    }

    // 添加历史记录
    pub fn add_entry(&self, entry: HistoryEntry) -> io::Result<()> {
        let mut history = self.load_history().unwrap_or_default();
        
        // 添加新条目
        history.push(entry);
        
        // 保持最多 max_entries 条记录
        if history.len() > self.max_entries {
            history.remove(0);
        }
        
        self.save_history(&history)
    }

    // 获取所有历史记录
    pub fn get_history(&self) -> io::Result<Vec<HistoryEntry>> {
        self.load_history()
    }

    // 清空历史记录
    pub fn clear_history(&self) -> io::Result<()> {
        self.save_history(&Vec::new())
    }

    // 加载历史记录
    fn load_history(&self) -> io::Result<Vec<HistoryEntry>> {
        if !Path::new(&self.file_path).exists() {
            return Ok(Vec::new());
        }
        
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        
        serde_json::from_reader(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    // 保存历史记录
    fn save_history(&self, history: &[HistoryEntry]) -> io::Result<()> {
        let file = File::create(&self.file_path)?;
        let writer = BufWriter::new(file);
        
        serde_json::to_writer_pretty(writer, history)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

// 获取当前时间戳
pub fn current_timestamp() -> String {
    chrono::Local::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_history_manager() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("history.json").to_str().unwrap().to_string();
        
        let manager = HistoryManager::new(&file_path, 3);
        
        // 添加条目
        let entry1 = HistoryEntry {
            expression: "2+2".to_string(),
            result: 4.0,
            timestamp: current_timestamp(),
        };
        
        manager.add_entry(entry1.clone()).unwrap();
        
        // 检查添加
        let history = manager.get_history().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].expression, "2+2");
        
        // 添加更多条目
        for i in 0..5 {
            let entry = HistoryEntry {
                expression: format!("{}+{}", i, i),
                result: (i * 2) as f64,
                timestamp: current_timestamp(),
            };
            manager.add_entry(entry).unwrap();
        }
        
        // 检查最多保留3条
        let history = manager.get_history().unwrap();
        assert_eq!(history.len(), 3);
        
        // 检查最近添加的在最后
        assert_eq!(history[2].expression, "4+4");
        
        // 清空历史
        manager.clear_history().unwrap();
        assert_eq!(manager.get_history().unwrap().len(), 0);
    }
}