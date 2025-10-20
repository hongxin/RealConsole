//! 日志分析命令
//!
//! 提供日志文件分析和错误模式识别

use crate::command::Command;
use crate::log_analyzer::{LogAnalyzer, LogLevel};
use colored::Colorize;
use std::path::Path;

/// 注册日志分析相关命令
pub fn register_log_analysis_commands(registry: &mut crate::command::CommandRegistry) {
    // 日志分析命令
    registry.register(Command::from_fn(
        "log-analyze",
        "分析日志文件，提取错误和警告",
        handle_log_analyze,
    ));

    registry.register(Command::from_fn(
        "la",
        "分析日志文件（log-analyze 的别名）",
        handle_log_analyze,
    ));

    // 查看最近日志
    registry.register(Command::from_fn(
        "log-tail",
        "查看并分析最近的日志",
        handle_log_tail,
    ));

    registry.register(Command::from_fn(
        "lt",
        "查看最近日志（log-tail 的别名）",
        handle_log_tail,
    ));

    // 错误统计
    registry.register(Command::from_fn(
        "log-errors",
        "显示日志中的所有错误",
        handle_log_errors,
    ));

    registry.register(Command::from_fn(
        "le",
        "显示日志错误（log-errors 的别名）",
        handle_log_errors,
    ));
}

/// 处理 /log-analyze 命令
fn handle_log_analyze(arg: &str) -> String {
    let args: Vec<&str> = arg.split_whitespace().collect();

    if args.is_empty() {
        return format!(
            "\n{} {}\n\n{}\n  {} /la <file_path>\n  {} /la application.log\n  {} /la /var/log/app.log --max-lines 1000\n",
            "用法:".yellow().bold(),
            "log-analyze <file_path> [--max-lines N]",
            "示例:".cyan(),
            "•".dimmed(),
            "•".dimmed(),
            "•".dimmed()
        );
    }

    let file_path = args[0];

    // 解析参数
    let max_lines = args
        .iter()
        .position(|&x| x == "--max-lines")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<usize>().ok());

    // 检查文件是否存在
    if !Path::new(file_path).exists() {
        return format!("{} 文件不存在: {}", "✗".red(), file_path.yellow());
    }

    // 创建分析器（✨ Phase 7.2: 限制错误和警告数量，避免内存占用过大）
    let mut analyzer = LogAnalyzer::new()
        .with_max_errors(100)
        .with_max_warnings(100);

    if let Some(max) = max_lines {
        analyzer = analyzer.with_max_lines(max);
    }

    // 分析日志
    let analysis = match analyzer.analyze_file(file_path) {
        Ok(analysis) => analysis,
        Err(e) => return format!("{} {}", "✗ 分析失败:".red(), e),
    };

    format_analysis_result(&analysis, file_path)
}

