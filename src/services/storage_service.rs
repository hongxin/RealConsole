//! 存储服务
//!
//! v1.104.0 新增：为服务层提供统一的存储访问
//!
//! # 功能特性
//! - 封装 Storage Layer 2.0
//! - 提供服务级别的存储抽象
//! - 支持命名空间隔离
//! - 集成生命周期管理
//!
//! # 使用示例
//! ```ignore
//! use crate::services::storage_service::StorageService;
//! use crate::storage::MemoryStorage;
//!
//! let backend = MemoryStorage::new();
//! let service = StorageService::new(backend);
//!
//! service.put("key", b"value").await?;
//! let data = service.get("key").await?;
//! ```

use crate::services::registry::{HealthStatus, ServiceError, ServiceLifecycle, ServiceState};
use crate::storage::{StorageBackend, StorageResult, StorageStats};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 存储服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageServiceConfig {
    /// 服务名称
    pub name: String,
    /// 默认命名空间
    pub default_namespace: String,
    /// 启用指标收集
    pub enable_metrics: bool,
    /// 健康检查间隔（秒）
    pub health_check_interval_secs: u64,
}

impl Default for StorageServiceConfig {
    fn default() -> Self {
        Self {
            name: "storage-service".to_string(),
            default_namespace: "default".to_string(),
            enable_metrics: true,
            health_check_interval_secs: 30,
        }
    }
}

/// 存储操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageOperation {
    /// 读取
    Read,
    /// 写入
    Write,
    /// 删除
    Delete,
    /// 列出
    List,
    /// 存在检查
    Exists,
}

/// 存储操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOpResult {
    /// 操作类型
    pub operation: StorageOperation,
    /// 操作键
    pub key: String,
    /// 是否成功
    pub success: bool,
    /// 耗时（微秒）
    pub duration_us: u64,
    /// 数据大小（字节）
    pub data_size: Option<usize>,
}

impl StorageOpResult {
    /// 创建成功结果
    pub fn success(operation: StorageOperation, key: impl Into<String>, duration_us: u64) -> Self {
        Self {
            operation,
            key: key.into(),
            success: true,
            duration_us,
            data_size: None,
        }
    }

    /// 创建失败结果
    pub fn failure(operation: StorageOperation, key: impl Into<String>, duration_us: u64) -> Self {
        Self {
            operation,
            key: key.into(),
            success: false,
            duration_us,
            data_size: None,
        }
    }

    /// 设置数据大小
    pub fn with_size(mut self, size: usize) -> Self {
        self.data_size = Some(size);
        self
    }
}

/// 存储服务
pub struct StorageService<B: StorageBackend> {
    /// 配置
    config: StorageServiceConfig,
    /// 存储后端
    backend: Arc<B>,
    /// 服务状态
    state: ServiceState,
    /// 操作计数
    op_counts: Arc<RwLock<OperationCounts>>,
}

/// 操作计数
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OperationCounts {
    /// 读取次数
    pub reads: u64,
    /// 写入次数
    pub writes: u64,
    /// 删除次数
    pub deletes: u64,
    /// 列出次数
    pub lists: u64,
    /// 存在检查次数
    pub exists_checks: u64,
    /// 成功次数
    pub successes: u64,
    /// 失败次数
    pub failures: u64,
}

