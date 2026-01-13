//! 服务注册中心
//!
//! v1.104.0 新增：统一服务管理和生命周期控制
//!
//! # 功能特性
//! - 服务注册与发现
//! - 生命周期管理（启动/停止/健康检查）
//! - 依赖注入支持
//! - 服务状态监控
//!
//! # 使用示例
//! ```ignore
//! use crate::services::registry::{ServiceRegistry, ServiceDescriptor};
//!
//! let mut registry = ServiceRegistry::new();
//! registry.register("my-service", service_instance);
//! registry.start_all().await;
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 服务 ID
pub type ServiceId = String;

/// 服务状态（一分为三扩展）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceState {
    /// 未初始化
    #[default]
    Uninitialized,
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
    /// 错误
    Error,
}

impl ServiceState {
    /// 是否可以启动
    pub fn can_start(&self) -> bool {
        matches!(self, ServiceState::Uninitialized | ServiceState::Stopped)
    }

    /// 是否可以停止
    pub fn can_stop(&self) -> bool {
        matches!(self, ServiceState::Running | ServiceState::Paused | ServiceState::Error)
    }

    /// 是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(self, ServiceState::Running)
    }

    /// 是否健康
    pub fn is_healthy(&self) -> bool {
        matches!(self, ServiceState::Running | ServiceState::Paused)
    }
}

/// 服务生命周期 trait
#[async_trait]
pub trait ServiceLifecycle: Send + Sync {
    /// 服务名称
    fn name(&self) -> &str;

    /// 初始化服务
    async fn init(&mut self) -> Result<(), ServiceError>;

    /// 启动服务
    async fn start(&mut self) -> Result<(), ServiceError>;

    /// 停止服务
    async fn stop(&mut self) -> Result<(), ServiceError>;

    /// 健康检查
    async fn health_check(&self) -> HealthStatus;

    /// 获取服务状态
    fn state(&self) -> ServiceState;

    /// 获取服务依赖
    fn dependencies(&self) -> Vec<ServiceId> {
        vec![]
    }
}

/// 服务错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceError {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 是否可重试
    pub retryable: bool,
}

impl ServiceError {
    /// 创建新错误
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable: false,
        }
    }

    /// 设置可重试
    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ServiceError {}

/// 健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// 是否健康
    pub healthy: bool,
    /// 状态消息
    pub message: String,
    /// 检查时间
    pub checked_at: DateTime<Utc>,
    /// 详细信息
    pub details: HashMap<String, String>,
}

impl HealthStatus {
    /// 创建健康状态
    pub fn healthy() -> Self {
        Self {
            healthy: true,
            message: "OK".to_string(),
            checked_at: Utc::now(),
            details: HashMap::new(),
        }
    }

    /// 创建不健康状态
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            healthy: false,
            message: message.into(),
            checked_at: Utc::now(),
            details: HashMap::new(),
        }
    }

    /// 添加详细信息
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::healthy()
    }
}

/// 服务描述符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescriptor {
    /// 服务 ID
    pub id: ServiceId,
    /// 服务名称
    pub name: String,
    /// 服务版本
    pub version: String,
    /// 服务状态
    pub state: ServiceState,
    /// 依赖列表
    pub dependencies: Vec<ServiceId>,
    /// 注册时间
    pub registered_at: DateTime<Utc>,
    /// 启动时间
    pub started_at: Option<DateTime<Utc>>,
    /// 最后健康检查
    pub last_health_check: Option<HealthStatus>,
}

impl ServiceDescriptor {
    /// 创建新描述符
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            state: ServiceState::Uninitialized,
            dependencies: Vec::new(),
            registered_at: Utc::now(),
            started_at: None,
            last_health_check: None,
        }
    }

    /// 添加依赖
    pub fn with_dependencies(mut self, deps: Vec<ServiceId>) -> Self {
        self.dependencies = deps;
        self
    }
}

/// 服务注册事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RegistryEvent {
    /// 服务注册
    ServiceRegistered { service_id: ServiceId },
    /// 服务注销
    ServiceUnregistered { service_id: ServiceId },
    /// 服务启动
    ServiceStarted { service_id: ServiceId },
    /// 服务停止
    ServiceStopped { service_id: ServiceId },
    /// 服务错误
    ServiceError { service_id: ServiceId, error: ServiceError },
    /// 健康检查完成
    HealthCheckCompleted { service_id: ServiceId, status: HealthStatus },
}

/// 服务包装器（用于存储 trait object）
struct ServiceWrapper {
    descriptor: ServiceDescriptor,
    instance: Arc<RwLock<dyn ServiceLifecycle>>,
}