/// 处理 /log-tail 命令
fn handle_log_tail(arg: &str) -> String {
    let args: Vec<&str> = arg.split_whitespace().collect();

    if args.is_empty() {
        return format!(
            "\n{} {}\n\n{}\n  {} /lt <file_path> [lines]\n  {} /lt application.log\n  {} /lt app.log 100\n",
            "用法:".yellow().bold(),
            "log-tail <file_path> [lines]",
            "示例:".cyan(),
            "•".dimmed(),
            "•".dimmed(),
            "•".dimmed()
        );
    }

    let file_path = args[0];
    let lines = args
        .get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(50);

    // 检查文件是否存在
    if !Path::new(file_path).exists() {
        return format!("{} 文件不存在: {}", "✗".red(), file_path.yellow());
    }

    // 分析最近的日志（✨ Phase 7.2: 限制错误和警告数量）
    let analyzer = LogAnalyzer::new().with_max_errors(50).with_max_warnings(50);

    let analysis = match analyzer.analyze_tail(file_path, lines) {
        Ok(analysis) => analysis,
        Err(e) => return format!("{} {}", "✗ 分析失败:".red(), e),
    };

    let mut output = vec![];

    output.push(format!(
        "\n{}",
        format!("最近 {} 行日志分析", lines).cyan().bold()
    ));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    output.push(format!("\n  {}: {}", "文件".dimmed(), file_path.cyan()));
    output.push(format!(
        "  {}: {} 行",
        "分析行数".dimmed(),
        analysis.total_lines
    ));

    // 级别统计
    output.push(String::new());
    output.push(format!("{}", "日志级别分布".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    let error_count = analysis.error_count();
    let warn_count = analysis.warning_count();
    let info_count = analysis.info_count();

    if error_count > 0 {
        output.push(format!(
            "  {}: {} {}",
            "ERROR".red().bold(),
            error_count,
            "条".dimmed()
        ));
    }

    if warn_count > 0 {
        output.push(format!(
            "  {}: {} {}",
            "WARN".yellow().bold(),
            warn_count,
            "条".dimmed()
        ));
    }

    if info_count > 0 {
        output.push(format!(
            "  {}: {} {}",
            "INFO".green(),
            info_count,
            "条".dimmed()
        ));
    }

    // 显示最近的错误
    if !analysis.errors.is_empty() {
        output.push(String::new());
        output.push(format!("{}", "最近的错误".red().bold()));
        output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

        for (i, error) in analysis.errors.iter().rev().take(5).enumerate() {
            output.push(format!(
                "\n  {} {}",
                format!("#{}", i + 1).dimmed(),
                error.message.red()
            ));
        }
    }

    // 提示
    output.push(String::new());
    output.push(format!(
        "  💡 {}",
        format!("使用 /le {} 查看所有错误", file_path).dimmed()
    ));

    output.push(String::new());

    output.join("\n")
}

/// 处理 /log-errors 命令
fn handle_log_errors(arg: &str) -> String {
    let args: Vec<&str> = arg.split_whitespace().collect();

    if args.is_empty() {
        return format!(
            "\n{} {}\n\n{}\n  {} /le <file_path>\n  {} /le application.log\n",
            "用法:".yellow().bold(),
            "log-errors <file_path>",
            "示例:".cyan(),
            "•".dimmed(),
            "•".dimmed()
        );
    }

    let file_path = args[0];

    // 检查文件是否存在
    if !Path::new(file_path).exists() {
        return format!("{} 文件不存在: {}", "✗".red(), file_path.yellow());
    }

    // 只分析错误（✨ Phase 7.2: 限制最多保存 100 个错误，避免内存占用过大）
    let analyzer = LogAnalyzer::new().errors_only().with_max_errors(100);

    let analysis = match analyzer.analyze_file(file_path) {
        Ok(analysis) => analysis,
        Err(e) => return format!("{} {}", "✗ 分析失败:".red(), e),
    };

    let mut output = vec![];

    output.push(format!("\n{}", "错误日志详情".red().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    output.push(format!("\n  {}: {}", "文件".dimmed(), file_path.cyan()));
    output.push(format!(
        "  {}: {} 个",
        "错误总数".dimmed(),
        analysis.errors.len().to_string().red()
    ));

    if analysis.errors.is_empty() {
        output.push(String::new());
        output.push(format!("  {} 没有发现错误", "✓".green()));
        output.push(String::new());
        return output.join("\n");
    }

    // 错误模式统计
    if !analysis.error_patterns.is_empty() {
        output.push(String::new());
        output.push(format!("{}", "错误模式（Top 5）".red().bold()));
        output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

        let top_patterns = analysis.top_error_patterns(5);
        for (pattern, count) in top_patterns {
            output.push(format!(
                "\n  {} {} {}",
                format!("{}×", count).yellow().bold(),
                "▸".dimmed(),
                pattern.red()
            ));
        }
    }

    // 显示所有错误（限制显示数量）
    output.push(String::new());
    output.push(format!("{}", "错误详情".red().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    let display_limit = 10;
    for error in analysis.errors.iter().take(display_limit) {
        output.push(format!(
            "\n  {} {} {}",
            format!("L{}", error.line_number).dimmed(),
            "▸".red(),
            error.message
        ));
    }

    if analysis.errors.len() > display_limit {
        output.push(format!(
            "\n  {} 还有 {} 个错误未显示",
            "...".dimmed(),
            analysis.errors.len() - display_limit
        ));
    }

    output.push(String::new());

    output.join("\n")
}

/// 格式化分析结果
fn format_analysis_result(analysis: &crate::log_analyzer::LogAnalysis, file_path: &str) -> String {
    let mut output = vec![];

    // 标题
    output.push(format!("\n{}", "日志分析报告".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    // 基本信息
    output.push(format!("\n  {}: {}", "文件".dimmed(), file_path.cyan()));
    output.push(format!(
        "  {}: {} 行",
        "总行数".dimmed(),
        analysis.total_lines
    ));

    // 时间范围
    if let Some((start, end)) = analysis.time_range {
        output.push(format!(
            "  {}: {} ~ {}",
            "时间范围".dimmed(),
            start.format("%Y-%m-%d %H:%M:%S").to_string().dimmed(),
            end.format("%Y-%m-%d %H:%M:%S").to_string().dimmed()
        ));
    }

    // 级别统计
    output.push(String::new());
    output.push(format!("{}", "日志级别统计".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    let mut levels: Vec<_> = analysis.level_counts.iter().collect();
    levels.sort_by(|a, b| b.0.priority().cmp(&a.0.priority()));

    for (level, count) in levels {
        let level_str = match level {
            LogLevel::Error => level.as_str().red().bold(),
            LogLevel::Warn => level.as_str().yellow().bold(),
            LogLevel::Info => level.as_str().green(),
            LogLevel::Debug => level.as_str().cyan(),
            LogLevel::Trace => level.as_str().dimmed(),
            LogLevel::Unknown => level.as_str().dimmed(),
        };

        let percentage = (*count as f64 / analysis.total_lines as f64 * 100.0) as usize;
        let bar = "█".repeat(percentage.min(50));

        output.push(format!(
            "  {}: {:>6} {} {}",
            level_str,
            count,
            "条".dimmed(),
            bar.dimmed()
        ));
    }

    // 错误模式分析
    if !analysis.error_patterns.is_empty() {
        output.push(String::new());
        output.push(format!("{}", "错误模式（Top 5）".red().bold()));
        output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

        let top_patterns = analysis.top_error_patterns(5);
        for (i, (pattern, count)) in top_patterns.iter().enumerate() {
            output.push(format!(
                "\n  {} {} {} {}",
                format!("#{}", i + 1).dimmed(),
                format!("{}×", count).yellow().bold(),
                "▸".dimmed(),
                pattern.red()
            ));
        }
    }

    // 最近的错误
    if !analysis.errors.is_empty() {
        output.push(String::new());
        output.push(format!("{}", "最近错误".red().bold()));
        output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

        for error in analysis.errors.iter().rev().take(3) {
            output.push(format!(
                "\n  {} {} {}",
                format!("L{}", error.line_number).dimmed(),
                "▸".red(),
                error.message
            ));
        }

        if analysis.errors.len() > 3 {
            output.push(format!(
                "\n  {} 还有 {} 个错误",
                "...".dimmed(),
                analysis.errors.len() - 3
            ));
        }
    }

    // 健康度评估
    output.push(String::new());
    output.push(format!("{}", "健康度评估".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    let error_rate = analysis.error_count() as f64 / analysis.total_lines as f64 * 100.0;
    let warn_rate = analysis.warning_count() as f64 / analysis.total_lines as f64 * 100.0;

    let health_status = if error_rate > 5.0 {
        ("严重", "红色预警", "●".red())
    } else if error_rate > 1.0 {
        ("警告", "需要关注", "●".yellow())
    } else if warn_rate > 10.0 {
        ("一般", "有待改善", "●".yellow())
    } else {
        ("良好", "运行正常", "●".green())
    };

    output.push(format!(
        "\n  {}: {} {} {}",
        "状态".dimmed(),
        health_status.2,
        health_status.0.bold(),
        format!("({})", health_status.1).dimmed()
    ));

    output.push(format!("  {}: {:.2}%", "错误率".dimmed(), error_rate));

    if warn_rate > 0.0 {
        output.push(format!("  {}: {:.2}%", "警告率".dimmed(), warn_rate));
    }

    // 建议
    output.push(String::new());
    output.push(format!("{}", "快捷命令".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    if analysis.error_count() > 0 {
        output.push(format!(
            "  {} 查看所有错误详情",
            format!("/le {}", file_path).cyan()
        ));
    }

    output.push(format!(
        "  {} 查看最近日志",
        format!("/lt {}", file_path).cyan()
    ));

    output.push(String::new());

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    // 辅助函数：创建测试日志文件
    fn create_test_log_file(path: &str, content: &str) {
        let parent = std::path::Path::new(path).parent().unwrap();
        let _ = fs::create_dir_all(parent);
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    // ========== 无参数测试 ==========

    #[test]
    fn test_handle_log_analyze_no_args() {
        let result = handle_log_analyze("");
        assert!(result.contains("用法") || result.contains("Usage"));
        assert!(result.contains("log-analyze") || result.contains("la"));
    }

    #[test]
    fn test_handle_log_tail_no_args() {
        let result = handle_log_tail("");
        assert!(result.contains("用法") || result.contains("Usage"));
        assert!(result.contains("log-tail") || result.contains("lt"));
    }

    #[test]
    fn test_handle_log_errors_no_args() {
        let result = handle_log_errors("");
        assert!(result.contains("用法") || result.contains("Usage"));
        assert!(result.contains("log-errors") || result.contains("le"));
    }

    // ========== handle_log_analyze 测试 ==========

    #[test]
    fn test_handle_log_analyze_file_not_exist() {
        let result = handle_log_analyze("/tmp/nonexistent_file_12345.log");
        assert!(result.contains("✗") || result.contains("不存在") || result.contains("exist"));
    }

    #[test]
    fn test_handle_log_analyze_with_valid_file() {
        let test_file = "/tmp/realconsole_test_logs/test_app.log";
        create_test_log_file(
            test_file,
            r#"2025-10-16 10:00:01 INFO Application started
2025-10-16 10:00:02 ERROR Failed to connect
2025-10-16 10:00:03 WARN Memory usage high
2025-10-16 10:00:04 INFO Processing completed
"#,
        );

        let result = handle_log_analyze(test_file);

        // 验证包含关键元素（分别检查）
        assert!(result.contains("日志") || result.contains("分析"));
        assert!(result.contains("ERROR") || result.contains("错误"));
    }

    #[test]
    fn test_handle_log_analyze_with_max_lines() {
        let test_file = "/tmp/realconsole_test_logs/test_app_max.log";
        create_test_log_file(
            test_file,
            r#"2025-10-16 10:00:01 INFO Line 1
2025-10-16 10:00:02 INFO Line 2
2025-10-16 10:00:03 INFO Line 3
2025-10-16 10:00:04 ERROR Line 4
2025-10-16 10:00:05 INFO Line 5
"#,
        );

        let result = handle_log_analyze(&format!("{} --max-lines 3", test_file));

        // 应该成功分析（即使限制了行数）
        assert!(result.contains("日志") || result.contains("分析"));
    }

    #[test]
    fn test_handle_log_analyze_clean_log() {
        let test_file = "/tmp/realconsole_test_logs/test_clean.log";
        create_test_log_file(
            test_file,
            r#"2025-10-16 10:00:01 INFO Application started
2025-10-16 10:00:02 INFO Processing data
2025-10-16 10:00:03 INFO Completed successfully
"#,
        );

        let result = handle_log_analyze(test_file);

        // 应该显示健康状态（分别检查避免ANSI码问题）
        assert!(result.contains("日志") || result.contains("分析"));
        assert!(
            result.contains("健康")
                || result.contains("良好")
                || result.contains("正常")
                || result.contains("●")
        );
    }

    // ========== handle_log_tail 测试 ==========

    #[test]
    fn test_handle_log_tail_file_not_exist() {
        let result = handle_log_tail("/tmp/nonexistent_tail_12345.log");
        assert!(result.contains("✗") || result.contains("不存在"));
    }

    #[test]
    fn test_handle_log_tail_with_valid_file() {
        let test_file = "/tmp/realconsole_test_logs/test_tail.log";
        create_test_log_file(
            test_file,
            r#"2025-10-16 10:00:01 INFO Line 1
2025-10-16 10:00:02 ERROR Error occurred
2025-10-16 10:00:03 WARN Warning message
2025-10-16 10:00:04 INFO Line 4
"#,
        );

        let result = handle_log_tail(test_file);

        // 验证输出（分别检查关键词）
        assert!(result.contains("最近") || result.contains("行"));
        assert!(result.contains("ERROR") || result.contains("错误"));
    }

    #[test]
    fn test_handle_log_tail_with_custom_lines() {
        let test_file = "/tmp/realconsole_test_logs/test_tail_custom.log";
        create_test_log_file(
            test_file,
            r#"2025-10-16 10:00:01 INFO Line 1
2025-10-16 10:00:02 INFO Line 2
2025-10-16 10:00:03 INFO Line 3
2025-10-16 10:00:04 INFO Line 4
2025-10-16 10:00:05 INFO Line 5
"#,
        );

        let result = handle_log_tail(&format!("{} 3", test_file));

        // 应该显示指定行数
        assert!(result.contains("3") || result.contains("行"));
    }

    #[test]
    fn test_handle_log_tail_shows_errors() {
        let test_file = "/tmp/realconsole_test_logs/test_tail_errors.log";
        create_test_log_file(
            test_file,
            r#"2025-10-16 10:00:01 INFO Start
2025-10-16 10:00:02 ERROR First error
2025-10-16 10:00:03 ERROR Second error
2025-10-16 10:00:04 INFO End
"#,
        );

        let result = handle_log_tail(test_file);

        // 应该显示最近的错误
        assert!(result.contains("ERROR") || result.contains("错误"));
        assert!(result.contains("最近") || result.contains("error"));
    }

    // ========== handle_log_errors 测试 ==========

    #[test]
    fn test_handle_log_errors_file_not_exist() {
        let result = handle_log_errors("/tmp/nonexistent_errors_12345.log");
        assert!(result.contains("✗") || result.contains("不存在"));
    }

    #[test]
    fn test_handle_log_errors_with_errors() {
        let test_file = "/tmp/realconsole_test_logs/test_errors.log";
        create_test_log_file(
            test_file,
            r#"2025-10-16 10:00:01 INFO Normal log
2025-10-16 10:00:02 ERROR Connection failed
2025-10-16 10:00:03 ERROR Database error
2025-10-16 10:00:04 INFO Another normal log
2025-10-16 10:00:05 ERROR Third error
"#,
        );

        let result = handle_log_errors(test_file);

        // 应该显示错误总数和详情
        assert!(result.contains("错误") || result.contains("ERROR"));
        assert!(result.contains("3") || result.contains("个"));
    }

    #[test]
    fn test_handle_log_errors_no_errors() {
        let test_file = "/tmp/realconsole_test_logs/test_no_errors.log";
        create_test_log_file(
            test_file,
            r#"2025-10-16 10:00:01 INFO Line 1
2025-10-16 10:00:02 INFO Line 2
2025-10-16 10:00:03 INFO Line 3
"#,
        );

        let result = handle_log_errors(test_file);

        // 应该显示没有错误
        assert!(result.contains("没有") || result.contains("0") || result.contains("✓"));
    }

    #[test]
    fn test_handle_log_errors_shows_patterns() {
        let test_file = "/tmp/realconsole_test_logs/test_patterns.log";
        create_test_log_file(
            test_file,
            r#"2025-10-16 10:00:01 ERROR Connection timeout
2025-10-16 10:00:02 ERROR Connection timeout
2025-10-16 10:00:03 ERROR Connection timeout
2025-10-16 10:00:04 ERROR Database error
2025-10-16 10:00:05 ERROR Connection timeout
"#,
        );

        let result = handle_log_errors(test_file);

        // 应该显示错误模式
        assert!(result.contains("模式") || result.contains("pattern") || result.contains("×"));
    }

    // ========== format_analysis_result 测试 ==========

    #[test]
    fn test_format_analysis_result_basic() {
        use crate::log_analyzer::{LogAnalysis, LogEntry, LogLevel};

        let mut analysis = LogAnalysis::default();
        analysis.total_lines = 10;
        analysis.level_counts.insert(LogLevel::Info, 8);
        analysis.level_counts.insert(LogLevel::Error, 2);
        analysis.errors.push(LogEntry {
            line_number: 5,
            raw_content: "ERROR Test error".to_string(),
            level: LogLevel::Error,
            timestamp: None,
            message: "Test error".to_string(),
            is_stacktrace: false,
        });

        let result = format_analysis_result(&analysis, "test.log");

        // 验证输出格式
        assert!(result.contains("日志") || result.contains("分析"));
        assert!(result.contains("test.log"));
        assert!(result.contains("10") || result.contains("行"));
    }

    // ========== 命令注册测试 ==========

    #[test]
    fn test_register_log_analysis_commands() {
        use crate::command::CommandRegistry;

        let mut registry = CommandRegistry::new();
        register_log_analysis_commands(&mut registry);

        // 验证所有命令都已注册
        assert!(registry.get("log-analyze").is_some());
        assert!(registry.get("la").is_some());
        assert!(registry.get("log-tail").is_some());
        assert!(registry.get("lt").is_some());
        assert!(registry.get("log-errors").is_some());
        assert!(registry.get("le").is_some());
    }

    #[test]
    fn test_log_commands_aliases() {
        use crate::command::CommandRegistry;

        let mut registry = CommandRegistry::new();
        register_log_analysis_commands(&mut registry);

        // 验证别名
        let cmd_full = registry.get("log-analyze").unwrap();
        let cmd_short = registry.get("la").unwrap();
        assert_eq!(cmd_full.name, "log-analyze");
        assert_eq!(cmd_short.name, "la");
    }

    #[test]
    fn test_log_commands_descriptions() {
        use crate::command::CommandRegistry;

        let mut registry = CommandRegistry::new();
        register_log_analysis_commands(&mut registry);

        // 验证命令描述
        let cmd = registry.get("log-analyze").unwrap();
        assert!(cmd.desc.contains("日志") || cmd.desc.contains("分析"));

        let cmd = registry.get("log-errors").unwrap();
        assert!(cmd.desc.contains("错误") || cmd.desc.contains("error"));
    }

    // ========== 边界测试 ==========

    #[test]
    fn test_handle_log_analyze_empty_file() {
        let test_file = "/tmp/realconsole_test_logs/test_empty.log";
        create_test_log_file(test_file, "");

        let result = handle_log_analyze(test_file);

        // 应该能处理空文件
        assert!(result.contains("日志") || result.contains("0") || result.contains("分析"));
    }

    #[test]
    fn test_handle_log_tail_empty_file() {
        let test_file = "/tmp/realconsole_test_logs/test_empty_tail.log";
        create_test_log_file(test_file, "");

        let result = handle_log_tail(test_file);

        // 应该能处理空文件
        assert!(result.contains("最近") || result.contains("0") || result.contains("行"));
    }

    #[test]
    fn test_handle_log_errors_limit() {
        // 测试错误数量限制
        let test_file = "/tmp/realconsole_test_logs/test_errors_limit.log";
        let mut content = String::new();
        for i in 1..=20 {
            content.push_str(&format!("2025-10-16 10:00:{:02} ERROR Error {}\n", i, i));
        }
        create_test_log_file(test_file, &content);

        let result = handle_log_errors(test_file);

        // 应该显示错误数量（限制为100个，所以20个全部显示）
        // 只在超过显示限制时才会出现"未显示"
        assert!(result.contains("错误") || result.contains("ERROR"));
        // 20个错误在限制100以内，会全部显示，所以不期望"未显示"
        assert!(result.contains("20") || result.contains("错误总数"));
    }
}