impl<B: StorageBackend + 'static> StorageService<B> {
    /// 创建新的存储服务
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, StorageServiceConfig::default())
    }

    /// 带配置创建
    pub fn with_config(backend: B, config: StorageServiceConfig) -> Self {
        Self {
            config,
            backend: Arc::new(backend),
            state: ServiceState::Uninitialized,
            op_counts: Arc::new(RwLock::new(OperationCounts::default())),
        }
    }

    /// 构建完整的键（加上命名空间）
    fn build_key(&self, namespace: Option<&str>, key: &str) -> String {
        let ns = namespace.unwrap_or(&self.config.default_namespace);
        format!("{}:{}", ns, key)
    }

    /// 读取数据
    pub async fn get(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.get_with_namespace(None, key).await
    }

    /// 读取数据（带命名空间）
    pub async fn get_with_namespace(&self, namespace: Option<&str>, key: &str) -> StorageResult<Vec<u8>> {
        let full_key = self.build_key(namespace, key);
        let result = self.backend.read(&full_key).await;

        let mut counts = self.op_counts.write().await;
        counts.reads += 1;
        if result.is_ok() {
            counts.successes += 1;
        } else {
            counts.failures += 1;
        }

        result
    }

    /// 写入数据
    pub async fn put(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.put_with_namespace(None, key, data).await
    }

    /// 写入数据（带命名空间）
    pub async fn put_with_namespace(&self, namespace: Option<&str>, key: &str, data: &[u8]) -> StorageResult<()> {
        let full_key = self.build_key(namespace, key);
        let result = self.backend.write(&full_key, data).await;

        let mut counts = self.op_counts.write().await;
        counts.writes += 1;
        if result.is_ok() {
            counts.successes += 1;
        } else {
            counts.failures += 1;
        }

        result
    }

    /// 删除数据
    pub async fn delete(&self, key: &str) -> StorageResult<()> {
        self.delete_with_namespace(None, key).await
    }

    /// 删除数据（带命名空间）
    pub async fn delete_with_namespace(&self, namespace: Option<&str>, key: &str) -> StorageResult<()> {
        let full_key = self.build_key(namespace, key);
        let result = self.backend.delete(&full_key).await;

        let mut counts = self.op_counts.write().await;
        counts.deletes += 1;
        if result.is_ok() {
            counts.successes += 1;
        } else {
            counts.failures += 1;
        }

        result
    }

    /// 检查键是否存在
    pub async fn exists(&self, key: &str) -> StorageResult<bool> {
        self.exists_with_namespace(None, key).await
    }

    /// 检查键是否存在（带命名空间）
    pub async fn exists_with_namespace(&self, namespace: Option<&str>, key: &str) -> StorageResult<bool> {
        let full_key = self.build_key(namespace, key);
        let result = self.backend.exists(&full_key).await;

        let mut counts = self.op_counts.write().await;
        counts.exists_checks += 1;
        if result.is_ok() {
            counts.successes += 1;
        } else {
            counts.failures += 1;
        }

        result
    }

    /// 列出键
    pub async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        self.list_with_namespace(None, prefix).await
    }

    /// 列出键（带命名空间）
    pub async fn list_with_namespace(&self, namespace: Option<&str>, prefix: &str) -> StorageResult<Vec<String>> {
        let ns = namespace.unwrap_or(&self.config.default_namespace);
        let full_prefix = format!("{}:{}", ns, prefix);
        let result = self.backend.list(&full_prefix).await;

        let mut counts = self.op_counts.write().await;
        counts.lists += 1;
        if result.is_ok() {
            counts.successes += 1;
        } else {
            counts.failures += 1;
        }

        // 移除命名空间前缀
        result.map(|keys| {
            keys.into_iter()
                .filter_map(|k| k.strip_prefix(&format!("{}:", ns)).map(String::from))
                .collect()
        })
    }

    /// 获取后端统计
    pub fn backend_stats(&self) -> StorageStats {
        self.backend.stats()
    }

    /// 获取操作计数
    pub async fn operation_counts(&self) -> OperationCounts {
        self.op_counts.read().await.clone()
    }

    /// 获取配置
    pub fn config(&self) -> &StorageServiceConfig {
        &self.config
    }
}

#[async_trait]
impl<B: StorageBackend + 'static> ServiceLifecycle for StorageService<B> {
    fn name(&self) -> &str {
        &self.config.name
    }

    async fn init(&mut self) -> Result<(), ServiceError> {
        self.state = ServiceState::Stopped;
        Ok(())
    }

    async fn start(&mut self) -> Result<(), ServiceError> {
        self.state = ServiceState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), ServiceError> {
        self.state = ServiceState::Stopped;
        Ok(())
    }

    async fn health_check(&self) -> HealthStatus {
        // 尝试一个简单的存在性检查
        match self.backend.exists("__health_check__").await {
            Ok(_) => HealthStatus::healthy()
                .with_detail("backend", self.backend.name()),
            Err(e) => HealthStatus::unhealthy(format!("Backend error: {}", e)),
        }
    }

    fn state(&self) -> ServiceState {
        self.state
    }
}

/// 存储服务统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageServiceStats {
    /// 服务名称
    pub service_name: String,
    /// 服务状态
    pub state: ServiceState,
    /// 后端名称
    pub backend_name: String,
    /// 操作计数
    pub operations: OperationCounts,
    /// 后端统计
    pub backend_stats: StorageStatsSnapshot,
}

/// 后端统计快照（可序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatsSnapshot {
    pub reads: u64,
    pub writes: u64,
    pub deletes: u64,
    pub hits: u64,
    pub misses: u64,
    pub total_bytes: u64,
    pub key_count: usize,
    pub hit_rate: f64,
}

