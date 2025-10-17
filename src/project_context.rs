//! 项目上下文感知模块
//!
//! 自动检测当前目录的项目类型，提供上下文感知的智能建议。
//!
//! 支持的项目类型：
//! - Rust (Cargo.toml)
//! - Python (requirements.txt, pyproject.toml, setup.py)
//! - Node.js (package.json)
//! - Go (go.mod)
//! - Java (pom.xml, build.gradle)

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// 项目类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Rust 项目
    Rust {
        cargo_toml: PathBuf,
        has_src: bool,
        has_tests: bool,
    },
    /// Python 项目
    Python {
        requirements: Option<PathBuf>,
        pyproject: Option<PathBuf>,
        setup_py: Option<PathBuf>,
    },
    /// Node.js 项目
    Node {
        package_json: PathBuf,
        has_node_modules: bool,
    },
    /// Go 项目
    Go {
        go_mod: PathBuf,
        has_go_sum: bool,
    },
    /// Java 项目
    Java {
        build_file: JavaBuildFile,
    },
    /// 未知项目类型
    Unknown,
}

/// Java 构建文件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JavaBuildFile {
    Maven(PathBuf),      // pom.xml
    Gradle(PathBuf),     // build.gradle
    GradleKts(PathBuf),  // build.gradle.kts
}

/// Git 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub has_git: bool,
    pub current_branch: Option<String>,
    pub is_dirty: bool,
    pub remote_url: Option<String>,
}

/// 项目上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// 项目根目录
    pub root: PathBuf,
    /// 项目类型
    pub project_type: ProjectType,
    /// Git 信息
    pub git_info: Option<GitInfo>,
}

