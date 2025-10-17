//! Git 智能助手
//!
//! 提供智能化的 Git 操作支持：
//! - 自动生成提交信息
//! - 分支管理
//! - 变更分析
//! - 冲突解决建议

use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

/// Git 仓库状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    /// 当前分支
    pub current_branch: Option<String>,
    /// 是否有未提交的变更
    pub has_changes: bool,
    /// 暂存区文件数
    pub staged_files: usize,
    /// 未暂存文件数
    pub unstaged_files: usize,
    /// 未跟踪文件数
    pub untracked_files: usize,
    /// 远程仓库 URL
    pub remote_url: Option<String>,
    /// 领先远程分支的提交数
    pub ahead: usize,
    /// 落后远程分支的提交数
    pub behind: usize,
}

/// 提交信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMessage {
    /// 提交类型 (feat, fix, docs, etc.)
    pub commit_type: String,
    /// 作用域 (可选)
    pub scope: Option<String>,
    /// 简短描述
    pub subject: String,
    /// 详细描述 (可选)
    pub body: Option<String>,
    /// 关联的 Issue (可选)
    pub footer: Option<String>,
}

impl CommitMessage {
    /// 格式化为 Conventional Commits 格式
    pub fn format(&self) -> String {
        let mut lines = vec![];

        // 标题行
        let header = if let Some(scope) = &self.scope {
            format!("{}({}): {}", self.commit_type, scope, self.subject)
        } else {
            format!("{}: {}", self.commit_type, self.subject)
        };
        lines.push(header);

        // 空行
        if self.body.is_some() || self.footer.is_some() {
            lines.push(String::new());
        }

        // 详细描述
        if let Some(body) = &self.body {
            lines.push(body.clone());
        }

        // Footer
        if let Some(footer) = &self.footer {
            if self.body.is_some() {
                lines.push(String::new());
            }
            lines.push(footer.clone());
        }

        lines.join("\n")
    }
}

/// Git 仓库管理器
pub struct GitRepository {
    /// 仓库根目录
    root: std::path::PathBuf,
}

impl GitRepository {
    /// 创建新的 Git 仓库管理器
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let root = path.as_ref().to_path_buf();

        // 检查是否是 Git 仓库
        if !root.join(".git").exists() {
            return Err("不是 Git 仓库".to_string());
        }

