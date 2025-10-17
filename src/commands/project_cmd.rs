//! 项目上下文命令
//!
//! 显示当前项目的上下文信息

use crate::command::Command;
use crate::project_context::ProjectContext;
use colored::Colorize;

/// 注册项目相关命令
pub fn register_project_commands(registry: &mut crate::command::CommandRegistry) {
    registry.register(Command::from_fn(
        "project",
        "显示当前项目信息",
        handle_project,
    ));

    registry.register(Command::from_fn(
        "proj",
        "显示当前项目信息（project 的别名）",
        handle_project,
    ));
}

/// 处理 /project 命令
fn handle_project(_arg: &str) -> String {
    let context = ProjectContext::detect();

    let mut output = vec![];

    // 标题
    output.push(format!("\n{}", "项目信息".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    // 基本信息
    output.push(format!("  {}: {}", "项目名称".dimmed(), context.project_name().cyan()));
    output.push(format!("  {}: {}", "项目类型".dimmed(), context.type_description().green()));
    output.push(format!("  {}: {}", "根目录".dimmed(), context.root.display().to_string().dimmed()));

    // 项目类型详情
    output.push(String::new());
    output.push(format!("{}", "项目详情".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    match &context.project_type {
        crate::project_context::ProjectType::Rust { cargo_toml, has_src, has_tests } => {
            output.push(format!("  {}: {}", "Cargo.toml".dimmed(), "✓".green()));
            output.push(format!("  {}: {}", "src/ 目录".dimmed(), if *has_src { "✓".green() } else { "✗".red() }));
            output.push(format!("  {}: {}", "tests/ 目录".dimmed(), if *has_tests { "✓".green() } else { "✗".red() }));
            output.push(format!("  {}: {}", "配置文件".dimmed(), cargo_toml.display().to_string().dimmed()));
        }
        crate::project_context::ProjectType::Python { requirements, pyproject, setup_py } => {
            output.push(format!("  {}: {}", "requirements.txt".dimmed(), if requirements.is_some() { "✓".green() } else { "✗".dimmed() }));
            output.push(format!("  {}: {}", "pyproject.toml".dimmed(), if pyproject.is_some() { "✓".green() } else { "✗".dimmed() }));
            output.push(format!("  {}: {}", "setup.py".dimmed(), if setup_py.is_some() { "✓".green() } else { "✗".dimmed() }));
        }
        crate::project_context::ProjectType::Node { package_json, has_node_modules } => {
            output.push(format!("  {}: {}", "package.json".dimmed(), "✓".green()));
            output.push(format!("  {}: {}", "node_modules/".dimmed(), if *has_node_modules { "✓".green() } else { "✗".dimmed() }));
            output.push(format!("  {}: {}", "配置文件".dimmed(), package_json.display().to_string().dimmed()));
        }
        crate::project_context::ProjectType::Go { go_mod, has_go_sum } => {
            output.push(format!("  {}: {}", "go.mod".dimmed(), "✓".green()));
            output.push(format!("  {}: {}", "go.sum".dimmed(), if *has_go_sum { "✓".green() } else { "✗".dimmed() }));
            output.push(format!("  {}: {}", "配置文件".dimmed(), go_mod.display().to_string().dimmed()));
        }
        crate::project_context::ProjectType::Java { build_file } => {
            use crate::project_context::JavaBuildFile;
            match build_file {
                JavaBuildFile::Maven(path) => {
                    output.push(format!("  {}: Maven", "构建工具".dimmed()));
                    output.push(format!("  {}: {}", "pom.xml".dimmed(), "✓".green()));
                    output.push(format!("  {}: {}", "配置文件".dimmed(), path.display().to_string().dimmed()));
                }
                JavaBuildFile::Gradle(path) | JavaBuildFile::GradleKts(path) => {
                    output.push(format!("  {}: Gradle", "构建工具".dimmed()));
                    output.push(format!("  {}: {}", "build.gradle".dimmed(), "✓".green()));
                    output.push(format!("  {}: {}", "配置文件".dimmed(), path.display().to_string().dimmed()));
                }
            }
        }
        crate::project_context::ProjectType::Unknown => {
            output.push(format!("  {}", "未检测到识别的项目类型".yellow()));
        }
    }

    // Git 信息
    if let Some(git_info) = &context.git_info {
        output.push(String::new());
        output.push(format!("{}", "Git 信息".cyan().bold()));
        output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

        if let Some(branch) = &git_info.current_branch {
            output.push(format!("  {}: {}", "当前分支".dimmed(), branch.green()));
        }

        let status = if git_info.is_dirty {
            "有未提交的变更".yellow()
        } else {
            "工作区干净".green()
        };
        output.push(format!("  {}: {}", "状态".dimmed(), status));

        if let Some(url) = &git_info.remote_url {
            output.push(format!("  {}: {}", "远程仓库".dimmed(), url.dimmed()));
        }
    }

    // 建议命令
    if context.is_recognized() {
        output.push(String::new());
        output.push(format!("{}", "建议命令".cyan().bold()));
        output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

        if let Some(build_cmd) = context.build_command() {
            output.push(format!("  {}: {}", "构建".dimmed(), build_cmd.cyan()));
        }

        if let Some(test_cmd) = context.test_command() {
            output.push(format!("  {}: {}", "测试".dimmed(), test_cmd.cyan()));
        }

        if let Some(run_cmd) = context.run_command() {
            output.push(format!("  {}: {}", "运行".dimmed(), run_cmd.cyan()));
        }

        output.push(String::new());
        output.push(format!("  {}", "💡 提示：可以直接输入\"运行测试\"等自然语言".dimmed()));
    }

    output.push(String::new());

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_project_command() {
        let result = handle_project("");
        // 应该返回项目信息（分别检查关键词）
        assert!(result.contains("项目") || result.contains("信息"));
    }

    #[test]
    fn test_handle_project_shows_type() {
        let result = handle_project("");
        // 应该显示项目类型
        assert!(result.contains("项目类型") || result.contains("type"));
    }

    #[test]
    fn test_handle_project_shows_root() {
        let result = handle_project("");
        // 应该显示根目录
        assert!(result.contains("根目录") || result.contains("root"));
    }

    #[test]
    fn test_handle_project_rust_detection() {
        // 当前就是 Rust 项目，应该检测到
        let result = handle_project("");

        // 应该包含 Rust 相关信息（分别检查）
        assert!(result.contains("Rust") || result.contains("Cargo"));
    }

    #[test]
    fn test_handle_project_output_format() {
        let result = handle_project("");

        // 验证输出包含多个分隔线
        assert!(result.matches("━").count() >= 2);
    }

    #[test]
    fn test_handle_project_shows_git_info() {
        let result = handle_project("");

        // 当前目录是 Git 仓库，应该显示 Git 信息
        assert!(result.contains("Git") || result.contains("分支") || result.contains("branch"));
    }

    #[test]
    fn test_handle_project_shows_suggestions() {
        let result = handle_project("");

        // Rust 项目应该有建议命令
        assert!(result.contains("建议") || result.contains("构建") || result.contains("测试"));
    }

    #[test]
    fn test_register_project_commands() {
        use crate::command::CommandRegistry;

        let mut registry = CommandRegistry::new();
        register_project_commands(&mut registry);

        // 验证命令注册
        assert!(registry.get("project").is_some());
        assert!(registry.get("proj").is_some());
    }

    #[test]
    fn test_project_command_alias() {
        use crate::command::CommandRegistry;

        let mut registry = CommandRegistry::new();
        register_project_commands(&mut registry);

        let cmd_full = registry.get("project").unwrap();
        let cmd_short = registry.get("proj").unwrap();
        assert_eq!(cmd_full.name, "project");
        assert_eq!(cmd_short.name, "proj");
    }

    #[test]
    fn test_project_command_description() {
        use crate::command::CommandRegistry;

        let mut registry = CommandRegistry::new();
        register_project_commands(&mut registry);

        let cmd = registry.get("project").unwrap();
        assert!(cmd.desc.contains("项目") || cmd.desc.contains("信息"));
    }
}
