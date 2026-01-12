//! 压缩存储层
//!
//! v1.64.0: v2.0 探路期 - 数据压缩
//!
//! ## 设计理念
//!
//! 基于"一分为三"哲学的压缩存储架构：
//! - **原始层**: 应用数据（未压缩）
//! - **压缩层**: 压缩/解压处理
//! - **存储层**: 底层 StorageBackend
//!
//! ## 压缩策略
//!
//! ```text
//! ┌───────────────────────────────────────────────────────┐
//! │                  CompressedStorage                    │
//! ├───────────────────────────────────────────────────────┤
//! │                                                       │
//! │  Write:                                               │
//! │    Raw Data ─────► Compress ─────► Backend           │
//! │                                                       │
//! │  Read:                                                │
//! │    Backend ─────► Decompress ─────► Raw Data         │
//! │                                                       │
//! │  Compression Levels:                                  │
//! │    - Fast (1): 快速压缩，压缩率较低                   │
//! │    - Default (6): 平衡压缩率和速度                    │
//! │    - Best (9): 最佳压缩率，速度较慢                   │
//! │                                                       │
//! │  Threshold:                                           │
//! │    - 小于阈值的数据不压缩（避免膨胀）                 │
//! │                                                       │
//! └───────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{CompressedStorage, FileStorage};
//!
//! let backend = FileStorage::new("/path/to/data");
//! let storage = CompressedStorage::new(backend);
//!
//! // 写入（自动压缩）
//! storage.write("key1", large_data).await?;
//!
//! // 读取（自动解压）
//! let data = storage.read("key1").await?;
//!
//! // 查看压缩统计
//! let stats = storage.compression_stats();
//! println!("Compression ratio: {:.2}%", stats.compression_ratio() * 100.0);
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use flate2::read::{GzDecoder, GzEncoder};
use flate2::Compression;
use std::io::{Read, Write as IoWrite};
use std::sync::atomic::{AtomicU64, Ordering};

/// 压缩级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum CompressionLevel {
    /// 不压缩
    None,
    /// 快速压缩 (level 1)
    Fast,
    /// 默认压缩 (level 6)
    #[default]
    Default,
    /// 最佳压缩 (level 9)
    Best,
    /// 自定义级别 (0-9)
    Custom(u32),
}

impl CompressionLevel {
    /// 转换为 flate2 的 Compression
    fn to_flate2(self) -> Option<Compression> {
        match self {
            CompressionLevel::None => None,
            CompressionLevel::Fast => Some(Compression::fast()),
            CompressionLevel::Default => Some(Compression::default()),
            CompressionLevel::Best => Some(Compression::best()),
            CompressionLevel::Custom(level) => Some(Compression::new(level)),
        }
    }
}


/// 压缩存储配置
#[derive(Debug, Clone)]
pub struct CompressedStorageConfig {
    /// 压缩级别
    pub level: CompressionLevel,
    /// 最小压缩阈值（小于此值的数据不压缩）
    pub min_size_threshold: usize,
    /// 是否在压缩后数据更大时跳过压缩
    pub skip_if_larger: bool,
}

impl Default for CompressedStorageConfig {
    fn default() -> Self {
        Self {
            level: CompressionLevel::Default,
            min_size_threshold: 64, // 64 bytes
            skip_if_larger: true,
        }
    }
}

/// 压缩统计
#[derive(Debug, Default)]
struct CompressionStats {
    /// 压缩次数
    compressions: AtomicU64,
    /// 解压次数
    decompressions: AtomicU64,
    /// 跳过压缩次数（小于阈值或压缩后更大）
    skipped: AtomicU64,
    /// 原始字节总数
    original_bytes: AtomicU64,
    /// 压缩后字节总数
    compressed_bytes: AtomicU64,
}

/// 详细压缩统计
#[derive(Debug, Clone)]
pub struct DetailedCompressionStats {
    /// 压缩次数
    pub compressions: u64,
    /// 解压次数
    pub decompressions: u64,
    /// 跳过压缩次数
    pub skipped: u64,
    /// 原始字节总数
    pub original_bytes: u64,
    /// 压缩后字节总数
    pub compressed_bytes: u64,
}

