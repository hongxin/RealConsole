//! 插件系统基础架构
//!
//! v1.105.0 新增：可扩展的插件系统
//!
//! # 功能特性
//! - 插件生命周期管理
//! - 插件注册与发现
//! - 插件配置管理
//! - 钩子系统
//!
//! # 架构设计
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           PluginManager                  │
//! ├─────────────────────────────────────────┤
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
//! │  │ Plugin  │  │ Plugin  │  │ Plugin  │ │
//! │  │   A     │  │   B     │  │   C     │ │
//! │  └────┬────┘  └────┬────┘  └────┬────┘ │
//! │       │            │            │       │
//! │  ┌────▼────────────▼────────────▼────┐ │
//! │  │           Hook System              │ │
//! │  └───────────────────────────────────┘ │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # 使用示例
//! ```ignore
//! use crate::plugin::{Plugin, PluginManager, PluginContext};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str { "my-plugin" }
//!     fn version(&self) -> &str { "1.0.0" }
//! }
//!
//! let mut manager = PluginManager::new();
//! manager.register(Box::new(MyPlugin));
//! ```

mod loader;
mod registry;

pub use loader::{PluginLoader, PluginLoaderConfig, PluginSource};
pub use registry::{PluginRegistry, PluginEntry, PluginFilter};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

/// 插件 ID
pub type PluginId = String;

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginState {
    /// 未加载
    #[default]
    Unloaded,
    /// 已加载
    Loaded,
    /// 已启用
    Enabled,
    /// 已禁用
    Disabled,
    /// 错误
    Error,
}

impl PluginState {
    /// 是否可以启用
    pub fn can_enable(&self) -> bool {
        matches!(self, PluginState::Loaded | PluginState::Disabled)
    }

    /// 是否可以禁用
    pub fn can_disable(&self) -> bool {
        matches!(self, PluginState::Enabled)
    }

    /// 是否活跃
    pub fn is_active(&self) -> bool {
        matches!(self, PluginState::Enabled)
    }
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件 ID
    pub id: PluginId,
    /// 插件名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 许可证
    pub license: Option<String>,
    /// 主页
    pub homepage: Option<String>,
    /// 依赖列表
    pub dependencies: Vec<PluginDependency>,
    /// 标签
    pub tags: Vec<String>,
}

impl PluginMetadata {
    /// 创建新的元数据
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: String::new(),
            author: String::new(),
            license: None,
            homepage: None,
            dependencies: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// 设置作者
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// 添加依赖
    pub fn with_dependency(mut self, dep: PluginDependency) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// 插件依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// 依赖的插件 ID
    pub plugin_id: PluginId,
    /// 版本要求
    pub version_req: String,
    /// 是否可选
    pub optional: bool,
}

impl PluginDependency {
    /// 创建必需依赖
    pub fn required(plugin_id: impl Into<String>, version_req: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            version_req: version_req.into(),
            optional: false,
        }
    }

    /// 创建可选依赖
    pub fn optional(plugin_id: impl Into<String>, version_req: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            version_req: version_req.into(),
            optional: true,
        }
    }
}

/// 插件配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 配置项
    pub settings: HashMap<String, serde_json::Value>,
}

impl PluginConfig {
    /// 创建空配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置配置项
    pub fn set(&mut self, key: impl Into<String>, value: impl Serialize) {
        if let Ok(v) = serde_json::to_value(value) {
            self.settings.insert(key.into(), v);
        }
    }

    /// 获取配置项
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.settings.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 检查配置项是否存在
    pub fn contains(&self, key: &str) -> bool {
        self.settings.contains_key(key)
    }
}

/// 插件上下文
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// 插件 ID
    pub plugin_id: PluginId,
    /// 配置
    pub config: PluginConfig,
    /// 数据目录
    pub data_dir: Option<String>,
    /// 环境变量
    pub env: HashMap<String, String>,
}

