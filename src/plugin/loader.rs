//! 插件加载器
//!
//! 负责从不同来源加载插件

use serde::{Deserialize, Serialize};

/// 插件来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginSource {
    /// 本地文件
    Local { path: String },
    /// 内置插件
    Builtin { name: String },
    /// 远程 URL
    Remote { url: String },
}

/// 加载器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLoaderConfig {
    /// 插件搜索路径
    pub search_paths: Vec<String>,
    /// 允许远程加载
    pub allow_remote: bool,
    /// 缓存目录
    pub cache_dir: Option<String>,
}

impl Default for PluginLoaderConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![
                "~/.realconsole/plugins".to_string(),
                "/usr/local/share/realconsole/plugins".to_string(),
            ],
            allow_remote: false,
            cache_dir: Some("~/.realconsole/plugin-cache".to_string()),
        }
    }
}

/// 插件加载器
pub struct PluginLoader {
    /// 配置
    config: PluginLoaderConfig,
}

impl PluginLoader {
    /// 创建新的加载器
    pub fn new() -> Self {
        Self::with_config(PluginLoaderConfig::default())
    }

    /// 带配置创建
    pub fn with_config(config: PluginLoaderConfig) -> Self {
        Self { config }
    }

    /// 获取配置
    pub fn config(&self) -> &PluginLoaderConfig {
        &self.config
    }

    /// 发现可用插件
    pub fn discover(&self) -> Vec<PluginSource> {
        let mut sources = Vec::new();

        // 扫描搜索路径
        for path in &self.config.search_paths {
            let expanded = shellexpand::tilde(path);
            if let Ok(entries) = std::fs::read_dir(expanded.as_ref()) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        sources.push(PluginSource::Local {
                            path: entry.path().to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }

        sources
    }

    /// 验证插件来源
    pub fn validate(&self, source: &PluginSource) -> Result<(), String> {
        match source {
            PluginSource::Local { path } => {
                let expanded = shellexpand::tilde(path);
                if !std::path::Path::new(expanded.as_ref()).exists() {
                    return Err(format!("Plugin path does not exist: {}", path));
                }
                Ok(())
            }
            PluginSource::Builtin { .. } => Ok(()),
            PluginSource::Remote { .. } => {
                if !self.config.allow_remote {
                    return Err("Remote plugin loading is disabled".to_string());
                }
                Ok(())
            }
        }
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_source() {
        let local = PluginSource::Local { path: "/tmp/plugin".to_string() };
        let builtin = PluginSource::Builtin { name: "core".to_string() };
        let remote = PluginSource::Remote { url: "https://example.com/plugin".to_string() };

        // 只是验证可以创建
        assert!(matches!(local, PluginSource::Local { .. }));
        assert!(matches!(builtin, PluginSource::Builtin { .. }));
        assert!(matches!(remote, PluginSource::Remote { .. }));
    }

    #[test]
    fn test_loader_config_default() {
        let config = PluginLoaderConfig::default();
        assert!(!config.allow_remote);
        assert!(!config.search_paths.is_empty());
    }

    #[test]
    fn test_plugin_loader_new() {
        let loader = PluginLoader::new();
        assert!(!loader.config().allow_remote);
    }

    #[test]
    fn test_validate_builtin() {
        let loader = PluginLoader::new();
        let source = PluginSource::Builtin { name: "core".to_string() };

        let result = loader.validate(&source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_remote_disabled() {
        let loader = PluginLoader::new();
        let source = PluginSource::Remote { url: "https://example.com".to_string() };

        let result = loader.validate(&source);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("disabled"));
    }
}
