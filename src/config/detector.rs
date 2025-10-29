// Environment Detector - 智能环境检测
//
// 检测用户的操作系统、Shell、已安装工具等信息，
// 为配置向导提供智能推荐的基础

use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// 操作系统信息
#[derive(Debug, Clone)]
pub struct OsInfo {
    pub os_type: String,      // "macos", "linux", "windows"
    pub version: String,      // "14.0", "Ubuntu 22.04", etc.
    pub arch: String,         // "x86_64", "arm64", etc.
}

/// Shell 信息
#[derive(Debug, Clone)]
pub struct ShellInfo {
    pub shell_type: String,   // "zsh", "bash", "fish", etc.
    pub shell_path: PathBuf,  // "/bin/zsh", etc.
    pub version: Option<String>,
}

/// 已安装工具
#[derive(Debug, Clone)]
pub struct InstalledTool {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
}

/// 用户画像
#[derive(Debug, Clone, PartialEq)]
pub enum UserProfile {
    Developer,        // 软件开发者（检测到git, 编程语言工具）
    DevOps,          // 运维工程师（检测到docker, k8s, ansible等）
    Student,         // 学生/学习者（环境简单）
    Unknown,         // 未知（默认）
}

/// 完整的环境信息
#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    pub os: OsInfo,
    pub shell: ShellInfo,
    pub tools: Vec<InstalledTool>,
    pub user_profile: UserProfile,
    pub home_dir: PathBuf,
    pub config_dir: PathBuf,
}

/// 环境检测器
pub struct EnvironmentDetector;

impl EnvironmentDetector {
    /// 创建环境检测器
    pub fn new() -> Self {
        Self
    }

    /// 检测所有环境信息
    pub fn detect_all(&self) -> Result<EnvironmentInfo> {
        let os = self.detect_os()?;
        let shell = self.detect_shell()?;
        let tools = self.detect_tools();
        let user_profile = self.detect_user_profile(&tools);
        let home_dir = dirs::home_dir()
            .context("Failed to get home directory")?;
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| home_dir.join(".config"))
            .join("realconsole");