impl PluginContext {
    /// 创建新的上下文
    pub fn new(plugin_id: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            config: PluginConfig::new(),
            data_dir: None,
            env: HashMap::new(),
        }
    }

    /// 设置配置
    pub fn with_config(mut self, config: PluginConfig) -> Self {
        self.config = config;
        self
    }

    /// 设置数据目录
    pub fn with_data_dir(mut self, dir: impl Into<String>) -> Self {
        self.data_dir = Some(dir.into());
        self
    }

    /// 设置环境变量
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

/// 插件 trait
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 获取插件元数据
    fn metadata(&self) -> &PluginMetadata;

    /// 初始化插件
    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let _ = ctx;
        Ok(())
    }

    /// 启动插件
    async fn start(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// 停止插件
    async fn stop(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// 卸载前的清理
    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// 健康检查
    async fn health_check(&self) -> PluginHealth {
        PluginHealth::healthy()
    }

    /// 获取插件能力
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![]
    }

    /// 转换为 Any（用于向下转型）
    fn as_any(&self) -> &dyn Any;
}

/// 插件错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginError {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 插件 ID
    pub plugin_id: Option<PluginId>,
}

impl PluginError {
    /// 创建新错误
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            plugin_id: None,
        }
    }

    /// 设置插件 ID
    pub fn with_plugin(mut self, plugin_id: impl Into<String>) -> Self {
        self.plugin_id = Some(plugin_id.into());
        self
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref id) = self.plugin_id {
            write!(f, "[{}][{}] {}", id, self.code, self.message)
        } else {
            write!(f, "[{}] {}", self.code, self.message)
        }
    }
}

impl std::error::Error for PluginError {}

/// 插件健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHealth {
    /// 是否健康
    pub healthy: bool,
    /// 状态消息
    pub message: String,
    /// 检查时间
    pub checked_at: DateTime<Utc>,
}

impl PluginHealth {
    /// 创建健康状态
    pub fn healthy() -> Self {
        Self {
            healthy: true,
            message: "OK".to_string(),
            checked_at: Utc::now(),
        }
    }

    /// 创建不健康状态
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            healthy: false,
            message: message.into(),
            checked_at: Utc::now(),
        }
    }
}

impl Default for PluginHealth {
    fn default() -> Self {
        Self::healthy()
    }
}

/// 插件能力
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    /// 提供命令
    Commands,
    /// 提供工具
    Tools,
    /// 提供钩子
    Hooks,
    /// 提供存储
    Storage,
    /// 提供 UI
    Ui,
    /// 自定义能力
    Custom(String),
}

/// 钩子点
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookPoint {
    /// 命令执行前
    BeforeCommand,
    /// 命令执行后
    AfterCommand,
    /// LLM 调用前
    BeforeLlm,
    /// LLM 调用后
    AfterLlm,
    /// 会话开始
    SessionStart,
    /// 会话结束
    SessionEnd,
    /// 自定义钩子点
    Custom(String),
}

/// 钩子处理结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum HookResult {
    /// 继续执行
    #[default]
    Continue,
    /// 跳过后续钩子
    Skip,
    /// 终止执行
    Abort(String),
    /// 修改数据
    Modify(serde_json::Value),
}

/// 插件管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManagerConfig {
    /// 插件目录
    pub plugin_dir: String,
    /// 启用的插件列表
    pub enabled_plugins: Vec<PluginId>,
    /// 禁用的插件列表
    pub disabled_plugins: Vec<PluginId>,
    /// 自动加载
    pub auto_load: bool,
    /// 严格模式（依赖检查）
    pub strict_mode: bool,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            plugin_dir: "~/.realconsole/plugins".to_string(),
            enabled_plugins: Vec::new(),
            disabled_plugins: Vec::new(),
            auto_load: true,
            strict_mode: false,
        }
    }
}

