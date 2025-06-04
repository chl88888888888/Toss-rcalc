use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::File;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter}; // 添加显式导入

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: f64,
    pub timestamp: String,
}

pub struct HistoryManager {
    file_path: String,
    max_entries: usize,
}

impl HistoryManager {
    pub fn new(file_path: &str, max_entries: usize) -> Self {
        if let Some(parent) = Path::new(file_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).expect("Failed to create history directory");
            }
        }

        HistoryManager {
            file_path: file_path.to_string(),
            max_entries,
        }
    }

    pub async fn add_entry(&self, entry: HistoryEntry) -> io::Result<()> {
        let mut history = self.load_history().await.unwrap_or_default();

        history.push(entry);

        if history.len() > self.max_entries {
            history.remove(0);
        }

        self.save_history(&history).await
    }

    pub async fn get_history(&self) -> io::Result<Vec<HistoryEntry>> {
        self.load_history().await
    }

    pub async fn clear_history(&self) -> io::Result<()> {
        self.save_history(&Vec::new()).await
    }

    async fn load_history(&self) -> io::Result<Vec<HistoryEntry>> {
        if !Path::new(&self.file_path).exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.file_path).await?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).await?;

        serde_json::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn save_history(&self, history: &[HistoryEntry]) -> io::Result<()> {
        let file = File::create(&self.file_path).await?;
        let mut writer = BufWriter::new(file);

        let json = serde_json::to_string_pretty(history)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        writer.write_all(json.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }

    pub fn clone_manager(&self) -> Self {
        HistoryManager {
            file_path: self.file_path.clone(),
            max_entries: self.max_entries,
        }
    }
}

pub fn current_timestamp() -> String {
    chrono::Local::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::runtime::Builder; // 使用 Builder 替代 Runtime::new

    #[test]
    fn test_history_manager() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir
            .path()
            .join("history.json")
            .to_str()
            .unwrap()
            .to_string();

        let manager = HistoryManager::new(&file_path, 3);

        // 创建多线程运行时
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();

        rt.block_on(async {
            // 添加条目
            let entry1 = HistoryEntry {
                expression: "2+2".to_string(),
                result: 4.0,
                timestamp: current_timestamp(),
            };

            manager.add_entry(entry1.clone()).await.unwrap();

            // 检查添加
            let history = manager.get_history().await.unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].expression, "2+2");

            // 添加更多条目
            for i in 0..5 {
                let entry = HistoryEntry {
                    expression: format!("{}+{}", i, i),
                    result: (i * 2) as f64,
                    timestamp: current_timestamp(),
                };
                manager.add_entry(entry).await.unwrap();
            }

            // 检查最多保留3条
            let history = manager.get_history().await.unwrap();
            assert_eq!(history.len(), 3);

            // 检查最近添加的在最后
            assert_eq!(history[2].expression, "4+4");

            // 清空历史
            manager.clear_history().await.unwrap();
            assert_eq!(manager.get_history().await.unwrap().len(), 0);
        });
    }
}
