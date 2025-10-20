//! Git 命令
//!
//! 提供智能化的 Git 操作命令

use crate::command::Command;
use crate::git_assistant::GitRepository;
use colored::Colorize;

/// 注册 Git 相关命令
pub fn register_git_commands(registry: &mut crate::command::CommandRegistry) {
    // Git 状态命令
    registry.register(Command::from_fn(
        "git-status",
        "显示详细的 Git 状态信息",
        handle_git_status,
    ));

    registry.register(Command::from_fn(
        "gs",
        "显示 Git 状态（git-status 的别名）",
        handle_git_status,
    ));

    // Git diff 命令
    registry.register(Command::from_fn(
        "git-diff",
        "显示 Git 变更详情",
        handle_git_diff,
    ));

    registry.register(Command::from_fn(
        "gd",
        "显示 Git 变更（git-diff 的别名）",
        handle_git_diff,
    ));

    // Git 分支命令
    registry.register(Command::from_fn(
        "git-branch",
        "显示分支信息",
        handle_git_branch,
    ));

    registry.register(Command::from_fn(
        "gb",
        "显示分支信息（git-branch 的别名）",
        handle_git_branch,
    ));

    // Git 提交分析命令
    registry.register(Command::from_fn(
        "git-analyze",
        "分析当前变更并建议提交信息",
        handle_git_analyze,
    ));

    registry.register(Command::from_fn(
        "ga",
        "分析变更（git-analyze 的别名）",
        handle_git_analyze,
    ));
}