impl DetailedCompressionStats {
    /// 压缩率（压缩后/原始）
    pub fn compression_ratio(&self) -> f64 {
        if self.original_bytes == 0 {
            1.0
        } else {
            self.compressed_bytes as f64 / self.original_bytes as f64
        }
    }

    /// 节省的空间比例
    pub fn space_savings(&self) -> f64 {
        1.0 - self.compression_ratio()
    }

    /// 平均原始大小
    pub fn avg_original_size(&self) -> f64 {
        let total = self.compressions + self.skipped;
        if total == 0 {
            0.0
        } else {
            self.original_bytes as f64 / total as f64
        }
    }

    /// 平均压缩大小
    pub fn avg_compressed_size(&self) -> f64 {
        if self.compressions == 0 {
            0.0
        } else {
            self.compressed_bytes as f64 / self.compressions as f64
        }
    }
}

/// 数据头标记
const COMPRESSED_MARKER: &[u8] = b"CMP1"; // Compressed v1
const UNCOMPRESSED_MARKER: &[u8] = b"RAW1"; // Raw v1

/// 压缩存储
///
/// 自动压缩/解压数据，减少存储空间
pub struct CompressedStorage<B: StorageBackend> {
    /// 后端存储
    backend: B,
    /// 配置
    config: CompressedStorageConfig,
    /// 统计
    stats: CompressionStats,
}

impl<B: StorageBackend> CompressedStorage<B> {
    /// 创建压缩存储（默认配置）
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, CompressedStorageConfig::default())
    }

    /// 使用快速压缩创建
    pub fn with_fast(backend: B) -> Self {
        Self::with_config(
            backend,
            CompressedStorageConfig {
                level: CompressionLevel::Fast,
                ..Default::default()
            },
        )
    }

    /// 使用最佳压缩创建
    pub fn with_best(backend: B) -> Self {
        Self::with_config(
            backend,
            CompressedStorageConfig {
                level: CompressionLevel::Best,
                ..Default::default()
            },
        )
    }

    /// 使用自定义配置创建
    pub fn with_config(backend: B, config: CompressedStorageConfig) -> Self {
        Self {
            backend,
            config,
            stats: CompressionStats::default(),
        }
    }

    /// 压缩数据
    fn compress(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        // 检查是否应该压缩
        if data.len() < self.config.min_size_threshold {
            self.stats.skipped.fetch_add(1, Ordering::Relaxed);
            self.stats
                .original_bytes
                .fetch_add(data.len() as u64, Ordering::Relaxed);
            return Ok(self.wrap_uncompressed(data));
        }

        // 检查压缩级别
        let compression = match self.config.level.to_flate2() {
            Some(c) => c,
            None => {
                self.stats.skipped.fetch_add(1, Ordering::Relaxed);
                self.stats
                    .original_bytes
                    .fetch_add(data.len() as u64, Ordering::Relaxed);
                return Ok(self.wrap_uncompressed(data));
            }
        };

        // 执行压缩
        let mut encoder = GzEncoder::new(data, compression);
        let mut compressed = Vec::new();
        encoder
            .read_to_end(&mut compressed)
            .map_err(|e| StorageError::Other(format!("Compression error: {}", e)))?;

        // 检查压缩后是否更大
        if self.config.skip_if_larger && compressed.len() >= data.len() {
            self.stats.skipped.fetch_add(1, Ordering::Relaxed);
            self.stats
                .original_bytes
                .fetch_add(data.len() as u64, Ordering::Relaxed);
            return Ok(self.wrap_uncompressed(data));
        }

        self.stats.compressions.fetch_add(1, Ordering::Relaxed);
        self.stats
            .original_bytes
            .fetch_add(data.len() as u64, Ordering::Relaxed);
        self.stats
            .compressed_bytes
            .fetch_add(compressed.len() as u64, Ordering::Relaxed);

        Ok(self.wrap_compressed(&compressed))
    }

    /// 解压数据
    fn decompress(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        if data.len() < 4 {
            return Err(StorageError::Other("Invalid compressed data: too short".into()));
        }

        let marker = &data[0..4];

        if marker == UNCOMPRESSED_MARKER {
            // 未压缩数据
            return Ok(data[4..].to_vec());
        }

        if marker != COMPRESSED_MARKER {
            // 兼容旧数据（没有标记）
            return self.try_decompress_legacy(data);
        }

        // 解压
        self.stats.decompressions.fetch_add(1, Ordering::Relaxed);

        let mut decoder = GzDecoder::new(&data[4..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| StorageError::Other(format!("Decompression error: {}", e)))?;

        Ok(decompressed)
    }

    /// 尝试解压旧格式数据
    fn try_decompress_legacy(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        // 尝试直接解压（兼容没有标记的旧数据）
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();

        match decoder.read_to_end(&mut decompressed) {
            Ok(_) => {
                self.stats.decompressions.fetch_add(1, Ordering::Relaxed);
                Ok(decompressed)
            }
            Err(_) => {
                // 不是压缩数据，返回原始数据
                Ok(data.to_vec())
            }
        }
    }

    /// 包装压缩数据
    fn wrap_compressed(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(4 + data.len());
        result.extend_from_slice(COMPRESSED_MARKER);
        result.extend_from_slice(data);
        result
    }

    /// 包装未压缩数据
    fn wrap_uncompressed(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(4 + data.len());
        result.extend_from_slice(UNCOMPRESSED_MARKER);
        result.extend_from_slice(data);
        result
    }

    /// 获取详细压缩统计
    pub fn compression_stats(&self) -> DetailedCompressionStats {
        DetailedCompressionStats {
            compressions: self.stats.compressions.load(Ordering::Relaxed),
            decompressions: self.stats.decompressions.load(Ordering::Relaxed),
            skipped: self.stats.skipped.load(Ordering::Relaxed),
            original_bytes: self.stats.original_bytes.load(Ordering::Relaxed),
            compressed_bytes: self.stats.compressed_bytes.load(Ordering::Relaxed),
        }
    }

    /// 获取配置
    pub fn config(&self) -> &CompressedStorageConfig {
        &self.config
    }

    /// 获取后端引用
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// 获取压缩级别
    pub fn level(&self) -> CompressionLevel {
        self.config.level
    }
}

