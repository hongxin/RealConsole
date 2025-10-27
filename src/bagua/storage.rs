//! 八卦记忆宫持久化存储
//!
//! 使用 JSONL 格式按维度存储记忆条目

use super::dimension::BaguaDimension;
use super::entry::MemoryEntry;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

/// 八卦记忆宫存储
///
/// 负责持久化和加载八维记忆数据
pub struct BaguaStorage {
    /// 存储根目录
    base_path: PathBuf,
}

impl BaguaStorage {
    /// 创建新的存储实例
    ///
    /// # Arguments
    /// * `base_path` - 存储根目录，通常为 ~/.realconsole/bagua
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// 从配置字符串创建
    ///
    /// # Arguments
    /// * `path_str` - 路径字符串，支持 ~ 展开
    pub fn from_config(path_str: &str) -> Result<Self> {
        let expanded_path = if path_str.starts_with('~') {
            // 展开 ~ 为用户主目录
            if let Some(home) = dirs::home_dir() {
                let remainder = path_str.strip_prefix('~').unwrap_or(path_str);
                let remainder = remainder.strip_prefix('/').unwrap_or(remainder);
                home.join(remainder)
            } else {
                PathBuf::from(path_str)
            }
        } else {
            PathBuf::from(path_str)
        };

        Ok(Self::new(expanded_path))
    }

    /// 使用默认位置创建
    ///
    /// 默认位置：~/.realconsole/bagua
    pub fn from_default_location() -> Result<Self> {
        let home = dirs::home_dir().context("无法获取用户主目录")?;
        let path = home.join(".realconsole").join("bagua");
        Ok(Self::new(path))
    }

    /// 获取维度的文件路径
    fn dimension_file_path(&self, dimension: BaguaDimension) -> PathBuf {
        self.base_path.join(format!("{:?}.jsonl", dimension))
    }

    /// 确保存储目录存在
    async fn ensure_directory(&self) -> Result<()> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)
                .await
                .with_context(|| format!("无法创建目录: {:?}", self.base_path))?;
        }
        Ok(())
    }

    /// 保存某个维度的数据
    ///
    /// 使用追加模式，不会覆盖现有数据
    /// ✨ v1.8.4 性能优化：使用 BufWriter 减少系统调用
    pub async fn append_dimension(
        &self,
        dimension: BaguaDimension,
        entries: &[MemoryEntry],
    ) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        self.ensure_directory().await?;

        let path = self.dimension_file_path(dimension);
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .with_context(|| format!("无法打开文件: {:?}", path))?;

        // ✨ 使用 BufWriter 缓冲写入，提升性能
        let mut writer = BufWriter::new(file);

        for entry in entries {
            let line = serde_json::to_string(entry)?;
            writer.write_all(line.as_bytes()).await?;
            writer.write_all(b"\n").await?;
        }

        writer.flush().await?;
        Ok(())
    }

    /// 覆盖保存某个维度的全部数据
    ///
    /// 会删除原有数据
    /// ✨ v1.8.4 性能优化：使用 BufWriter 减少系统调用
    pub async fn save_dimension(
        &self,
        dimension: BaguaDimension,
        entries: &[MemoryEntry],
    ) -> Result<()> {
        self.ensure_directory().await?;

        let path = self.dimension_file_path(dimension);
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .await
            .with_context(|| format!("无法打开文件: {:?}", path))?;

        // ✨ 使用 BufWriter 缓冲写入，提升性能
        let mut writer = BufWriter::new(file);

        for entry in entries {
            let line = serde_json::to_string(entry)?;
            writer.write_all(line.as_bytes()).await?;
            writer.write_all(b"\n").await?;
        }

        writer.flush().await?;
        Ok(())
    }

    /// 加载某个维度的数据
    ///
    /// # Arguments
    /// * `dimension` - 要加载的维度
    /// * `limit` - 最多加载多少条（从最新的开始）
    pub async fn load_dimension(
        &self,
        dimension: BaguaDimension,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryEntry>> {
        let path = self.dimension_file_path(dimension);

        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&path)
            .await
            .with_context(|| format!("无法打开文件: {:?}", path))?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut entries = Vec::new();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<MemoryEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("⚠️ 解析条目失败: {} (行: {})", e, line);
                    continue;
                }
            }
        }

        // 反转以获得最新的在前
        entries.reverse();

        // 限制数量
        if let Some(limit) = limit {
            entries.truncate(limit);
        }

        Ok(entries)
    }

    /// 加载所有维度的数据
    ///
    /// # Arguments
    /// * `limit_per_dimension` - 每个维度最多加载多少条
    pub async fn load_all_dimensions(
        &self,
        limit_per_dimension: Option<usize>,
    ) -> Result<std::collections::HashMap<BaguaDimension, Vec<MemoryEntry>>> {
        let mut result = std::collections::HashMap::new();

        for dimension in BaguaDimension::all() {
            let entries = self.load_dimension(dimension, limit_per_dimension).await?;
            if !entries.is_empty() {
                result.insert(dimension, entries);
            }
        }

        Ok(result)
    }

    /// 清理过期数据
    ///
    /// 删除超过 retention_days 天的条目
    pub async fn cleanup_expired(
        &self,
        dimension: BaguaDimension,
        retention_days: u64,
    ) -> Result<usize> {
        let entries = self.load_dimension(dimension, None).await?;
        let now = chrono::Utc::now();
        let retention_duration = chrono::Duration::days(retention_days as i64);

        let mut removed_count = 0;

        let mut valid_entries = Vec::new();
        for entry in entries {
            if now.signed_duration_since(entry.timestamp) < retention_duration {
                valid_entries.push(entry);
            } else {
                removed_count += 1;
            }
        }

        // 如果有删除，重新保存
        if removed_count > 0 {
            self.save_dimension(dimension, &valid_entries).await?;
        }

        Ok(removed_count)
    }

    /// 获取存储统计信息
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let mut total_entries = 0;
        let mut dimension_counts = std::collections::HashMap::new();
        let mut total_size_bytes = 0;

        for dimension in BaguaDimension::all() {
            let path = self.dimension_file_path(dimension);
            if path.exists() {
                let entries = self.load_dimension(dimension, None).await?;
                let count = entries.len();
                dimension_counts.insert(dimension, count);
                total_entries += count;

                if let Ok(metadata) = fs::metadata(&path).await {
                    total_size_bytes += metadata.len();
                }
            }
        }

        Ok(StorageStats {
            total_entries,
            dimension_counts,
            total_size_bytes,
            storage_path: self.base_path.clone(),
        })
    }
}

