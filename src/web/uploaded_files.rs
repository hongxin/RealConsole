//! 上传文件管理器
//!
//! v1.46.0: 用于存储和管理浏览器上传的 CSV 文件
//!
//! 功能：
//! - LRU 缓存（最多 10 个文件）
//! - 大小限制（单文件 1MB，总计 5MB）
//! - 自动生成文件 ID
//! - 会话结束自动清理

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 文件大小限制
const MAX_FILE_SIZE: usize = 1024 * 1024; // 1MB
const MAX_TOTAL_SIZE: usize = 5 * 1024 * 1024; // 5MB
const MAX_FILE_COUNT: usize = 10;

/// 上传的文件信息
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// 文件 ID
    pub id: String,
    /// 文件名
    pub filename: String,
    /// 文件内容（CSV 文本）
    pub content: String,
    /// 文件大小（字节）
    pub size: usize,
    /// 上传时间（时间戳）
    pub uploaded_at: std::time::Instant,
}

/// 上传文件管理器
#[derive(Clone)]
pub struct UploadedFiles {
    files: Arc<RwLock<HashMap<String, UploadedFile>>>,
    next_id: Arc<RwLock<usize>>,
}

impl UploadedFiles {
    /// 创建新的文件管理器
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// 添加文件
    pub fn add(&self, filename: String, content: String) -> Result<String> {
        let size = content.len();

        // 检查单文件大小
        if size > MAX_FILE_SIZE {
            return Err(anyhow!(
                "文件过大：{:.2}MB，最大允许 1MB",
                size as f64 / 1024.0 / 1024.0
            ));
        }

        // 获取写锁
        let mut files = self.files.write().map_err(|e| anyhow!("获取锁失败: {}", e))?;

        // 检查文件数量限制
        if files.len() >= MAX_FILE_COUNT {
            // 删除最旧的文件（LRU）
            if let Some(oldest_id) = self.find_oldest_file(&files) {
                files.remove(&oldest_id);
            }
        }

        // 检查总大小限制
        let total_size: usize = files.values().map(|f| f.size).sum();
        if total_size + size > MAX_TOTAL_SIZE {
            // 删除最旧的文件直到有足够空间
            while files.values().map(|f| f.size).sum::<usize>() + size > MAX_TOTAL_SIZE {
                if let Some(oldest_id) = self.find_oldest_file(&files) {
                    files.remove(&oldest_id);
                } else {
                    break;
                }
            }
        }

        // 生成文件 ID
        let file_id = {
            let mut next_id = self.next_id.write().map_err(|e| anyhow!("获取锁失败: {}", e))?;
            let id = format!("uploaded_{:03}", *next_id);
            *next_id += 1;
            id
        };

        // 创建文件记录
        let file = UploadedFile {
            id: file_id.clone(),
            filename: filename.clone(),
            content,
            size,
            uploaded_at: std::time::Instant::now(),
        };

        // 存储文件
        files.insert(file_id.clone(), file);

        Ok(file_id)
    }

    /// 获取文件内容
    pub fn get(&self, file_id: &str) -> Result<String> {
        let files = self.files.read().map_err(|e| anyhow!("获取锁失败: {}", e))?;

        files
            .get(file_id)
            .map(|f| f.content.clone())
            .ok_or_else(|| anyhow!("文件不存在: {}", file_id))
    }

    /// 获取文件信息（不含内容）
    pub fn get_info(&self, file_id: &str) -> Result<(String, usize)> {
        let files = self.files.read().map_err(|e| anyhow!("获取锁失败: {}", e))?;

        files
            .get(file_id)
            .map(|f| (f.filename.clone(), f.size))
            .ok_or_else(|| anyhow!("文件不存在: {}", file_id))
    }

    /// 列出所有文件
    pub fn list(&self) -> Result<Vec<(String, String, usize)>> {
        let files = self.files.read().map_err(|e| anyhow!("获取锁失败: {}", e))?;

        Ok(files
            .values()
            .map(|f| (f.id.clone(), f.filename.clone(), f.size))
            .collect())
    }

    /// 删除文件
    pub fn remove(&self, file_id: &str) -> Result<()> {
        let mut files = self.files.write().map_err(|e| anyhow!("获取锁失败: {}", e))?;

        files
            .remove(file_id)
            .ok_or_else(|| anyhow!("文件不存在: {}", file_id))?;

        Ok(())
    }

    /// 清空所有文件
    pub fn clear(&self) -> Result<()> {
        let mut files = self.files.write().map_err(|e| anyhow!("获取锁失败: {}", e))?;
        files.clear();
        Ok(())
    }

    /// 获取统计信息
    pub fn stats(&self) -> Result<(usize, usize)> {
        let files = self.files.read().map_err(|e| anyhow!("获取锁失败: {}", e))?;

        let count = files.len();
        let total_size = files.values().map(|f| f.size).sum();

        Ok((count, total_size))
    }

    /// 查找最旧的文件 ID（LRU）
    fn find_oldest_file(&self, files: &HashMap<String, UploadedFile>) -> Option<String> {
        files
            .values()
            .min_by_key(|f| f.uploaded_at)
            .map(|f| f.id.clone())
    }
}

impl Default for UploadedFiles {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_file() {
        let manager = UploadedFiles::new();
        let content = "col1,col2\n1,2\n3,4".to_string();

        let file_id = manager.add("test.csv".to_string(), content.clone()).unwrap();
        assert_eq!(file_id, "uploaded_001");

        let retrieved = manager.get(&file_id).unwrap();
        assert_eq!(retrieved, content);
    }

    #[test]
    fn test_file_size_limit() {
        let manager = UploadedFiles::new();
        let large_content = "x".repeat(2 * 1024 * 1024); // 2MB

        let result = manager.add("large.csv".to_string(), large_content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("文件过大"));
    }

    #[test]
    fn test_lru_eviction() {
        let manager = UploadedFiles::new();

        // 添加 11 个小文件（超过 MAX_FILE_COUNT = 10）
        for i in 1..=11 {
            let content = format!("data{}", i);
            manager.add(format!("file{}.csv", i), content).unwrap();
        }

        // 第一个文件应该被移除
        let result = manager.get("uploaded_001");
        assert!(result.is_err());

        // 第二个文件应该存在
        let result = manager.get("uploaded_002");
        assert!(result.is_ok());

        // 总共应该有 10 个文件
        let (count, _) = manager.stats().unwrap();
        assert_eq!(count, 10);
    }

    #[test]
    fn test_list_files() {
        let manager = UploadedFiles::new();

        manager.add("file1.csv".to_string(), "a,b\n1,2".to_string()).unwrap();
        manager.add("file2.csv".to_string(), "x,y\n3,4".to_string()).unwrap();

        let files = manager.list().unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_remove_file() {
        let manager = UploadedFiles::new();

        let file_id = manager.add("test.csv".to_string(), "data".to_string()).unwrap();
        assert!(manager.get(&file_id).is_ok());

        manager.remove(&file_id).unwrap();
        assert!(manager.get(&file_id).is_err());
    }
}
