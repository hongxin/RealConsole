//! 内存存储后端
//!
//! 基于内存的存储实现，适用于测试和缓存场景

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// 内存存储后端
///
/// 将数据存储在内存中，适用于：
/// - 单元测试
/// - 临时缓存
/// - 高性能场景
///
/// # 线程安全
///
/// 使用 `RwLock` 实现并发安全，读操作可并发，写操作独占
///
/// # 注意
///
/// - 数据不持久化，进程结束后丢失
/// - 内存使用量随数据增长
pub struct MemoryStorage {
    /// 数据存储
    data: RwLock<HashMap<String, Vec<u8>>>,
    /// 统计信息
    stats: MemoryStorageStats,
}

/// 原子统计计数器
struct MemoryStorageStats {
    reads: AtomicU64,
    writes: AtomicU64,
    deletes: AtomicU64,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl MemoryStorage {
    /// 创建空的内存存储
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            stats: MemoryStorageStats {
                reads: AtomicU64::new(0),
                writes: AtomicU64::new(0),
                deletes: AtomicU64::new(0),
                hits: AtomicU64::new(0),
                misses: AtomicU64::new(0),
            },
        }
    }

    /// 从现有数据创建
    pub fn with_data(data: HashMap<String, Vec<u8>>) -> Self {
        Self {
            data: RwLock::new(data),
            stats: MemoryStorageStats {
                reads: AtomicU64::new(0),
                writes: AtomicU64::new(0),
                deletes: AtomicU64::new(0),
                hits: AtomicU64::new(0),
                misses: AtomicU64::new(0),
            },
        }
    }

    /// 获取当前键数量
    pub fn len(&self) -> usize {
        self.data.read().unwrap().len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.data.read().unwrap().is_empty()
    }

    /// 清空所有数据
    pub fn clear(&self) {
        self.data.write().unwrap().clear();
    }

    /// 获取总数据量（字节）
    pub fn total_bytes(&self) -> usize {
        self.data
            .read()
            .unwrap()
            .values()
            .map(|v| v.len())
            .sum()
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for MemoryStorage {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.stats.reads.fetch_add(1, Ordering::Relaxed);

        let data = self.data.read().unwrap();
        match data.get(key) {
            Some(value) => {
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                Ok(value.clone())
            }
            None => {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                Err(StorageError::NotFound(key.to_string()))
            }
        }
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.stats.writes.fetch_add(1, Ordering::Relaxed);

        let mut storage = self.data.write().unwrap();
        storage.insert(key.to_string(), data.to_vec());

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.stats.deletes.fetch_add(1, Ordering::Relaxed);

        let mut storage = self.data.write().unwrap();
        storage.remove(key);

        Ok(())
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let data = self.data.read().unwrap();
        let mut keys: Vec<String> = data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        keys.sort();
        Ok(keys)
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data.contains_key(key))
    }

    fn stats(&self) -> StorageStats {
        let data = self.data.read().unwrap();
        StorageStats {
            reads: self.stats.reads.load(Ordering::Relaxed),
            writes: self.stats.writes.load(Ordering::Relaxed),
            deletes: self.stats.deletes.load(Ordering::Relaxed),
            hits: self.stats.hits.load(Ordering::Relaxed),
            misses: self.stats.misses.load(Ordering::Relaxed),
            total_bytes: data.values().map(|v| v.len() as u64).sum(),
            key_count: data.len(),
        }
    }

    fn name(&self) -> &'static str {
        "MemoryStorage"
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_write_read() {
        let storage = MemoryStorage::new();

        storage.write("key1", b"hello world").await.unwrap();

        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"hello world");
    }

    #[tokio::test]
    async fn test_memory_storage_read_not_found() {
        let storage = MemoryStorage::new();

        let result = storage.read("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_memory_storage_delete() {
        let storage = MemoryStorage::new();

        storage.write("key1", b"data").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());

        storage.delete("key1").await.unwrap();
        assert!(!storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_storage_list() {
        let storage = MemoryStorage::new();

        storage.write("ns/key1", b"data1").await.unwrap();
        storage.write("ns/key2", b"data2").await.unwrap();
        storage.write("other/key3", b"data3").await.unwrap();

        let all_keys = storage.list("").await.unwrap();
        assert_eq!(all_keys.len(), 3);

        let ns_keys = storage.list("ns/").await.unwrap();
        assert_eq!(ns_keys.len(), 2);
    }

    #[tokio::test]
    async fn test_memory_storage_exists() {
        let storage = MemoryStorage::new();

        assert!(!storage.exists("key1").await.unwrap());

        storage.write("key1", b"data").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_storage_overwrite() {
        let storage = MemoryStorage::new();

        storage.write("key1", b"original").await.unwrap();
        storage.write("key1", b"updated").await.unwrap();

        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"updated");
    }

    #[tokio::test]
    async fn test_memory_storage_clear() {
        let storage = MemoryStorage::new();

        storage.write("key1", b"data1").await.unwrap();
        storage.write("key2", b"data2").await.unwrap();
        assert_eq!(storage.len(), 2);

        storage.clear();
        assert!(storage.is_empty());
    }

    #[tokio::test]
    async fn test_memory_storage_stats() {
        let storage = MemoryStorage::new();

        storage.write("key1", b"data").await.unwrap();
        storage.read("key1").await.unwrap();
        let _ = storage.read("nonexistent").await;
        storage.delete("key1").await.unwrap();

        let stats = storage.stats();
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.reads, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.deletes, 1);
    }

    #[tokio::test]
    async fn test_memory_storage_hit_rate() {
        let storage = MemoryStorage::new();

        storage.write("key1", b"data").await.unwrap();

        // 4 hits
        for _ in 0..4 {
            storage.read("key1").await.unwrap();
        }

        // 1 miss
        let _ = storage.read("nonexistent").await;

        let stats = storage.stats();
        assert!((stats.hit_rate() - 0.8).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_memory_storage_with_data() {
        let mut initial_data = HashMap::new();
        initial_data.insert("key1".to_string(), b"value1".to_vec());
        initial_data.insert("key2".to_string(), b"value2".to_vec());

        let storage = MemoryStorage::with_data(initial_data);

        assert_eq!(storage.len(), 2);
        assert_eq!(storage.read("key1").await.unwrap(), b"value1");
    }

    #[tokio::test]
    async fn test_memory_storage_total_bytes() {
        let storage = MemoryStorage::new();

        storage.write("key1", b"hello").await.unwrap(); // 5 bytes
        storage.write("key2", b"world").await.unwrap(); // 5 bytes

        assert_eq!(storage.total_bytes(), 10);
    }

    #[tokio::test]
    async fn test_memory_storage_name() {
        let storage = MemoryStorage::new();
        assert_eq!(storage.name(), "MemoryStorage");
    }

    #[test]
    fn test_memory_storage_default() {
        let storage = MemoryStorage::default();
        assert!(storage.is_empty());
    }
}