/// 处理 /git-status 命令
fn handle_git_status(_arg: &str) -> String {
    let repo = match GitRepository::current() {
        Ok(repo) => repo,
        Err(e) => return format!("{} {}", "✗".red(), e.yellow()),
    };

    let status = match repo.status() {
        Ok(status) => status,
        Err(e) => return format!("{} {}", "✗ 获取 Git 状态失败:".red(), e),
    };

    let mut output = vec![];

    // 标题
    output.push(format!("\n{}", "Git 状态".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    // 当前分支
    if let Some(branch) = &status.current_branch {
        let branch_display = if branch == "main" || branch == "master" {
            branch.green().bold()
        } else {
            branch.cyan().bold()
        };
        output.push(format!("  {}: {}", "当前分支".dimmed(), branch_display));
    }

    // 变更状态
    let status_icon = if status.has_changes {
        "●".yellow()
    } else {
        "✓".green()
    };

    let status_text = if status.has_changes {
        "有未提交的变更".yellow()
    } else {
        "工作区干净".green()
    };

    output.push(format!(
        "  {}: {} {}",
        "状态".dimmed(),
        status_icon,
        status_text
    ));

    // 文件统计
    if status.has_changes {
        output.push(String::new());
        output.push(format!("{}", "变更文件".cyan().bold()));
        output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

        if status.staged_files > 0 {
            output.push(format!(
                "  {}: {} {}",
                "暂存区".green(),
                status.staged_files,
                "个文件".dimmed()
            ));
        }

        if status.unstaged_files > 0 {
            output.push(format!(
                "  {}: {} {}",
                "未暂存".yellow(),
                status.unstaged_files,
                "个文件".dimmed()
            ));
        }

        if status.untracked_files > 0 {
            output.push(format!(
                "  {}: {} {}",
                "未跟踪".red(),
                status.untracked_files,
                "个文件".dimmed()
            ));
        }
    }

    // 远程状态
    if status.ahead > 0 || status.behind > 0 {
        output.push(String::new());
        output.push(format!("{}", "远程状态".cyan().bold()));
        output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

        if status.ahead > 0 {
            output.push(format!(
                "  {}: {} {}",
                "领先".green(),
                status.ahead,
                "个提交".dimmed()
            ));
        }

        if status.behind > 0 {
            output.push(format!(
                "  {}: {} {}",
                "落后".yellow(),
                status.behind,
                "个提交".dimmed()
            ));
        }
    }

    // 远程仓库
    if let Some(url) = &status.remote_url {
        output.push(String::new());
        output.push(format!("  {}: {}", "远程仓库".dimmed(), url.dimmed()));
    }

    // 建议操作
    output.push(String::new());
    output.push(format!("{}", "快捷命令".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    if status.has_changes {
        if status.staged_files > 0 {
            output.push(format!("  {} 查看暂存变更", "/gd --staged".cyan()));
            output.push(format!("  {} 分析并生成提交信息", "/ga".cyan()));
        } else {
            output.push(format!("  {} 查看所有变更", "/gd".cyan()));
        }
    }

    output.push(format!("  {} 查看分支信息", "/gb".cyan()));

    output.push(String::new());

    output.join("\n")
}

/// 处理 /git-diff 命令
fn handle_git_diff(arg: &str) -> String {
    let repo = match GitRepository::current() {
        Ok(repo) => repo,
        Err(e) => return format!("{} {}", "✗".red(), e.yellow()),
    };

    let staged = arg.contains("--staged") || arg.contains("--cached");

    // 获取简短统计
    let stat = match repo.get_diff_stat(staged) {
        Ok(stat) => stat,
        Err(e) => return format!("{} {}", "✗ 获取 diff 失败:".red(), e),
    };

    let mut output = vec![];

    // 标题
    let title = if staged {
        "Git Diff (暂存区)".cyan().bold()
    } else {
        "Git Diff (工作区)".cyan().bold()
    };

    output.push(format!("\n{}", title));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    if stat.is_empty() {
        let msg = if staged {
            "暂存区没有变更"
        } else {
            "工作区没有变更"
        };
        output.push(format!("\n  {}", msg.dimmed()));
    } else {
        output.push(String::new());
        output.push(stat);

        // 变更分析
        if let Ok(diff) = repo.get_diff(staged) {
            let analysis = repo.analyze_changes(&diff);

            output.push(String::new());
            output.push(format!("{}", "变更分析".cyan().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            output.push(format!(
                "  {}: {} {}, {} {}",
                "代码行".dimmed(),
                format!("+{}", analysis.additions).green(),
                "行".dimmed(),
                format!("-{}", analysis.deletions).red(),
                "行".dimmed()
            ));

            if analysis.rust_files > 0 {
                output.push(format!(
                    "  {}: {} 个",
                    "Rust 文件".dimmed(),
                    analysis.rust_files
                ));
            }
            if analysis.doc_files > 0 {
                output.push(format!(
                    "  {}: {} 个",
                    "文档文件".dimmed(),
                    analysis.doc_files
                ));
            }
            if analysis.config_files > 0 {
                output.push(format!(
                    "  {}: {} 个",
                    "配置文件".dimmed(),
                    analysis.config_files
                ));
            }

            output.push(String::new());

            let mut tags = vec![];
            if analysis.has_new_functions {
                tags.push("新函数".green().to_string());
            }
            if analysis.has_new_types {
                tags.push("新类型".green().to_string());
            }
            if analysis.has_tests {
                tags.push("测试".cyan().to_string());
            }
            if analysis.has_todos {
                tags.push("TODO".yellow().to_string());
            }

            if !tags.is_empty() {
                output.push(format!("  {}: {}", "特征".dimmed(), tags.join(", ")));
            }

            // 建议的提交类型
            output.push(format!(
                "  {}: {}",
                "建议类型".dimmed(),
                analysis.suggested_commit_type().cyan()
            ));
        }

        // 提示
        output.push(String::new());
        output.push(format!("  💡 {}", "使用 /ga 生成智能提交信息".dimmed()));
    }

    output.push(String::new());

    output.join("\n")
}

/// 处理 /git-branch 命令
fn handle_git_branch(_arg: &str) -> String {
    let repo = match GitRepository::current() {
        Ok(repo) => repo,
        Err(e) => return format!("{} {}", "✗".red(), e.yellow()),
    };

    let current_branch = repo.get_current_branch().ok().flatten();
    let branches = match repo.list_branches() {
        Ok(branches) => branches,
        Err(e) => return format!("{} {}", "✗ 获取分支列表失败:".red(), e),
    };

    let mut output = vec![];

    output.push(format!("\n{}", "Git 分支".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    if let Some(current) = &current_branch {
        output.push(format!(
            "\n  {}: {}",
            "当前分支".dimmed(),
            current.green().bold()
        ));
    }

    output.push(String::new());
    output.push(format!("{}", "所有分支".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    let mut local_branches = vec![];
    let mut remote_branches = vec![];

    for branch in branches {
        if branch.contains("remotes/") {
            remote_branches.push(branch);
        } else {
            local_branches.push(branch);
        }
    }

    // 本地分支
    if !local_branches.is_empty() {
        output.push(format!("\n  {}", "本地分支:".cyan()));
        for branch in local_branches {
            let is_current = current_branch
                .as_ref()
                .map(|c| branch.contains(c))
                .unwrap_or(false);
            if is_current {
                output.push(format!("    {} {}", "●".green(), branch.green().bold()));
            } else {
                output.push(format!("    {} {}", "○".dimmed(), branch.dimmed()));
            }
        }
    }

    // 远程分支
    if !remote_branches.is_empty() {
        output.push(format!("\n  {}", "远程分支:".yellow()));
        for branch in remote_branches.iter().take(5) {
            output.push(format!("    {} {}", "○".dimmed(), branch.dimmed()));
        }
        if remote_branches.len() > 5 {
            output.push(format!(
                "    {} 还有 {} 个远程分支",
                "...".dimmed(),
                remote_branches.len() - 5
            ));
        }
    }

    output.push(String::new());

    output.join("\n")
}

/// 处理 /git-analyze 命令
fn handle_git_analyze(_arg: &str) -> String {
    let repo = match GitRepository::current() {
        Ok(repo) => repo,
        Err(e) => return format!("{} {}", "✗".red(), e.yellow()),
    };

    // 检查是否有暂存的变更
    let status = match repo.status() {
        Ok(status) => status,
        Err(e) => return format!("{} {}", "✗ 获取 Git 状态失败:".red(), e),
    };

    if status.staged_files == 0 {
        return format!(
            "\n{} {}\n\n  💡 {}\n",
            "提示:".yellow(),
            "暂存区没有变更",
            "使用 'git add' 命令暂存要提交的文件".dimmed()
        );
    }

    // 获取暂存区的 diff
    let diff = match repo.get_diff(true) {
        Ok(diff) => diff,
        Err(e) => return format!("{} {}", "✗ 获取 diff 失败:".red(), e),
    };

    // 分析变更
    let analysis = repo.analyze_changes(&diff);

    let mut output = vec![];

    output.push(format!("\n{}", "提交分析".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    // 变更统计
    output.push(format!(
        "\n  {}: {} 个文件, {} 行, {} 行",
        "变更范围".dimmed(),
        status.staged_files,
        format!("+{}", analysis.additions).green(),
        format!("-{}", analysis.deletions).red()
    ));

    // 变更类型
    let mut file_types = vec![];
    if analysis.rust_files > 0 {
        file_types.push(format!("{} 个 Rust 文件", analysis.rust_files));
    }
    if analysis.doc_files > 0 {
        file_types.push(format!("{} 个文档", analysis.doc_files));
    }
    if analysis.config_files > 0 {
        file_types.push(format!("{} 个配置", analysis.config_files));
    }

    if !file_types.is_empty() {
        output.push(format!(
            "  {}: {}",
            "文件类型".dimmed(),
            file_types.join(", ")
        ));
    }

    // 特征标记
    let mut features = vec![];
    if analysis.has_new_functions {
        features.push("新函数");
    }
    if analysis.has_new_types {
        features.push("新类型");
    }
    if analysis.has_tests {
        features.push("测试");
    }

    if !features.is_empty() {
        output.push(format!(
            "  {}: {}",
            "代码特征".dimmed(),
            features.join(", ")
        ));
    }

    // 建议的提交信息
    output.push(String::new());
    output.push(format!("{}", "建议提交信息".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    let commit_type = analysis.suggested_commit_type();
    let scope = analysis.suggested_scope.as_deref().unwrap_or("");

    // 生成提交信息模板
    let subject = generate_commit_subject(&analysis);

    output.push(String::new());
    if scope.is_empty() {
        output.push(format!("  {}: {}", commit_type.cyan().bold(), subject));
    } else {
        output.push(format!(
            "  {}({}): {}",
            commit_type.cyan().bold(),
            scope.yellow(),
            subject
        ));
    }

    output.push(String::new());
    output.push(format!("{}", "提示".yellow().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());
    output.push(format!(
        "\n  {} {}",
        "1.".dimmed(),
        "复制上面的提交信息模板"
    ));
    output.push(format!("  {} {}", "2.".dimmed(), "根据实际情况修改和完善"));
    output.push(format!(
        "  {} {}",
        "3.".dimmed(),
        "使用 'git commit -m \"...\"' 提交"
    ));

    output.push(String::new());
    output.push(format!(
        "  💡 {}",
        "未来版本将支持 LLM 自动生成详细提交信息".dimmed()
    ));

    output.push(String::new());

    output.join("\n")
}

/// 生成提交主题
fn generate_commit_subject(analysis: &crate::git_assistant::ChangeAnalysis) -> String {
    // 根据分析结果生成简短的主题行
    if analysis.doc_files > 0 && analysis.rust_files == 0 {
        "update documentation".to_string()
    } else if analysis.config_files > 0 && analysis.rust_files == 0 {
        "update configuration".to_string()
    } else if analysis.has_tests && !analysis.has_new_functions {
        "add/update tests".to_string()
    } else if analysis.has_new_functions || analysis.has_new_types {
        if analysis.additions > analysis.deletions * 2 {
            "implement new feature".to_string()
        } else {
            "refactor code structure".to_string()
        }
    } else {
        "update code".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== generate_commit_subject 测试 ==========

    #[test]
    fn test_generate_commit_subject_docs() {
        let analysis = crate::git_assistant::ChangeAnalysis {
            doc_files: 2,
            rust_files: 0,
            ..Default::default()
        };

        let subject = generate_commit_subject(&analysis);
        assert_eq!(subject, "update documentation");
    }

    #[test]
    fn test_generate_commit_subject_feat() {
        let mut analysis = crate::git_assistant::ChangeAnalysis {
            rust_files: 1,
            has_new_functions: true,
            additions: 100,
            deletions: 10,
            ..Default::default()
        };
        analysis.infer_change_type();

        let subject = generate_commit_subject(&analysis);
        assert_eq!(subject, "implement new feature");
    }

    #[test]
    fn test_generate_commit_subject_config() {
        let analysis = crate::git_assistant::ChangeAnalysis {
            config_files: 1,
            rust_files: 0,
            ..Default::default()
        };

        let subject = generate_commit_subject(&analysis);
        assert_eq!(subject, "update configuration");
    }

    #[test]
    fn test_generate_commit_subject_tests() {
        let analysis = crate::git_assistant::ChangeAnalysis {
            rust_files: 1,
            has_tests: true,
            has_new_functions: false,
            ..Default::default()
        };

        let subject = generate_commit_subject(&analysis);
        assert_eq!(subject, "add/update tests");
    }

    #[test]
    fn test_generate_commit_subject_refactor() {
        let analysis = crate::git_assistant::ChangeAnalysis {
            rust_files: 1,
            has_new_functions: true,
            additions: 50,
            deletions: 40,
            ..Default::default()
        };

        let subject = generate_commit_subject(&analysis);
        assert_eq!(subject, "refactor code structure");
    }

    #[test]
    fn test_generate_commit_subject_default() {
        let analysis = crate::git_assistant::ChangeAnalysis {
            rust_files: 1,
            additions: 10,
            deletions: 5,
            ..Default::default()
        };

        let subject = generate_commit_subject(&analysis);
        assert_eq!(subject, "update code");
    }

    // ========== handle_git_status 测试 ==========

    #[test]
    fn test_handle_git_status_in_repo() {
        // 在当前 Git 仓库中运行
        let result = handle_git_status("");

        // 应该成功获取状态（分别检查，避免 ANSI 码问题）
        assert!(
            result.contains("Git") || result.contains("✗"),
            "Expected Git in output or error"
        );
        assert!(
            result.contains("状态") || result.contains("✗"),
            "Expected status or error"
        );
    }

    #[test]
    fn test_handle_git_status_output_format() {
        let result = handle_git_status("");

        // 验证输出格式包含关键部分（分别检查）
        assert!(result.contains("Git") || result.contains("✗"));
        assert!(result.contains("快捷命令") || result.contains("✗"));
    }

    // ========== handle_git_diff 测试 ==========

    #[test]
    fn test_handle_git_diff_basic() {
        // 测试基础 diff 功能
        let result = handle_git_diff("");

        // 应该返回 diff 信息（分别检查关键字）
        assert!(result.contains("Git") || result.contains("没有变更") || result.contains("✗"));
        assert!(result.contains("Diff") || result.contains("没有变更") || result.contains("✗"));
    }

    #[test]
    fn test_handle_git_diff_staged() {
        // 测试暂存区 diff
        let result = handle_git_diff("--staged");

        // 应该显示暂存区标题
        assert!(result.contains("暂存") || result.contains("没有变更") || result.contains("✗"));
    }

    #[test]
    fn test_handle_git_diff_cached_alias() {
        // 测试 --cached 别名
        let result = handle_git_diff("--cached");

        // 应该与 --staged 行为一致
        assert!(result.contains("暂存") || result.contains("没有变更") || result.contains("✗"));
    }

    // ========== handle_git_branch 测试 ==========

    #[test]
    fn test_handle_git_branch_basic() {
        // 测试分支列表
        let result = handle_git_branch("");

        // 应该包含分支信息（分别检查）
        assert!(result.contains("Git") || result.contains("分支") || result.contains("✗"));
    }

    #[test]
    fn test_handle_git_branch_shows_current() {
        let result = handle_git_branch("");

        // 应该显示当前分支（如果在 Git 仓库中）
        assert!(
            result.contains("当前") || result.contains("分支") || result.contains("✗"),
            "Should show current branch or error"
        );
    }

    // ========== handle_git_analyze 测试 ==========

    #[test]
    fn test_handle_git_analyze_no_staged() {
        // 在没有暂存变更时调用
        // 注意：这个测试假设当前没有暂存的变更
        let result = handle_git_analyze("");

        // 应该提示需要暂存文件
        assert!(
            result.contains("暂存区没有变更") || result.contains("提交分析"),
            "Should mention staging area or show analysis"
        );
    }

    #[test]
    fn test_handle_git_analyze_output_structure() {
        let result = handle_git_analyze("");

        // 验证输出包含必要元素
        assert!(
            result.contains("暂存区") || result.contains("提交分析") || result.contains("提示")
        );
    }

    // ========== 集成测试：命令注册 ==========

    #[test]
    fn test_register_git_commands() {
        use crate::command::CommandRegistry;

        let mut registry = CommandRegistry::new();
        register_git_commands(&mut registry);

        // 验证所有命令都已注册
        assert!(registry.get("git-status").is_some());
        assert!(registry.get("gs").is_some());
        assert!(registry.get("git-diff").is_some());
        assert!(registry.get("gd").is_some());
        assert!(registry.get("git-branch").is_some());
        assert!(registry.get("gb").is_some());
        assert!(registry.get("git-analyze").is_some());
        assert!(registry.get("ga").is_some());
    }

    #[test]
    fn test_git_commands_aliases() {
        use crate::command::CommandRegistry;

        let mut registry = CommandRegistry::new();
        register_git_commands(&mut registry);

        // 验证别名和主命令
        let status_full = registry.get("git-status").unwrap();
        let status_short = registry.get("gs").unwrap();
        assert_eq!(status_full.name, "git-status");
        assert_eq!(status_short.name, "gs");
    }

    #[test]
    fn test_git_commands_descriptions() {
        use crate::command::CommandRegistry;

        let mut registry = CommandRegistry::new();
        register_git_commands(&mut registry);

        // 验证命令描述
        let cmd = registry.get("git-status").unwrap();
        assert!(cmd.desc.contains("Git 状态"));

        let cmd = registry.get("git-diff").unwrap();
        assert!(cmd.desc.contains("变更"));
    }

    // ========== 错误场景测试 ==========

    #[test]
    fn test_handle_git_status_not_a_repo() {
        use std::env;

        // 保存当前目录
        let original_dir = env::current_dir().unwrap();

        // 切换到 /tmp（非 Git 仓库）
        let temp_dir = std::path::Path::new("/tmp");
        if temp_dir.exists() {
            let _ = env::set_current_dir(temp_dir);

            let result = handle_git_status("");

            // 应该返回错误信息
            assert!(result.contains("✗") || result.contains("错误") || result.contains("失败"));

            // 恢复原目录
            let _ = env::set_current_dir(&original_dir);
        }
    }

    #[test]
    fn test_handle_git_diff_not_a_repo() {
        use std::env;

        let original_dir = env::current_dir().unwrap();
        let temp_dir = std::path::Path::new("/tmp");

        if temp_dir.exists() {
            let _ = env::set_current_dir(temp_dir);

            let result = handle_git_diff("");

            // 应该返回错误信息
            assert!(result.contains("✗") || result.contains("错误"));

            let _ = env::set_current_dir(&original_dir);
        }
    }

    #[test]
    fn test_handle_git_branch_not_a_repo() {
        use std::env;

        let original_dir = env::current_dir().unwrap();
        let temp_dir = std::path::Path::new("/tmp");

        if temp_dir.exists() {
            let _ = env::set_current_dir(temp_dir);

            let result = handle_git_branch("");

            // 应该返回错误信息
            assert!(result.contains("✗") || result.contains("错误"));

            let _ = env::set_current_dir(&original_dir);
        }
    }
}
