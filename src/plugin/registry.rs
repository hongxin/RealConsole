//! 插件注册表
//!
//! 管理已注册的插件及其状态

use super::{Plugin, PluginError, PluginId, PluginInfo, PluginState};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 插件条目
pub struct PluginEntry {
    /// 插件实例
    pub plugin: Box<dyn Plugin>,
    /// 状态
    pub state: PluginState,
    /// 注册时间
    pub registered_at: DateTime<Utc>,
    /// 启用时间
    pub enabled_at: Option<DateTime<Utc>>,
}

impl PluginEntry {
    /// 创建新条目
    pub fn new(plugin: Box<dyn Plugin>) -> Self {
        Self {
            plugin,
            state: PluginState::Loaded,
            registered_at: Utc::now(),
            enabled_at: None,
        }
    }

    /// 转换为信息
    pub fn to_info(&self) -> PluginInfo {
        let metadata = self.plugin.metadata();
        PluginInfo {
            id: metadata.id.clone(),
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            description: metadata.description.clone(),
            state: self.state,
            capabilities: self.plugin.capabilities(),
            enabled_at: self.enabled_at,
        }
    }
}

/// 插件过滤器
#[derive(Debug, Clone)]
pub enum PluginFilter {
    /// 按状态过滤
    State(PluginState),
    /// 按标签过滤
    Tag(String),
    /// 按能力过滤
    Capability(super::PluginCapability),
    /// 组合过滤（AND）
    And(Vec<PluginFilter>),
    /// 组合过滤（OR）
    Or(Vec<PluginFilter>),
}

impl PluginFilter {
    /// 检查插件是否匹配
    pub fn matches(&self, entry: &PluginEntry) -> bool {
        match self {
            PluginFilter::State(state) => entry.state == *state,
            PluginFilter::Tag(tag) => entry.plugin.metadata().tags.contains(tag),
            PluginFilter::Capability(cap) => entry.plugin.capabilities().contains(cap),
            PluginFilter::And(filters) => filters.iter().all(|f| f.matches(entry)),
            PluginFilter::Or(filters) => filters.iter().any(|f| f.matches(entry)),
        }
    }
}

/// 插件注册表
pub struct PluginRegistry {
    /// 已注册的插件
    plugins: HashMap<PluginId, PluginEntry>,
}