#[async_trait]
impl<B: StorageBackend + Send + Sync> StorageBackend for CompressedStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        let compressed = self.backend.read(key).await?;
        self.decompress(&compressed)
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        let compressed = self.compress(data)?;
        self.backend.write(key, &compressed).await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.backend.delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        self.backend.list(prefix).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        self.backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        let backend_stats = self.backend.stats();
        let compression_stats = self.compression_stats();

        StorageStats {
            reads: compression_stats.decompressions,
            writes: compression_stats.compressions + compression_stats.skipped,
            deletes: backend_stats.deletes,
            hits: 0,
            misses: 0,
            total_bytes: compression_stats.compressed_bytes,
            key_count: backend_stats.key_count,
        }
    }

    fn name(&self) -> &'static str {
        "CompressedStorage"
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[tokio::test]
    async fn test_compressed_storage_new() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        assert_eq!(storage.level(), CompressionLevel::Default);
        assert_eq!(storage.name(), "CompressedStorage");
    }

    #[tokio::test]
    async fn test_compressed_storage_write_read() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        let data = b"Hello, World! This is a test message that should be compressed.";
        storage.write("key1", data).await.unwrap();

        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data);
    }

    #[tokio::test]
    async fn test_compressed_storage_large_data() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        // 创建大量重复数据（压缩效果好）
        let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        storage.write("key1", &data).await.unwrap();

        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data);

        // 验证压缩发生
        let stats = storage.compression_stats();
        assert_eq!(stats.compressions, 1);
        assert!(stats.compression_ratio() < 1.0);
    }

    #[tokio::test]
    async fn test_compressed_storage_small_data_skipped() {
        let backend = MemoryStorage::new();
        let config = CompressedStorageConfig {
            min_size_threshold: 100,
            ..Default::default()
        };
        let storage = CompressedStorage::with_config(backend, config);

        // 小数据应该跳过压缩
        let data = b"small";
        storage.write("key1", data).await.unwrap();

        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data);

        let stats = storage.compression_stats();
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.compressions, 0);
    }

    #[tokio::test]
    async fn test_compressed_storage_delete() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        storage.write("key1", b"data").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());

        storage.delete("key1").await.unwrap();
        assert!(!storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_compressed_storage_list() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        storage.write("key1", b"data1").await.unwrap();
        storage.write("key2", b"data2").await.unwrap();

        let keys = storage.list("").await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_compressed_storage_fast() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::with_fast(backend);

        assert_eq!(storage.level(), CompressionLevel::Fast);

        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        storage.write("key1", &data).await.unwrap();

        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data);
    }

    #[tokio::test]
    async fn test_compressed_storage_best() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::with_best(backend);

        assert_eq!(storage.level(), CompressionLevel::Best);

        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        storage.write("key1", &data).await.unwrap();

        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data);
    }

    #[tokio::test]
    async fn test_compressed_storage_no_compression() {
        let backend = MemoryStorage::new();
        let config = CompressedStorageConfig {
            level: CompressionLevel::None,
            ..Default::default()
        };
        let storage = CompressedStorage::with_config(backend, config);

        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        storage.write("key1", &data).await.unwrap();

        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data);

        let stats = storage.compression_stats();
        assert_eq!(stats.compressions, 0);
        assert_eq!(stats.skipped, 1);
    }

    #[tokio::test]
    async fn test_compressed_storage_compression_ratio() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        // 高度可压缩的数据（全是相同字节）
        let data = vec![0u8; 10000];
        storage.write("key1", &data).await.unwrap();

        let stats = storage.compression_stats();
        assert!(stats.compression_ratio() < 0.1); // 压缩率应该很低
        assert!(stats.space_savings() > 0.9); // 节省超过 90%
    }

    #[tokio::test]
    async fn test_compressed_storage_skip_if_larger() {
        let backend = MemoryStorage::new();
        let config = CompressedStorageConfig {
            min_size_threshold: 0, // 不跳过小数据
            skip_if_larger: true,
            ..Default::default()
        };
        let storage = CompressedStorage::with_config(backend, config);

        // 随机数据压缩效果差
        let data: Vec<u8> = (0..100).map(|_| rand::random::<u8>()).collect();
        storage.write("key1", &data).await.unwrap();

        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data);
    }

    #[tokio::test]
    async fn test_compressed_storage_stats() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        // 写入可压缩数据
        let data = vec![0u8; 1000];
        storage.write("key1", &data).await.unwrap();
        storage.read("key1").await.unwrap();

        let stats = storage.compression_stats();
        assert_eq!(stats.compressions, 1);
        assert_eq!(stats.decompressions, 1);
        assert_eq!(stats.original_bytes, 1000);
        assert!(stats.compressed_bytes < 1000);
    }

    #[tokio::test]
    async fn test_compressed_storage_default_config() {
        let config = CompressedStorageConfig::default();

        assert_eq!(config.level, CompressionLevel::Default);
        assert_eq!(config.min_size_threshold, 64);
        assert!(config.skip_if_larger);
    }

    #[tokio::test]
    async fn test_compressed_storage_read_not_found() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        let result = storage.read("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_compression_level_custom() {
        let backend = MemoryStorage::new();
        let config = CompressedStorageConfig {
            level: CompressionLevel::Custom(3),
            ..Default::default()
        };
        let storage = CompressedStorage::with_config(backend, config);

        let data = vec![0u8; 1000];
        storage.write("key1", &data).await.unwrap();

        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data);
    }

    #[tokio::test]
    async fn test_compressed_storage_multiple_writes() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        for i in 0..5 {
            let data = vec![i as u8; 1000];
            storage.write(&format!("key{}", i), &data).await.unwrap();
        }

        let stats = storage.compression_stats();
        assert_eq!(stats.compressions, 5);
    }

    #[tokio::test]
    async fn test_compressed_storage_overwrite() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        let data1 = vec![1u8; 1000];
        let data2 = vec![2u8; 1000];

        storage.write("key1", &data1).await.unwrap();
        storage.write("key1", &data2).await.unwrap();

        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data2);
    }

    #[tokio::test]
    async fn test_detailed_stats_averages() {
        let backend = MemoryStorage::new();
        let storage = CompressedStorage::new(backend);

        // 写入 3 个相同大小的数据
        for i in 0..3 {
            let data = vec![i as u8; 1000];
            storage.write(&format!("key{}", i), &data).await.unwrap();
        }

        let stats = storage.compression_stats();
        assert!((stats.avg_original_size() - 1000.0).abs() < 1.0);
        assert!(stats.avg_compressed_size() < 1000.0);
    }
}