impl ProjectContext {
    /// 检测当前目录的项目上下文
    pub fn detect() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::detect_in_directory(&root)
    }

    /// 在指定目录检测项目上下文
    pub fn detect_in_directory(dir: &Path) -> Self {
        let project_type = Self::detect_project_type(dir);
        let git_info = Self::detect_git_info(dir);

        Self {
            root: dir.to_path_buf(),
            project_type,
            git_info,
        }
    }

    /// 检测项目类型
    fn detect_project_type(dir: &Path) -> ProjectType {
        // 按优先级检测

        // Rust
        if let Some(cargo_toml) = Self::find_file(dir, "Cargo.toml") {
            return ProjectType::Rust {
                cargo_toml,
                has_src: dir.join("src").exists(),
                has_tests: dir.join("tests").exists(),
            };
        }

        // Node.js
        if let Some(package_json) = Self::find_file(dir, "package.json") {
            return ProjectType::Node {
                package_json,
                has_node_modules: dir.join("node_modules").exists(),
            };
        }

        // Go
        if let Some(go_mod) = Self::find_file(dir, "go.mod") {
            return ProjectType::Go {
                go_mod: go_mod.clone(),
                has_go_sum: Self::find_file(dir, "go.sum").is_some(),
            };
        }

        // Python
        let requirements = Self::find_file(dir, "requirements.txt");
        let pyproject = Self::find_file(dir, "pyproject.toml");
        let setup_py = Self::find_file(dir, "setup.py");

        if requirements.is_some() || pyproject.is_some() || setup_py.is_some() {
            return ProjectType::Python {
                requirements,
                pyproject,
                setup_py,
            };
        }

        // Java
        if let Some(pom_xml) = Self::find_file(dir, "pom.xml") {
            return ProjectType::Java {
                build_file: JavaBuildFile::Maven(pom_xml),
            };
        }

        if let Some(build_gradle) = Self::find_file(dir, "build.gradle") {
            return ProjectType::Java {
                build_file: JavaBuildFile::Gradle(build_gradle),
            };
        }

        if let Some(build_gradle_kts) = Self::find_file(dir, "build.gradle.kts") {
            return ProjectType::Java {
                build_file: JavaBuildFile::GradleKts(build_gradle_kts),
            };
        }

        ProjectType::Unknown
    }

    /// 检测 Git 信息
    fn detect_git_info(dir: &Path) -> Option<GitInfo> {
        let git_dir = dir.join(".git");
        if !git_dir.exists() {
            return None;
        }

        // 获取当前分支
        let current_branch = Self::get_git_branch(dir);

        // 检查是否有未提交的变更
        let is_dirty = Self::is_git_dirty(dir);

        // 获取远程 URL
        let remote_url = Self::get_git_remote_url(dir);

        Some(GitInfo {
            has_git: true,
            current_branch,
            is_dirty,
            remote_url,
        })
    }

    /// 查找文件
    fn find_file(dir: &Path, filename: &str) -> Option<PathBuf> {
        let path = dir.join(filename);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// 获取 Git 分支名
    fn get_git_branch(dir: &Path) -> Option<String> {
        use std::process::Command;

        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(dir)
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Some(branch)
        } else {
            None
        }
    }

    /// 检查 Git 是否有未提交的变更
    fn is_git_dirty(dir: &Path) -> bool {
        use std::process::Command;

        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(dir)
            .output();

        if let Ok(output) = output {
            !output.stdout.is_empty()
        } else {
            false
        }
    }

    /// 获取 Git 远程 URL
    fn get_git_remote_url(dir: &Path) -> Option<String> {
        use std::process::Command;

        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(dir)
            .output()
            .ok()?;

        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Some(url)
        } else {
            None
        }
    }

    /// 获取项目名称
    pub fn project_name(&self) -> String {
        self.root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// 获取项目类型描述
    pub fn type_description(&self) -> &str {
        match &self.project_type {
            ProjectType::Rust { .. } => "Rust",
            ProjectType::Python { .. } => "Python",
            ProjectType::Node { .. } => "Node.js",
            ProjectType::Go { .. } => "Go",
            ProjectType::Java { .. } => "Java",
            ProjectType::Unknown => "Unknown",
        }
    }

    /// 获取构建命令建议
    pub fn build_command(&self) -> Option<&str> {
        match &self.project_type {
            ProjectType::Rust { .. } => Some("cargo build"),
            ProjectType::Node { .. } => Some("npm run build"),
            ProjectType::Go { .. } => Some("go build"),
            ProjectType::Java { build_file } => match build_file {
                JavaBuildFile::Maven(_) => Some("mvn compile"),
                JavaBuildFile::Gradle(_) | JavaBuildFile::GradleKts(_) => Some("./gradlew build"),
            },
            _ => None,
        }
    }

    /// 获取测试命令建议
    pub fn test_command(&self) -> Option<&str> {
        match &self.project_type {
            ProjectType::Rust { .. } => Some("cargo test"),
            ProjectType::Python { .. } => Some("pytest"),
            ProjectType::Node { .. } => Some("npm test"),
            ProjectType::Go { .. } => Some("go test ./..."),
            ProjectType::Java { build_file } => match build_file {
                JavaBuildFile::Maven(_) => Some("mvn test"),
                JavaBuildFile::Gradle(_) | JavaBuildFile::GradleKts(_) => Some("./gradlew test"),
            },
            _ => None,
        }
    }

    /// 获取运行命令建议
    pub fn run_command(&self) -> Option<&str> {
        match &self.project_type {
            ProjectType::Rust { .. } => Some("cargo run"),
            ProjectType::Python { .. } => Some("python main.py"),
            ProjectType::Node { .. } => Some("npm start"),
            ProjectType::Go { .. } => Some("go run ."),
            ProjectType::Java { build_file } => match build_file {
                JavaBuildFile::Maven(_) => Some("mvn exec:java"),
                JavaBuildFile::Gradle(_) | JavaBuildFile::GradleKts(_) => Some("./gradlew run"),
            },
            _ => None,
        }
    }

    /// 是否为识别的项目类型
    pub fn is_recognized(&self) -> bool {
        !matches!(self.project_type, ProjectType::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_detect_rust_project() {
        // 在临时目录创建 Cargo.toml
        let temp_dir = std::env::temp_dir().join("test_rust_project");
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        fs::create_dir_all(temp_dir.join("src")).unwrap();

        let context = ProjectContext::detect_in_directory(&temp_dir);

        assert!(matches!(context.project_type, ProjectType::Rust { .. }));
        assert_eq!(context.type_description(), "Rust");
        assert_eq!(context.build_command(), Some("cargo build"));
        assert_eq!(context.test_command(), Some("cargo test"));
        assert_eq!(context.run_command(), Some("cargo run"));

        // 清理
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_detect_node_project() {
        let temp_dir = std::env::temp_dir().join("test_node_project");
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("package.json"), "{}").unwrap();

        let context = ProjectContext::detect_in_directory(&temp_dir);

        assert!(matches!(context.project_type, ProjectType::Node { .. }));
        assert_eq!(context.type_description(), "Node.js");

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_detect_unknown_project() {
        let temp_dir = std::env::temp_dir().join("test_unknown_project");
        fs::create_dir_all(&temp_dir).unwrap();

        let context = ProjectContext::detect_in_directory(&temp_dir);

        assert!(matches!(context.project_type, ProjectType::Unknown));
        assert_eq!(context.type_description(), "Unknown");
        assert!(!context.is_recognized());

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_project_name() {
        let temp_dir = std::env::temp_dir().join("my-awesome-project");
        fs::create_dir_all(&temp_dir).unwrap();

        let context = ProjectContext::detect_in_directory(&temp_dir);

        assert_eq!(context.project_name(), "my-awesome-project");

        fs::remove_dir_all(&temp_dir).ok();
    }
}