impl PluginRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// 注册插件
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<PluginId, PluginError> {
        let plugin_id = plugin.metadata().id.clone();

        if self.plugins.contains_key(&plugin_id) {
            return Err(PluginError::new(
                "DUPLICATE",
                format!("Plugin {} already registered", plugin_id),
            ).with_plugin(&plugin_id));
        }

        let entry = PluginEntry::new(plugin);
        self.plugins.insert(plugin_id.clone(), entry);

        Ok(plugin_id)
    }

    /// 注销插件
    pub fn unregister(&mut self, plugin_id: &PluginId) -> Result<(), PluginError> {
        self.plugins.remove(plugin_id).ok_or_else(|| {
            PluginError::new("NOT_FOUND", format!("Plugin {} not found", plugin_id))
        })?;
        Ok(())
    }

    /// 获取插件
    pub fn get(&self, plugin_id: &PluginId) -> Option<&PluginEntry> {
        self.plugins.get(plugin_id)
    }

    /// 获取插件（可变）
    pub fn get_mut(&mut self, plugin_id: &PluginId) -> Option<&mut PluginEntry> {
        self.plugins.get_mut(plugin_id)
    }

    /// 检查插件是否存在
    pub fn contains(&self, plugin_id: &PluginId) -> bool {
        self.plugins.contains_key(plugin_id)
    }

    /// 列出所有插件
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.values().map(|e| e.to_info()).collect()
    }

    /// 按过滤器列出插件
    pub fn filter(&self, filter: PluginFilter) -> Vec<PluginInfo> {
        self.plugins
            .values()
            .filter(|e| filter.matches(e))
            .map(|e| e.to_info())
            .collect()
    }

    /// 获取插件数量
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// 清空注册表
    pub fn clear(&mut self) {
        self.plugins.clear();
    }

    /// 获取所有插件 ID
    pub fn ids(&self) -> Vec<PluginId> {
        self.plugins.keys().cloned().collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{PluginCapability, PluginMetadata};
    use async_trait::async_trait;
    use std::any::Any;

    struct MockPlugin {
        metadata: PluginMetadata,
        capabilities: Vec<PluginCapability>,
    }

    impl MockPlugin {
        fn new(id: &str) -> Self {
            Self {
                metadata: PluginMetadata::new(id, format!("Mock {}", id), "1.0.0"),
                capabilities: vec![],
            }
        }

        fn with_capability(mut self, cap: PluginCapability) -> Self {
            self.capabilities.push(cap);
            self
        }

        fn with_tag(mut self, tag: &str) -> Self {
            self.metadata.tags.push(tag.to_string());
            self
        }
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        fn capabilities(&self) -> Vec<PluginCapability> {
            self.capabilities.clone()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_plugin_entry_new() {
        let plugin = Box::new(MockPlugin::new("test"));
        let entry = PluginEntry::new(plugin);

        assert_eq!(entry.state, PluginState::Loaded);
        assert!(entry.enabled_at.is_none());
    }

    #[test]
    fn test_plugin_entry_to_info() {
        let plugin = Box::new(MockPlugin::new("test"));
        let entry = PluginEntry::new(plugin);
        let info = entry.to_info();

        assert_eq!(info.id, "test");
        assert_eq!(info.state, PluginState::Loaded);
    }

    #[test]
    fn test_plugin_registry_new() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_plugin_registry_register() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(MockPlugin::new("test-1"));

        let result = registry.register(plugin);
        assert!(result.is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_plugin_registry_duplicate() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(MockPlugin::new("test-1"))).unwrap();

        let result = registry.register(Box::new(MockPlugin::new("test-1")));
        assert!(result.is_err());
        assert!(result.unwrap_err().code.contains("DUPLICATE"));
    }

    #[test]
    fn test_plugin_registry_unregister() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(MockPlugin::new("test-1"))).unwrap();

        let result = registry.unregister(&"test-1".to_string());
        assert!(result.is_ok());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_plugin_registry_get() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(MockPlugin::new("test-1"))).unwrap();

        assert!(registry.get(&"test-1".to_string()).is_some());
        assert!(registry.get(&"nonexistent".to_string()).is_none());
    }

    #[test]
    fn test_plugin_registry_list() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(MockPlugin::new("test-1"))).unwrap();
        registry.register(Box::new(MockPlugin::new("test-2"))).unwrap();

        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_plugin_filter_state() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(MockPlugin::new("test-1"))).unwrap();

        let filter = PluginFilter::State(PluginState::Loaded);
        let results = registry.filter(filter);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_plugin_filter_capability() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(
            MockPlugin::new("test-1").with_capability(PluginCapability::Commands)
        )).unwrap();
        registry.register(Box::new(MockPlugin::new("test-2"))).unwrap();

        let filter = PluginFilter::Capability(PluginCapability::Commands);
        let results = registry.filter(filter);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_plugin_filter_tag() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(MockPlugin::new("test-1").with_tag("core"))).unwrap();
        registry.register(Box::new(MockPlugin::new("test-2"))).unwrap();

        let filter = PluginFilter::Tag("core".to_string());
        let results = registry.filter(filter);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_plugin_filter_and() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(
            MockPlugin::new("test-1")
                .with_capability(PluginCapability::Commands)
                .with_tag("core")
        )).unwrap();
        registry.register(Box::new(
            MockPlugin::new("test-2").with_tag("core")
        )).unwrap();

        let filter = PluginFilter::And(vec![
            PluginFilter::Tag("core".to_string()),
            PluginFilter::Capability(PluginCapability::Commands),
        ]);
        let results = registry.filter(filter);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_plugin_registry_ids() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(MockPlugin::new("test-1"))).unwrap();
        registry.register(Box::new(MockPlugin::new("test-2"))).unwrap();

        let ids = registry.ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"test-1".to_string()));
        assert!(ids.contains(&"test-2".to_string()));
    }

    #[test]
    fn test_plugin_registry_clear() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(MockPlugin::new("test-1"))).unwrap();
        registry.register(Box::new(MockPlugin::new("test-2"))).unwrap();

        registry.clear();
        assert_eq!(registry.count(), 0);
    }
}