/// 服务注册中心
pub struct ServiceRegistry {
    /// 注册的服务
    services: HashMap<ServiceId, ServiceWrapper>,
    /// 启动顺序
    start_order: Vec<ServiceId>,
    /// 事件回调
    event_handlers: Vec<Box<dyn Fn(RegistryEvent) + Send + Sync>>,
}

impl ServiceRegistry {
    /// 创建新的注册中心
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            start_order: Vec::new(),
            event_handlers: Vec::new(),
        }
    }

    /// 注册服务
    pub fn register<S: ServiceLifecycle + 'static>(
        &mut self,
        id: impl Into<String>,
        version: impl Into<String>,
        service: S,
    ) -> Result<(), ServiceError> {
        let id = id.into();
        let name = service.name().to_string();
        let deps = service.dependencies();

        if self.services.contains_key(&id) {
            return Err(ServiceError::new("DUPLICATE", format!("Service {} already registered", id)));
        }

        let descriptor = ServiceDescriptor::new(id.clone(), name, version)
            .with_dependencies(deps);

        let wrapper = ServiceWrapper {
            descriptor,
            instance: Arc::new(RwLock::new(service)),
        };

        self.services.insert(id.clone(), wrapper);
        self.emit_event(RegistryEvent::ServiceRegistered { service_id: id });

        Ok(())
    }

    /// 注销服务
    pub async fn unregister(&mut self, id: &ServiceId) -> Result<(), ServiceError> {
        if let Some(wrapper) = self.services.get(id) {
            let mut service = wrapper.instance.write().await;
            if service.state().can_stop() {
                service.stop().await?;
            }
        }

        self.services.remove(id);
        self.start_order.retain(|s| s != id);
        self.emit_event(RegistryEvent::ServiceUnregistered { service_id: id.clone() });

        Ok(())
    }

    /// 获取服务描述符
    pub fn get_descriptor(&self, id: &ServiceId) -> Option<ServiceDescriptor> {
        self.services.get(id).map(|w| w.descriptor.clone())
    }

    /// 获取所有服务描述符
    pub fn list_services(&self) -> Vec<ServiceDescriptor> {
        self.services.values().map(|w| w.descriptor.clone()).collect()
    }

    /// 启动单个服务
    pub async fn start_service(&mut self, id: &ServiceId) -> Result<(), ServiceError> {
        let wrapper = self.services.get(id)
            .ok_or_else(|| ServiceError::new("NOT_FOUND", format!("Service {} not found", id)))?;

        // 检查依赖是否都已启动
        for dep_id in &wrapper.descriptor.dependencies {
            if let Some(dep) = self.services.get(dep_id) {
                if !dep.descriptor.state.is_running() {
                    return Err(ServiceError::new(
                        "DEPENDENCY_NOT_RUNNING",
                        format!("Dependency {} not running", dep_id),
                    ));
                }
            } else {
                return Err(ServiceError::new(
                    "DEPENDENCY_NOT_FOUND",
                    format!("Dependency {} not found", dep_id),
                ));
            }
        }

        let mut service = wrapper.instance.write().await;

        // 初始化（如果需要）
        if service.state() == ServiceState::Uninitialized {
            service.init().await?;
        }

        // 启动
        service.start().await?;

        // 更新描述符
        drop(service);
        if let Some(wrapper) = self.services.get_mut(id) {
            wrapper.descriptor.state = ServiceState::Running;
            wrapper.descriptor.started_at = Some(Utc::now());
        }

        if !self.start_order.contains(id) {
            self.start_order.push(id.clone());
        }

        self.emit_event(RegistryEvent::ServiceStarted { service_id: id.clone() });
        Ok(())
    }

    /// 停止单个服务
    pub async fn stop_service(&mut self, id: &ServiceId) -> Result<(), ServiceError> {
        let wrapper = self.services.get(id)
            .ok_or_else(|| ServiceError::new("NOT_FOUND", format!("Service {} not found", id)))?;

        let mut service = wrapper.instance.write().await;
        service.stop().await?;

        drop(service);
        if let Some(wrapper) = self.services.get_mut(id) {
            wrapper.descriptor.state = ServiceState::Stopped;
        }

        self.emit_event(RegistryEvent::ServiceStopped { service_id: id.clone() });
        Ok(())
    }

    /// 按依赖顺序启动所有服务
    pub async fn start_all(&mut self) -> Result<(), ServiceError> {
        let order = self.resolve_start_order()?;

        for id in order {
            self.start_service(&id).await?;
        }

        Ok(())
    }

    /// 按逆序停止所有服务
    pub async fn stop_all(&mut self) -> Result<(), ServiceError> {
        let order: Vec<_> = self.start_order.iter().rev().cloned().collect();

        for id in order {
            if let Err(e) = self.stop_service(&id).await {
                // 记录错误但继续停止其他服务
                self.emit_event(RegistryEvent::ServiceError {
                    service_id: id,
                    error: e,
                });
            }
        }

        Ok(())
    }

    /// 解析启动顺序（拓扑排序）
    fn resolve_start_order(&self) -> Result<Vec<ServiceId>, ServiceError> {
        let mut in_degree: HashMap<ServiceId, usize> = HashMap::new();
        let mut dependents: HashMap<ServiceId, Vec<ServiceId>> = HashMap::new();

        // 初始化
        for (id, wrapper) in &self.services {
            in_degree.insert(id.clone(), wrapper.descriptor.dependencies.len());

            for dep_id in &wrapper.descriptor.dependencies {
                dependents.entry(dep_id.clone()).or_default().push(id.clone());
            }
        }

        // 找出入度为 0 的服务
        let mut queue: Vec<ServiceId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(id) = queue.pop() {
            result.push(id.clone());

            if let Some(deps) = dependents.get(&id) {
                for dep_id in deps {
                    if let Some(degree) = in_degree.get_mut(dep_id) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(dep_id.clone());
                        }
                    }
                }
            }
        }

        if result.len() != self.services.len() {
            return Err(ServiceError::new(
                "CIRCULAR_DEPENDENCY",
                "Circular dependency detected in services",
            ));
        }

        Ok(result)
    }

    /// 执行健康检查
    pub async fn health_check(&mut self, id: &ServiceId) -> Result<HealthStatus, ServiceError> {
        let wrapper = self.services.get(id)
            .ok_or_else(|| ServiceError::new("NOT_FOUND", format!("Service {} not found", id)))?;

        let service = wrapper.instance.read().await;
        let status = service.health_check().await;

        drop(service);
        if let Some(wrapper) = self.services.get_mut(id) {
            wrapper.descriptor.last_health_check = Some(status.clone());
        }

        self.emit_event(RegistryEvent::HealthCheckCompleted {
            service_id: id.clone(),
            status: status.clone(),
        });

        Ok(status)
    }

    /// 执行所有服务的健康检查
    pub async fn health_check_all(&mut self) -> HashMap<ServiceId, HealthStatus> {
        let ids: Vec<_> = self.services.keys().cloned().collect();
        let mut results = HashMap::new();

        for id in ids {
            if let Ok(status) = self.health_check(&id).await {
                results.insert(id, status);
            }
        }

        results
    }

    /// 注册事件处理器
    pub fn on_event<F: Fn(RegistryEvent) + Send + Sync + 'static>(&mut self, handler: F) {
        self.event_handlers.push(Box::new(handler));
    }

    /// 发送事件
    fn emit_event(&self, event: RegistryEvent) {
        for handler in &self.event_handlers {
            handler(event.clone());
        }
    }

    /// 获取运行中的服务数量
    pub fn running_count(&self) -> usize {
        self.services.values().filter(|w| w.descriptor.state.is_running()).count()
    }

    /// 获取总服务数量
    pub fn total_count(&self) -> usize {
        self.services.len()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 注册表统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// 总服务数
    pub total_services: usize,
    /// 运行中服务数
    pub running_services: usize,
    /// 停止服务数
    pub stopped_services: usize,
    /// 错误服务数
    pub error_services: usize,
    /// 健康服务数
    pub healthy_services: usize,
}