/// 存储统计信息
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// 总条目数
    pub total_entries: usize,

    /// 各维度的条目数
    pub dimension_counts: std::collections::HashMap<BaguaDimension, usize>,

    /// 总大小（字节）
    pub total_size_bytes: u64,

    /// 存储路径
    pub storage_path: PathBuf,
}

impl StorageStats {
    /// 格式化存储大小
    pub fn formatted_size(&self) -> String {
        let kb = self.total_size_bytes as f64 / 1024.0;
        if kb < 1024.0 {
            format!("{:.2} KB", kb)
        } else {
            format!("{:.2} MB", kb / 1024.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bagua::entry::MemoryContent;

    #[tokio::test]
    async fn test_storage_creation() {
        let storage = BaguaStorage::new(PathBuf::from("/tmp/test_bagua"));
        assert_eq!(storage.base_path, PathBuf::from("/tmp/test_bagua"));
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = BaguaStorage::new(temp_dir.path().to_path_buf());

        let entries = vec![
            MemoryEntry::new(
                BaguaDimension::Qian,
                MemoryContent::Intent {
                    goal: "测试目标".to_string(),
                    context: None,
                    priority: 0.8,
                },
            ),
            MemoryEntry::new(
                BaguaDimension::Qian,
                MemoryContent::Intent {
                    goal: "另一个目标".to_string(),
                    context: Some("上下文".to_string()),
                    priority: 0.9,
                },
            ),
        ];

        // 保存
        storage
            .save_dimension(BaguaDimension::Qian, &entries)
            .await
            .unwrap();

        // 加载
        let loaded = storage
            .load_dimension(BaguaDimension::Qian, None)
            .await
            .unwrap();

        assert_eq!(loaded.len(), 2);
    }

    #[tokio::test]
    async fn test_load_with_limit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = BaguaStorage::new(temp_dir.path().to_path_buf());

        let entries: Vec<_> = (0..10)
            .map(|i| {
                MemoryEntry::new(
                    BaguaDimension::Zhen,
                    MemoryContent::Action {
                        command: format!("command_{}", i),
                        result: crate::bagua::entry::ActionResult::Success,
                        duration_ms: 100,
                    },
                )
            })
            .collect();

        storage
            .save_dimension(BaguaDimension::Zhen, &entries)
            .await
            .unwrap();

        let loaded = storage
            .load_dimension(BaguaDimension::Zhen, Some(5))
            .await
            .unwrap();

        assert_eq!(loaded.len(), 5);
    }
}
