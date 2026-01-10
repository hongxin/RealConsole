//! 文件系统存储后端
//!
//! 基于本地文件系统的存储实现

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 文件系统存储后端
///
/// 将数据存储为本地文件，键映射为文件路径
///
/// # 文件结构
///
/// ```text
/// base_dir/
/// ├── namespace1/
/// │   ├── key1.dat
/// │   └── key2.dat
/// └── namespace2/
///     └── key3.dat
/// ```
///
/// # 键格式
///
/// - 简单键: `key1` → `base_dir/key1.dat`
/// - 带命名空间: `ns/key1` → `base_dir/ns/key1.dat`
pub struct FileStorage {
    /// 基础目录
    base_dir: PathBuf,
    /// 文件扩展名
    extension: String,
    /// 统计信息
    stats: FileStorageStats,
}

/// 原子统计计数器
struct FileStorageStats {
    reads: AtomicU64,
    writes: AtomicU64,
    deletes: AtomicU64,
}

impl FileStorage {
    /// 创建文件存储
    ///
    /// # 参数
    /// - `base_dir`: 基础目录路径
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            extension: "dat".to_string(),
            stats: FileStorageStats {
                reads: AtomicU64::new(0),
                writes: AtomicU64::new(0),
                deletes: AtomicU64::new(0),
            },
        }
    }

    /// 设置文件扩展名
    pub fn with_extension(mut self, ext: &str) -> Self {
        self.extension = ext.to_string();
        self
    }

    /// 将键转换为文件路径
    fn key_to_path(&self, key: &str) -> PathBuf {
        let sanitized = key.replace(['/', '\\'], std::path::MAIN_SEPARATOR_STR);
        self.base_dir.join(format!("{}.{}", sanitized, self.extension))
    }

    /// 确保父目录存在
    async fn ensure_parent_dir(&self, path: &Path) -> StorageResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }

    /// 获取基础目录
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

#[async_trait]
impl StorageBackend for FileStorage {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.stats.reads.fetch_add(1, Ordering::Relaxed);

        let path = self.key_to_path(key);

        if !path.exists() {
            return Err(StorageError::NotFound(key.to_string()));
        }

        let mut file = fs::File::open(&path).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;

        Ok(data)
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.stats.writes.fetch_add(1, Ordering::Relaxed);

        let path = self.key_to_path(key);
        self.ensure_parent_dir(&path).await?;

        // 写入临时文件然后原子重命名
        let temp_path = path.with_extension("tmp");

        let mut file = fs::File::create(&temp_path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;

        fs::rename(&temp_path, &path).await?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.stats.deletes.fetch_add(1, Ordering::Relaxed);

        let path = self.key_to_path(key);

        if path.exists() {
            fs::remove_file(&path).await?;
        }

        Ok(())
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let search_dir = if prefix.is_empty() {
            self.base_dir.clone()
        } else {
            self.base_dir.join(prefix)
        };

        if !search_dir.exists() {
            return Ok(vec![]);
        }

        let mut keys = Vec::new();
        let mut entries = fs::read_dir(&search_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(stem) = path.file_stem() {
                    if let Some(ext) = path.extension() {
                        if ext == self.extension.as_str() {
                            let key = if prefix.is_empty() {
                                stem.to_string_lossy().to_string()
                            } else {
                                format!("{}/{}", prefix, stem.to_string_lossy())
                            };
                            keys.push(key);
                        }
                    }
                }
            }
        }

        keys.sort();
        Ok(keys)
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let path = self.key_to_path(key);
        Ok(path.exists())
    }

    fn stats(&self) -> StorageStats {
        StorageStats {
            reads: self.stats.reads.load(Ordering::Relaxed),
            writes: self.stats.writes.load(Ordering::Relaxed),
            deletes: self.stats.deletes.load(Ordering::Relaxed),
            hits: 0,
            misses: 0,
            total_bytes: 0,
            key_count: 0,
        }
    }

    fn name(&self) -> &'static str {
        "FileStorage"
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_storage_write_read() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        // 写入
        storage.write("key1", b"hello world").await.unwrap();

        // 读取
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"hello world");
    }

    #[tokio::test]
    async fn test_file_storage_read_not_found() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        let result = storage.read("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_file_storage_delete() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        storage.write("key1", b"data").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());

        storage.delete("key1").await.unwrap();
        assert!(!storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_file_storage_delete_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        // 删除不存在的键应该成功
        storage.delete("nonexistent").await.unwrap();
    }

    #[tokio::test]
    async fn test_file_storage_list() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        storage.write("key1", b"data1").await.unwrap();
        storage.write("key2", b"data2").await.unwrap();
        storage.write("key3", b"data3").await.unwrap();

        let keys = storage.list("").await.unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[tokio::test]
    async fn test_file_storage_exists() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        assert!(!storage.exists("key1").await.unwrap());

        storage.write("key1", b"data").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_file_storage_overwrite() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        storage.write("key1", b"original").await.unwrap();
        storage.write("key1", b"updated").await.unwrap();

        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"updated");
    }

    #[tokio::test]
    async fn test_file_storage_stats() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        storage.write("key1", b"data").await.unwrap();
        storage.read("key1").await.unwrap();
        storage.delete("key1").await.unwrap();

        let stats = storage.stats();
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.deletes, 1);
    }

    #[tokio::test]
    async fn test_file_storage_custom_extension() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path()).with_extension("json");

        storage.write("key1", b"{}").await.unwrap();

        let path = temp_dir.path().join("key1.json");
        assert!(path.exists());
    }

    #[tokio::test]
    async fn test_file_storage_name() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        assert_eq!(storage.name(), "FileStorage");
    }
}