impl ServiceRegistry {
    /// 获取注册表统计
    pub fn stats(&self) -> RegistryStats {
        let total = self.services.len();
        let running = self.services.values().filter(|w| w.descriptor.state == ServiceState::Running).count();
        let stopped = self.services.values().filter(|w| w.descriptor.state == ServiceState::Stopped).count();
        let error = self.services.values().filter(|w| w.descriptor.state == ServiceState::Error).count();
        let healthy = self.services.values().filter(|w| {
            w.descriptor.last_health_check.as_ref().map(|h| h.healthy).unwrap_or(false)
        }).count();

        RegistryStats {
            total_services: total,
            running_services: running,
            stopped_services: stopped,
            error_services: error,
            healthy_services: healthy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试服务实现
    struct TestService {
        name: String,
        state: ServiceState,
        deps: Vec<ServiceId>,
    }

    impl TestService {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                state: ServiceState::Uninitialized,
                deps: vec![],
            }
        }

        fn with_deps(mut self, deps: Vec<&str>) -> Self {
            self.deps = deps.into_iter().map(String::from).collect();
            self
        }
    }

    #[async_trait]
    impl ServiceLifecycle for TestService {
        fn name(&self) -> &str {
            &self.name
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
            if self.state == ServiceState::Running {
                HealthStatus::healthy()
            } else {
                HealthStatus::unhealthy("Not running")
            }
        }