        Ok(Self { root })
    }

    /// 获取当前目录的 Git 仓库
    pub fn current() -> Result<Self, String> {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("无法获取当前目录: {}", e))?;

        Self::new(current_dir)
    }

    /// 获取 Git 状态
    pub fn status(&self) -> Result<GitStatus, String> {
        let current_branch = self.get_current_branch()?;
        let remote_url = self.get_remote_url().ok();

        // 获取 git status --porcelain 输出
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("执行 git status 失败: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git status 失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let status_output = String::from_utf8_lossy(&output.stdout);

        let mut staged_files = 0;
        let mut unstaged_files = 0;
        let mut untracked_files = 0;

        for line in status_output.lines() {
            if line.len() < 3 {
                continue;
            }
            let index_status = &line[0..1];
            let worktree_status = &line[1..2];

            // 暂存区状态
            if index_status != " " && index_status != "?" {
                staged_files += 1;
            }

            // 工作区状态
            if worktree_status != " " && worktree_status != "?" {
                unstaged_files += 1;
            }

            // 未跟踪文件
            if index_status == "?" {
                untracked_files += 1;
            }
        }

        let has_changes = staged_files > 0 || unstaged_files > 0 || untracked_files > 0;

        // 获取领先/落后信息
        let (ahead, behind) = self.get_ahead_behind().unwrap_or((0, 0));

        Ok(GitStatus {
            current_branch,
            has_changes,
            staged_files,
            unstaged_files,
            untracked_files,
            remote_url,
            ahead,
            behind,
        })
    }

    /// 获取当前分支名
    pub fn get_current_branch(&self) -> Result<Option<String>, String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("执行 git branch 失败: {}", e))?;

        if !output.status.success() {
            return Ok(None);
        }

        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Some(branch))
    }

    /// 获取远程仓库 URL
    pub fn get_remote_url(&self) -> Result<String, String> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("执行 git remote 失败: {}", e))?;

        if !output.status.success() {
            return Err("没有配置远程仓库".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// 获取领先/落后信息
    pub fn get_ahead_behind(&self) -> Result<(usize, usize), String> {
        let output = Command::new("git")
            .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("执行 git rev-list 失败: {}", e))?;

        if !output.status.success() {
            // 没有上游分支
            return Ok((0, 0));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().split_whitespace().collect();

        if parts.len() != 2 {
            return Ok((0, 0));
        }

        let ahead = parts[0].parse::<usize>().unwrap_or(0);
        let behind = parts[1].parse::<usize>().unwrap_or(0);

        Ok((ahead, behind))
    }

    /// 获取 git diff
    pub fn get_diff(&self, staged: bool) -> Result<String, String> {
        let mut args = vec!["diff"];
        if staged {
            args.push("--cached");
        }

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("执行 git diff 失败: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git diff 失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 获取简短的 diff 统计
    pub fn get_diff_stat(&self, staged: bool) -> Result<String, String> {
        let mut args = vec!["diff", "--stat"];
        if staged {
            args.push("--cached");
        }

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("执行 git diff --stat 失败: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git diff --stat 失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 分析变更类型
    pub fn analyze_changes(&self, diff: &str) -> ChangeAnalysis {
        let mut analysis = ChangeAnalysis::default();

        for line in diff.lines() {
            if line.starts_with("+++") || line.starts_with("---") {
                // 文件路径
                if let Some(path) = line.split_whitespace().nth(1) {
                    if path != "/dev/null" {
                        let path = path.trim_start_matches("b/");

                        // 根据文件扩展名分类
                        if path.ends_with(".rs") {
                            analysis.rust_files += 1;
                        } else if path.ends_with(".md") || path.ends_with(".txt") {
                            analysis.doc_files += 1;
                        } else if path.ends_with(".toml") || path.ends_with(".yaml") || path.ends_with(".json") {
                            analysis.config_files += 1;
                        } else if path.ends_with(".sh") {
                            analysis.script_files += 1;
                        }
                    }
                }
            } else if line.starts_with("+") && !line.starts_with("+++") {
                analysis.additions += 1;

                // 检测特定模式
                if line.contains("fn ") || line.contains("impl ") {
                    analysis.has_new_functions = true;
                }
                if line.contains("struct ") || line.contains("enum ") {
                    analysis.has_new_types = true;
                }
                if line.contains("test") || line.contains("#[test]") {
                    analysis.has_tests = true;
                }
                if line.contains("TODO") || line.contains("FIXME") {
                    analysis.has_todos = true;
                }
            } else if line.starts_with("-") && !line.starts_with("---") {
                analysis.deletions += 1;
            }
        }

        // 推断变更类型
        analysis.infer_change_type();

        analysis
    }

    /// 列出所有分支
    pub fn list_branches(&self) -> Result<Vec<String>, String> {
        let output = Command::new("git")
            .args(["branch", "-a"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("执行 git branch 失败: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git branch 失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let branches: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.trim_start_matches("* ").trim().to_string())
            .collect();

        Ok(branches)
    }

    /// 获取最近的提交
    pub fn recent_commits(&self, count: usize) -> Result<Vec<String>, String> {
        let output = Command::new("git")
            .args(["log", &format!("-{}", count), "--oneline"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("执行 git log 失败: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git log 失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let commits: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.to_string())
            .collect();

        Ok(commits)
    }
}

/// 变更分析结果
#[derive(Debug, Clone, Default)]
pub struct ChangeAnalysis {
    pub additions: usize,
    pub deletions: usize,
    pub rust_files: usize,
    pub doc_files: usize,
    pub config_files: usize,
    pub script_files: usize,
    pub has_new_functions: bool,
    pub has_new_types: bool,
    pub has_tests: bool,
    pub has_todos: bool,
    pub suggested_type: Option<String>,
    pub suggested_scope: Option<String>,
}

impl ChangeAnalysis {
    /// 推断提交类型
    pub fn infer_change_type(&mut self) {
        // 根据文件类型和变更模式推断
        if self.doc_files > 0 && self.rust_files == 0 {
            self.suggested_type = Some("docs".to_string());
        } else if self.config_files > 0 && self.rust_files == 0 {
            self.suggested_type = Some("chore".to_string());
            self.suggested_scope = Some("config".to_string());
        } else if self.has_tests && !self.has_new_functions {
            self.suggested_type = Some("test".to_string());
        } else if self.has_new_functions || self.has_new_types {
            if self.additions > self.deletions * 2 {
                self.suggested_type = Some("feat".to_string());
            } else {
                self.suggested_type = Some("refactor".to_string());
            }
        } else if self.deletions > self.additions {
            self.suggested_type = Some("refactor".to_string());
        } else {
            self.suggested_type = Some("fix".to_string());
        }
    }

    /// 获取建议的提交类型
    pub fn suggested_commit_type(&self) -> &str {
        self.suggested_type.as_deref().unwrap_or("chore")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_message_format() {
        let msg = CommitMessage {
            commit_type: "feat".to_string(),
            scope: Some("git".to_string()),
            subject: "add git smart assistant".to_string(),
            body: Some("Implement intelligent Git operations".to_string()),
            footer: Some("Closes #123".to_string()),
        };

        let formatted = msg.format();
        assert!(formatted.contains("feat(git): add git smart assistant"));
        assert!(formatted.contains("Implement intelligent Git operations"));
        assert!(formatted.contains("Closes #123"));
    }

    #[test]
    fn test_commit_message_format_no_scope() {
        let msg = CommitMessage {
            commit_type: "fix".to_string(),
            scope: None,
            subject: "fix bug".to_string(),
            body: None,
            footer: None,
        };

        let formatted = msg.format();
        assert_eq!(formatted, "fix: fix bug");
    }

    #[test]
    fn test_change_analysis_infer_docs() {
        let mut analysis = ChangeAnalysis {
            doc_files: 2,
            rust_files: 0,
            additions: 10,
            ..Default::default()
        };

        analysis.infer_change_type();
        assert_eq!(analysis.suggested_commit_type(), "docs");
    }

    #[test]
    fn test_change_analysis_infer_feat() {
        let mut analysis = ChangeAnalysis {
            rust_files: 1,
            has_new_functions: true,
            additions: 100,
            deletions: 10,
            ..Default::default()
        };

        analysis.infer_change_type();
        assert_eq!(analysis.suggested_commit_type(), "feat");
    }
}