impl From<StorageStats> for StorageStatsSnapshot {
    fn from(stats: StorageStats) -> Self {
        Self {
            reads: stats.reads,
            writes: stats.writes,
            deletes: stats.deletes,
            hits: stats.hits,
            misses: stats.misses,
            total_bytes: stats.total_bytes,
            key_count: stats.key_count,
            hit_rate: stats.hit_rate(),
        }
    }
}

impl<B: StorageBackend + 'static> StorageService<B> {
    /// 获取完整统计
    pub async fn full_stats(&self) -> StorageServiceStats {
        StorageServiceStats {
            service_name: self.config.name.clone(),
            state: self.state,
            backend_name: self.backend.name().to_string(),
            operations: self.operation_counts().await,
            backend_stats: self.backend_stats().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[test]
    fn test_storage_service_config_default() {
        let config = StorageServiceConfig::default();
        assert_eq!(config.name, "storage-service");
        assert_eq!(config.default_namespace, "default");
        assert!(config.enable_metrics);
    }

    #[test]
    fn test_storage_op_result() {
        let result = StorageOpResult::success(StorageOperation::Read, "key1", 100);
        assert!(result.success);
        assert_eq!(result.operation, StorageOperation::Read);

        let result = StorageOpResult::failure(StorageOperation::Write, "key2", 50);
        assert!(!result.success);

        let result = result.with_size(1024);
        assert_eq!(result.data_size, Some(1024));
    }

    #[tokio::test]
    async fn test_storage_service_new() {
        let backend = MemoryStorage::new();
        let service = StorageService::new(backend);

        assert_eq!(service.state(), ServiceState::Uninitialized);
        assert_eq!(service.config().name, "storage-service");
    }

    #[tokio::test]
    async fn test_storage_service_put_get() {
        let backend = MemoryStorage::new();
        let service = StorageService::new(backend);

        service.put("key1", b"value1").await.unwrap();
        let data = service.get("key1").await.unwrap();

        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_storage_service_namespace() {
        let backend = MemoryStorage::new();
        let service = StorageService::new(backend);

        // 不同命名空间
        service.put_with_namespace(Some("ns1"), "key", b"value1").await.unwrap();
        service.put_with_namespace(Some("ns2"), "key", b"value2").await.unwrap();

        let data1 = service.get_with_namespace(Some("ns1"), "key").await.unwrap();
        let data2 = service.get_with_namespace(Some("ns2"), "key").await.unwrap();

        assert_eq!(data1, b"value1");
        assert_eq!(data2, b"value2");
    }

    #[tokio::test]
    async fn test_storage_service_delete() {
        let backend = MemoryStorage::new();
        let service = StorageService::new(backend);

        service.put("key1", b"value1").await.unwrap();
        assert!(service.exists("key1").await.unwrap());

        service.delete("key1").await.unwrap();
        assert!(!service.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_storage_service_list() {
        let backend = MemoryStorage::new();
        let service = StorageService::new(backend);

        service.put("prefix:key1", b"value1").await.unwrap();
        service.put("prefix:key2", b"value2").await.unwrap();
        service.put("other:key3", b"value3").await.unwrap();

        let keys = service.list("prefix:").await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_storage_service_operation_counts() {
        let backend = MemoryStorage::new();
        let service = StorageService::new(backend);

        service.put("key1", b"value1").await.unwrap();
        service.get("key1").await.unwrap();
        service.exists("key1").await.unwrap();

        let counts = service.operation_counts().await;
        assert_eq!(counts.writes, 1);
        assert_eq!(counts.reads, 1);
        assert_eq!(counts.exists_checks, 1);
        assert_eq!(counts.successes, 3);
    }

    #[tokio::test]
    async fn test_storage_service_lifecycle() {
        let backend = MemoryStorage::new();
        let mut service = StorageService::new(backend);

        assert_eq!(service.state(), ServiceState::Uninitialized);

        service.init().await.unwrap();
        assert_eq!(service.state(), ServiceState::Stopped);

        service.start().await.unwrap();
        assert_eq!(service.state(), ServiceState::Running);

        service.stop().await.unwrap();
        assert_eq!(service.state(), ServiceState::Stopped);
    }

    #[tokio::test]
    async fn test_storage_service_health_check() {
        let backend = MemoryStorage::new();
        let service = StorageService::new(backend);

        let status = service.health_check().await;
        assert!(status.healthy);
    }

    #[tokio::test]
    async fn test_storage_service_full_stats() {
        let backend = MemoryStorage::new();
        let service = StorageService::new(backend);

        service.put("key1", b"value1").await.unwrap();

        let stats = service.full_stats().await;
        assert_eq!(stats.service_name, "storage-service");
        assert_eq!(stats.operations.writes, 1);
    }
}