        fn state(&self) -> ServiceState {
            self.state
        }

        fn dependencies(&self) -> Vec<ServiceId> {
            self.deps.clone()
        }
    }

    #[test]
    fn test_service_state_methods() {
        assert!(ServiceState::Uninitialized.can_start());
        assert!(ServiceState::Stopped.can_start());
        assert!(!ServiceState::Running.can_start());

        assert!(ServiceState::Running.can_stop());
        assert!(!ServiceState::Stopped.can_stop());

        assert!(ServiceState::Running.is_running());
        assert!(!ServiceState::Stopped.is_running());
    }

    #[test]
    fn test_service_error() {
        let error = ServiceError::new("TEST", "Test error");
        assert_eq!(error.code, "TEST");
        assert!(!error.retryable);

        let error = error.retryable();
        assert!(error.retryable);
    }

    #[test]
    fn test_health_status() {
        let healthy = HealthStatus::healthy();
        assert!(healthy.healthy);

        let unhealthy = HealthStatus::unhealthy("Error message");
        assert!(!unhealthy.healthy);
        assert_eq!(unhealthy.message, "Error message");

        let with_detail = healthy.with_detail("key", "value");
        assert_eq!(with_detail.details.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_service_descriptor() {
        let desc = ServiceDescriptor::new("svc-1", "TestService", "1.0.0")
            .with_dependencies(vec!["dep-1".to_string()]);

        assert_eq!(desc.id, "svc-1");
        assert_eq!(desc.name, "TestService");
        assert_eq!(desc.version, "1.0.0");
        assert_eq!(desc.dependencies.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let registry = ServiceRegistry::new();
        assert_eq!(registry.total_count(), 0);
        assert_eq!(registry.running_count(), 0);
    }

    #[tokio::test]
    async fn test_registry_register() {
        let mut registry = ServiceRegistry::new();
        let service = TestService::new("test");

        let result = registry.register("test-1", "1.0.0", service);
        assert!(result.is_ok());
        assert_eq!(registry.total_count(), 1);
    }

    #[tokio::test]
    async fn test_registry_duplicate_register() {
        let mut registry = ServiceRegistry::new();

        registry.register("test-1", "1.0.0", TestService::new("test")).unwrap();
        let result = registry.register("test-1", "1.0.0", TestService::new("test2"));

        assert!(result.is_err());
        assert!(result.unwrap_err().code.contains("DUPLICATE"));
    }

    #[tokio::test]
    async fn test_registry_start_service() {
        let mut registry = ServiceRegistry::new();
        registry.register("test-1", "1.0.0", TestService::new("test")).unwrap();

        let result = registry.start_service(&"test-1".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(registry.running_count(), 1);
    }

    #[tokio::test]
    async fn test_registry_stop_service() {
        let mut registry = ServiceRegistry::new();
        registry.register("test-1", "1.0.0", TestService::new("test")).unwrap();
        registry.start_service(&"test-1".to_string()).await.unwrap();

        let result = registry.stop_service(&"test-1".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(registry.running_count(), 0);
    }

    #[tokio::test]
    async fn test_registry_dependency_order() {
        let mut registry = ServiceRegistry::new();

        // 注册有依赖关系的服务
        registry.register("base", "1.0.0", TestService::new("base")).unwrap();
        registry.register("dependent", "1.0.0", TestService::new("dependent").with_deps(vec!["base"])).unwrap();

        // 启动所有服务（应该先启动 base）
        let result = registry.start_all().await;
        assert!(result.is_ok());
        assert_eq!(registry.running_count(), 2);
    }

    #[tokio::test]
    async fn test_registry_health_check() {
        let mut registry = ServiceRegistry::new();
        registry.register("test-1", "1.0.0", TestService::new("test")).unwrap();
        registry.start_service(&"test-1".to_string()).await.unwrap();

        let status = registry.health_check(&"test-1".to_string()).await.unwrap();
        assert!(status.healthy);
    }

    #[tokio::test]
    async fn test_registry_stats() {
        let mut registry = ServiceRegistry::new();
        registry.register("test-1", "1.0.0", TestService::new("test1")).unwrap();
        registry.register("test-2", "1.0.0", TestService::new("test2")).unwrap();
        registry.start_service(&"test-1".to_string()).await.unwrap();

        let stats = registry.stats();
        assert_eq!(stats.total_services, 2);
        assert_eq!(stats.running_services, 1);
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let mut registry = ServiceRegistry::new();
        registry.register("test-1", "1.0.0", TestService::new("test")).unwrap();

        let result = registry.unregister(&"test-1".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(registry.total_count(), 0);
    }
}