        Ok(EnvironmentInfo {
            os,
            shell,
            tools,
            user_profile,
            home_dir,
            config_dir,
        })
    }

    /// 检测操作系统
    pub fn detect_os(&self) -> Result<OsInfo> {
        let os_type = if cfg!(target_os = "macos") {
            "macos".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "unknown".to_string()
        };

        let arch = env::consts::ARCH.to_string();

        // 检测版本（简化版）
        let version = if cfg!(target_os = "macos") {
            self.run_command("sw_vers", &["-productVersion"])
                .unwrap_or_else(|_| "unknown".to_string())
        } else if cfg!(target_os = "linux") {
            // 尝试读取 /etc/os-release
            std::fs::read_to_string("/etc/os-release")
                .ok()
                .and_then(|content| {
                    content.lines()
                        .find(|line| line.starts_with("PRETTY_NAME="))
                        .map(|line| line.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
                })
                .unwrap_or_else(|| "Linux".to_string())
        } else {
            "unknown".to_string()
        };

        Ok(OsInfo {
            os_type,
            version,
            arch,
        })
    }

    /// 检测 Shell
    pub fn detect_shell(&self) -> Result<ShellInfo> {
        // 从环境变量获取 SHELL
        let shell_path = env::var("SHELL")
            .context("SHELL environment variable not found")?;
        let shell_path = PathBuf::from(shell_path);

        let shell_type = shell_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 尝试获取版本
        let version = self.run_command(&shell_type, &["--version"])
            .ok()
            .and_then(|output| {
                // 提取第一行作为版本信息
                output.lines().next().map(|s| s.to_string())
            });

        Ok(ShellInfo {
            shell_type,
            shell_path,
            version,
        })
    }

    /// 检测已安装工具
    pub fn detect_tools(&self) -> Vec<InstalledTool> {
        let tool_names = vec![
            "git", "cargo", "rustc", "npm", "node", "python", "python3",
            "docker", "kubectl", "ansible", "terraform", "vim", "nvim",
            "code", "make", "gcc", "clang",
        ];

        tool_names
            .into_iter()
            .filter_map(|name| self.detect_single_tool(name))
            .collect()
    }

    /// 检测单个工具
    fn detect_single_tool(&self, name: &str) -> Option<InstalledTool> {
        // 使用 which 命令查找工具路径
        let output = Command::new("which")
            .arg(name)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let path = String::from_utf8_lossy(&output.stdout);
        let path = PathBuf::from(path.trim());

        // 尝试获取版本
        let version = self.run_command(name, &["--version"])
            .ok()
            .or_else(|| self.run_command(name, &["-v"]).ok())
            .or_else(|| self.run_command(name, &["-version"]).ok())
            .and_then(|output| {
                // 提取第一行作为版本信息
                output.lines().next().map(|s| s.to_string())
            });

        Some(InstalledTool {
            name: name.to_string(),
            path,
            version,
        })
    }

    /// 检测用户画像
    pub fn detect_user_profile(&self, tools: &[InstalledTool]) -> UserProfile {
        let tool_names: Vec<&str> = tools.iter()
            .map(|t| t.name.as_str())
            .collect();

        // 开发者特征：git + 编程语言工具
        let has_git = tool_names.contains(&"git");
        let has_programming_tools = tool_names.contains(&"cargo")
            || tool_names.contains(&"npm")
            || tool_names.contains(&"python")
            || tool_names.contains(&"python3");

        // DevOps特征：容器/编排工具
        let has_devops_tools = tool_names.contains(&"docker")
            || tool_names.contains(&"kubectl")
            || tool_names.contains(&"ansible")
            || tool_names.contains(&"terraform");

        if has_devops_tools {
            UserProfile::DevOps
        } else if has_git && has_programming_tools {
            UserProfile::Developer
        } else if has_git || has_programming_tools {
            UserProfile::Student
        } else {
            UserProfile::Unknown
        }
    }

    /// 运行命令并获取输出
    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(cmd)
            .args(args)
            .output()
            .with_context(|| format!("Failed to execute command: {} {:?}", cmd, args))?;

        if !output.status.success() {
            anyhow::bail!("Command failed: {} {:?}", cmd, args);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Default for EnvironmentDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_os() {
        let detector = EnvironmentDetector::new();
        let os = detector.detect_os().unwrap();

        assert!(!os.os_type.is_empty());
        assert!(!os.arch.is_empty());
        println!("OS: {} {}", os.os_type, os.version);
    }

    #[test]
    fn test_detect_shell() {
        let detector = EnvironmentDetector::new();
        let shell = detector.detect_shell().unwrap();

        assert!(!shell.shell_type.is_empty());
        assert!(shell.shell_path.exists());
        println!("Shell: {} at {:?}", shell.shell_type, shell.shell_path);
    }

    #[test]
    fn test_detect_tools() {
        let detector = EnvironmentDetector::new();
        let tools = detector.detect_tools();

        assert!(!tools.is_empty());
        println!("Found {} tools", tools.len());
        for tool in tools.iter().take(5) {
            println!("  - {}: {:?}", tool.name, tool.version);
        }
    }

    #[test]
    fn test_detect_all() {
        let detector = EnvironmentDetector::new();
        let env = detector.detect_all().unwrap();

        assert!(!env.os.os_type.is_empty());
        assert!(!env.shell.shell_type.is_empty());
        println!("Environment detected:");
        println!("  OS: {} {}", env.os.os_type, env.os.version);
        println!("  Shell: {}", env.shell.shell_type);
        println!("  Tools: {}", env.tools.len());
        println!("  Profile: {:?}", env.user_profile);
    }

    #[test]
    fn test_user_profile_detection() {
        let detector = EnvironmentDetector::new();

        // Test Developer profile
        let dev_tools = vec![
            InstalledTool {
                name: "git".to_string(),
                path: PathBuf::from("/usr/bin/git"),
                version: Some("2.39.0".to_string()),
            },
            InstalledTool {
                name: "cargo".to_string(),
                path: PathBuf::from("/usr/bin/cargo"),
                version: Some("1.70.0".to_string()),
            },
        ];
        let profile = detector.detect_user_profile(&dev_tools);
        assert_eq!(profile, UserProfile::Developer);

        // Test DevOps profile
        let devops_tools = vec![
            InstalledTool {
                name: "docker".to_string(),
                path: PathBuf::from("/usr/bin/docker"),
                version: Some("24.0.0".to_string()),
            },
            InstalledTool {
                name: "kubectl".to_string(),
                path: PathBuf::from("/usr/bin/kubectl"),
                version: Some("1.28.0".to_string()),
            },
        ];
        let profile = detector.detect_user_profile(&devops_tools);
        assert_eq!(profile, UserProfile::DevOps);
    }
}
