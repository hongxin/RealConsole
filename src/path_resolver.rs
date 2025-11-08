//! 路径解析器
//!
//! 提供统一的配置文件路径搜索策略，支持多个搜索位置：
//! 1. 当前工作目录
//! 2. 用户配置目录 (~/.realconsole/)
//!
//! 用于：
//! - realconsole.yaml 配置文件
//! - locales/*.yaml 语言文件
//! - .env 环境变量文件
//! - 其他配置文件

use std::path::{Path, PathBuf};

/// 路径解析器 - 提供统一的配置文件搜索策略
pub struct PathResolver;

impl PathResolver {
    /// 获取用户配置目录（~/.realconsole/）
    pub fn user_config_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".realconsole"))
    }

    /// 在多个位置搜索文件
    ///
    /// 搜索顺序：
    /// 1. 当前工作目录
    /// 2. 用户配置目录 (~/.realconsole/)
    ///
    /// # 参数
    /// - `filename`: 文件名（相对路径），如 "realconsole.yaml" 或 "locales/zh-CN.yaml"
    ///
    /// # 返回
    /// - `Some(PathBuf)`: 找到的第一个存在的文件路径
    /// - `None`: 未找到文件
    pub fn resolve(filename: &str) -> Option<PathBuf> {
        // 1. 当前工作目录
        let cwd_path = PathBuf::from(filename);
        if cwd_path.exists() {
            return Some(cwd_path);
        }

        // 2. 用户配置目录
        if let Some(user_dir) = Self::user_config_dir() {
            let user_path = user_dir.join(filename);
            if user_path.exists() {
                return Some(user_path);
            }
        }

        None
    }

    /// 解析配置文件路径（支持显式路径和自动搜索）
    ///
    /// # 参数
    /// - `path_or_filename`: 可以是绝对路径、相对路径或文件名
    ///
    /// # 行为
    /// - 如果是绝对路径：直接返回（不验证是否存在）
    /// - 如果是相对路径/文件名：按搜索策略查找
    ///
    /// # 示例
    /// ```ignore
    /// // 显式路径（不搜索）
    /// PathResolver::resolve_config("/etc/realconsole.yaml");  // 返回 /etc/realconsole.yaml
    ///
    /// // 文件名（自动搜索）
    /// PathResolver::resolve_config("realconsole.yaml");       // 搜索 ./ 和 ~/.realconsole/
    /// ```
    pub fn resolve_config(path_or_filename: &str) -> Option<PathBuf> {
        let path = Path::new(path_or_filename);

        // 如果是绝对路径，直接返回（用户显式指定）
        if path.is_absolute() {
            return Some(path.to_path_buf());
        }

        // 相对路径或文件名：进行搜索
        Self::resolve(path_or_filename)
    }

    /// 获取所有可能的搜索路径（用于调试和提示）
    ///
    /// # 参数
    /// - `filename`: 文件名
    ///
    /// # 返回
    /// 按搜索顺序返回所有可能的路径（无论是否存在）
    pub fn search_paths(filename: &str) -> Vec<PathBuf> {
        let mut paths = vec![
            // 1. 当前工作目录
            PathBuf::from(filename),
        ];

        // 2. 用户配置目录
        if let Some(user_dir) = Self::user_config_dir() {
            paths.push(user_dir.join(filename));
        }

        paths
    }

    /// 确保用户配置目录存在
    ///
    /// # 返回
    /// - `Ok(PathBuf)`: 配置目录路径
    /// - `Err(String)`: 创建失败的错误信息
    pub fn ensure_user_config_dir() -> Result<PathBuf, String> {
        let config_dir = Self::user_config_dir().ok_or("无法获取用户主目录")?;

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        Ok(config_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_config_dir() {
        let config_dir = PathResolver::user_config_dir();
        assert!(config_dir.is_some());
        if let Some(dir) = config_dir {
            assert!(dir.ends_with(".realconsole"));
        }
    }

    #[test]
    fn test_resolve_absolute_path() {
        let result = PathResolver::resolve_config("/etc/realconsole.yaml");
        assert_eq!(result, Some(PathBuf::from("/etc/realconsole.yaml")));
    }

    #[test]
    fn test_search_paths() {
        let paths = PathResolver::search_paths("realconsole.yaml");
        assert!(!paths.is_empty()); // 至少有当前目录

        // 第一个路径应该是当前目录
        assert_eq!(paths[0], PathBuf::from("realconsole.yaml"));

        // 如果有用户目录，第二个路径应该在 ~/.realconsole/ 下
        if paths.len() > 1 {
            assert!(paths[1].ends_with(".realconsole/realconsole.yaml"));
        }
    }

    #[test]
    fn test_search_paths_with_subdirs() {
        let paths = PathResolver::search_paths("locales/zh-CN.yaml");

        // 第一个路径：当前目录
        assert_eq!(paths[0], PathBuf::from("locales/zh-CN.yaml"));

        // 第二个路径：用户配置目录（如果存在）
        if paths.len() > 1 {
            assert!(paths[1].ends_with(".realconsole/locales/zh-CN.yaml"));
        }
    }
}