/// 插件管理器
pub struct PluginManager {
    /// 配置
    config: PluginManagerConfig,
    /// 插件注册表
    registry: PluginRegistry,
    /// 加载器
    loader: PluginLoader,
    /// 钩子映射
    hooks: HashMap<HookPoint, Vec<PluginId>>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Self::with_config(PluginManagerConfig::default())
    }

    /// 带配置创建
    pub fn with_config(config: PluginManagerConfig) -> Self {
        Self {
            config,
            registry: PluginRegistry::new(),
            loader: PluginLoader::new(),
            hooks: HashMap::new(),
        }
    }

    /// 注册插件
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<PluginId, PluginError> {
        let metadata = plugin.metadata().clone();
        let plugin_id = metadata.id.clone();

        self.registry.register(plugin)?;

        // 注册钩子
        for cap in self.registry.get(&plugin_id)
            .map(|p| p.plugin.capabilities())
            .unwrap_or_default()
        {
            if cap == PluginCapability::Hooks {
                // 默认注册到所有钩子点
                for point in [
                    HookPoint::BeforeCommand,
                    HookPoint::AfterCommand,
                    HookPoint::BeforeLlm,
                    HookPoint::AfterLlm,
                ] {
                    self.hooks.entry(point).or_default().push(plugin_id.clone());
                }
            }
        }

        Ok(plugin_id)
    }

    /// 卸载插件
    pub async fn unregister(&mut self, plugin_id: &PluginId) -> Result<(), PluginError> {
        // 先停止插件
        if let Some(entry) = self.registry.get_mut(plugin_id) {
            entry.plugin.stop().await?;
            entry.plugin.cleanup().await?;
        }

        // 从钩子中移除
        for plugins in self.hooks.values_mut() {
            plugins.retain(|id| id != plugin_id);
        }

        self.registry.unregister(plugin_id)
    }

    /// 启用插件
    pub async fn enable(&mut self, plugin_id: &PluginId, ctx: &PluginContext) -> Result<(), PluginError> {
        let entry = self.registry.get_mut(plugin_id)
            .ok_or_else(|| PluginError::new("NOT_FOUND", format!("Plugin {} not found", plugin_id)))?;

        if !entry.state.can_enable() {
            return Err(PluginError::new("INVALID_STATE", "Plugin cannot be enabled in current state"));
        }

        entry.plugin.init(ctx).await?;
        entry.plugin.start().await?;
        entry.state = PluginState::Enabled;
        entry.enabled_at = Some(Utc::now());

        Ok(())
    }

    /// 禁用插件
    pub async fn disable(&mut self, plugin_id: &PluginId) -> Result<(), PluginError> {
        let entry = self.registry.get_mut(plugin_id)
            .ok_or_else(|| PluginError::new("NOT_FOUND", format!("Plugin {} not found", plugin_id)))?;

        if !entry.state.can_disable() {
            return Err(PluginError::new("INVALID_STATE", "Plugin cannot be disabled in current state"));
        }

        entry.plugin.stop().await?;
        entry.state = PluginState::Disabled;

        Ok(())
    }

    /// 获取插件
    pub fn get(&self, plugin_id: &PluginId) -> Option<&PluginEntry> {
        self.registry.get(plugin_id)
    }

    /// 列出所有插件
    pub fn list(&self) -> Vec<PluginInfo> {
        self.registry.list()
    }

    /// 列出已启用的插件
    pub fn list_enabled(&self) -> Vec<PluginInfo> {
        self.registry.filter(PluginFilter::State(PluginState::Enabled))
    }

    /// 执行钩子
    pub async fn run_hook(&self, point: &HookPoint, data: serde_json::Value) -> HookResult {
        let plugins = match self.hooks.get(point) {
            Some(p) => p,
            None => return HookResult::Continue,
        };

        let current_data = data;

        for plugin_id in plugins {
            if let Some(entry) = self.registry.get(plugin_id) {
                if entry.state != PluginState::Enabled {
                    continue;
                }

                // 这里简化处理，实际应该调用插件的钩子方法
                // 目前只返回继续
            }
        }

        HookResult::Modify(current_data)
    }

    /// 获取配置
    pub fn config(&self) -> &PluginManagerConfig {
        &self.config
    }

    /// 获取插件数量
    pub fn count(&self) -> usize {
        self.registry.count()
    }

    /// 获取已启用插件数量
    pub fn enabled_count(&self) -> usize {
        self.registry.filter(PluginFilter::State(PluginState::Enabled)).len()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 插件信息（轻量级）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件 ID
    pub id: PluginId,
    /// 名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 描述
    pub description: String,
    /// 状态
    pub state: PluginState,
    /// 能力
    pub capabilities: Vec<PluginCapability>,
    /// 启用时间
    pub enabled_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试插件实现
    struct TestPlugin {
        metadata: PluginMetadata,
    }

    impl TestPlugin {
        fn new(id: &str) -> Self {
            Self {
                metadata: PluginMetadata::new(id, format!("Test Plugin {}", id), "1.0.0"),
            }
        }
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        fn capabilities(&self) -> Vec<PluginCapability> {
            vec![PluginCapability::Commands]
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_plugin_state_methods() {
        assert!(PluginState::Loaded.can_enable());
        assert!(PluginState::Disabled.can_enable());
        assert!(!PluginState::Enabled.can_enable());

        assert!(PluginState::Enabled.can_disable());
        assert!(!PluginState::Disabled.can_disable());

        assert!(PluginState::Enabled.is_active());
        assert!(!PluginState::Disabled.is_active());
    }

    #[test]
    fn test_plugin_metadata() {
        let metadata = PluginMetadata::new("test-plugin", "Test Plugin", "1.0.0")
            .with_description("A test plugin")
            .with_author("Test Author")
            .with_tag("test");

        assert_eq!(metadata.id, "test-plugin");
        assert_eq!(metadata.name, "Test Plugin");
        assert_eq!(metadata.description, "A test plugin");
        assert_eq!(metadata.author, "Test Author");
        assert!(metadata.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_plugin_dependency() {
        let required = PluginDependency::required("dep-1", ">=1.0.0");
        assert!(!required.optional);

        let optional = PluginDependency::optional("dep-2", ">=2.0.0");
        assert!(optional.optional);
    }

    #[test]
    fn test_plugin_config() {
        let mut config = PluginConfig::new();
        config.set("key1", "value1");
        config.set("key2", 42);

        assert_eq!(config.get::<String>("key1"), Some("value1".to_string()));
        assert_eq!(config.get::<i32>("key2"), Some(42));
        assert!(config.contains("key1"));
        assert!(!config.contains("key3"));
    }

    #[test]
    fn test_plugin_context() {
        let ctx = PluginContext::new("test-plugin")
            .with_data_dir("/tmp/test")
            .with_env("KEY", "VALUE");

        assert_eq!(ctx.plugin_id, "test-plugin");
        assert_eq!(ctx.data_dir, Some("/tmp/test".to_string()));
        assert_eq!(ctx.env.get("KEY"), Some(&"VALUE".to_string()));
    }

    #[test]
    fn test_plugin_error() {
        let error = PluginError::new("TEST_ERROR", "Test error message")
            .with_plugin("test-plugin");

        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.plugin_id, Some("test-plugin".to_string()));
        assert!(error.to_string().contains("test-plugin"));
    }

    #[test]
    fn test_plugin_health() {
        let healthy = PluginHealth::healthy();
        assert!(healthy.healthy);

        let unhealthy = PluginHealth::unhealthy("Something wrong");
        assert!(!unhealthy.healthy);
        assert_eq!(unhealthy.message, "Something wrong");
    }

    #[test]
    fn test_plugin_manager_config_default() {
        let config = PluginManagerConfig::default();
        assert!(config.auto_load);
        assert!(!config.strict_mode);
    }

    #[test]
    fn test_plugin_manager_new() {
        let manager = PluginManager::new();
        assert_eq!(manager.count(), 0);
        assert_eq!(manager.enabled_count(), 0);
    }

    #[test]
    fn test_plugin_manager_register() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new("test-1"));

        let result = manager.register(plugin);
        assert!(result.is_ok());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_plugin_manager_list() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test-1"))).unwrap();
        manager.register(Box::new(TestPlugin::new("test-2"))).unwrap();

        let list = manager.list();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_plugin_manager_enable_disable() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test-1"))).unwrap();

        let ctx = PluginContext::new("test-1");
        manager.enable(&"test-1".to_string(), &ctx).await.unwrap();
        assert_eq!(manager.enabled_count(), 1);

        manager.disable(&"test-1".to_string()).await.unwrap();
        assert_eq!(manager.enabled_count(), 0);
    }
}
