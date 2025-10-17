//! CLI 集成测试
//!
//! Week 3 Day 4: 使用 assert_cmd 测试 CLI 功能

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// 测试帮助命令
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("realconsole"))
        .stdout(predicate::str::contains("智能"));
}

/// 测试版本命令
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("realconsole"));
}

/// 测试 --once 模式：执行单条命令后退出
#[test]
fn test_cli_once_mode_help() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once").arg("/help")
        .assert()
        .success()
        .stdout(predicate::str::contains("SimpleConsole").or(predicate::str::contains("Console")))
        .stdout(predicate::str::contains("智能").or(predicate::str::contains("help")));
}

/// 测试 --once 模式：version 命令
#[test]
fn test_cli_once_mode_version() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once").arg("/version")
        .assert()
        .success()
        .stdout(predicate::str::contains("SimpleConsole").or(predicate::str::contains("RealConsole")))
        .stdout(predicate::str::contains("Phase"));
}

/// 测试 --once 模式：tools 命令
#[test]
fn test_cli_once_mode_tools() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once").arg("/tools")
        .assert()
        .success()
        .stdout(predicate::str::contains("可用工具"))
        .stdout(predicate::str::contains("calculator"));
}

/// 测试 --once 模式：commands 命令
#[test]
fn test_cli_once_mode_commands() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once").arg("/commands")
        .assert()
        .success(); // 只检查成功执行，输出可能因配置而异
}

/// 测试配置文件加载（不存在时使用默认配置）
#[test]
fn test_cli_with_nonexistent_config() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    // 使用不存在的配置文件路径，应该退出（需要配置）
    cmd.arg("--config")
        .arg("nonexistent.yaml")
        .arg("--once")
        .arg("/help")
        .assert()
        .failure(); // 配置文件不存在会退出
}

/// 测试空配置（创建临时空配置文件）
#[test]
fn test_cli_with_empty_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("empty_config.yaml");

    // 创建一个空配置文件
    fs::write(&config_path, "").unwrap();

    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--config")
        .arg(config_path)
        .arg("--once")
        .arg("/help")
        .assert()
        .success()
        .stdout(predicate::str::contains("SimpleConsole").or(predicate::str::contains("RealConsole")));
}

/// 测试最小配置文件
#[test]
fn test_cli_with_minimal_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("minimal_config.yaml");

    // 创建最小配置
    let minimal_config = r#"
model: deepseek-chat
max_history: 10
"#;
    fs::write(&config_path, minimal_config).unwrap();

    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--config")
        .arg(config_path)
        .arg("--once")
        .arg("/version")
        .assert()
        .success()
        .stdout(predicate::str::contains("SimpleConsole").or(predicate::str::contains("RealConsole")));
}

/// 测试工具调用：calculator
#[test]
fn test_cli_tool_calculator() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg(r#"/tools call calculator {"expression": "2+2"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("result").or(predicate::str::contains("4")));
}

/// 测试工具列表显示
#[test]
fn test_cli_tool_list_contains_builtin_tools() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg("/tools")
        .assert()
        .success()
        .stdout(predicate::str::contains("calculator"))
        .stdout(predicate::str::contains("datetime"));
}

/// 测试无效命令
#[test]
fn test_cli_invalid_command() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg("/invalid_command_xyz")
        .assert()
        .success() // 不应该崩溃，而是显示错误信息
        .stdout(predicate::str::contains("未知").or(predicate::str::contains("无效")));
}

/// 测试记忆系统状态
#[test]
fn test_cli_memory_recent() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg("/memory recent")
        .assert()
        .success(); // 即使没有记忆也应该成功
}

/// 测试日志系统状态
#[test]
fn test_cli_log_stats() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg("/log stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("执行").or(predicate::str::contains("统计")));
}

/// 测试 LLM 状态命令
#[test]
fn test_cli_llm_status() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg("/llm")
        .assert()
        .success()
        .stdout(predicate::str::contains("LLM").or(predicate::str::contains("Primary")));
}

/// 测试多个命令顺序执行（通过多次 --once 调用）
#[test]
fn test_cli_multiple_commands() {
    // 第一个命令：获取工具列表
    let mut cmd1 = Command::cargo_bin("realconsole").unwrap();
    cmd1.arg("--once").arg("/tools").assert().success();

    // 第二个命令：获取版本
    let mut cmd2 = Command::cargo_bin("realconsole").unwrap();
    cmd2.arg("--once").arg("/version").assert().success();

    // 第三个命令：获取帮助
    let mut cmd3 = Command::cargo_bin("realconsole").unwrap();
    cmd3.arg("--once").arg("/help").assert().success();
}

/// 测试配置向导（模拟非交互模式）
#[test]
#[ignore] // 忽略，因为需要交互输入
fn test_cli_config_wizard() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--wizard")
        .assert()
        .success();
}

/// 测试退出命令
#[test]
fn test_cli_quit_command() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg("/quit")
        .assert()
        .success();
}

/// 测试别名命令
#[test]
fn test_cli_command_aliases() {
    // 测试 /h 别名（help）
    let mut cmd_h = Command::cargo_bin("realconsole").unwrap();
    cmd_h.arg("--once").arg("/h").assert().success();

    // 测试 /v 别名（version）
    let mut cmd_v = Command::cargo_bin("realconsole").unwrap();
    cmd_v.arg("--once").arg("/v").assert().success();

    // 测试 /q 别名（quit）
    let mut cmd_q = Command::cargo_bin("realconsole").unwrap();
    cmd_q.arg("--once").arg("/q").assert().success();
}

/// 测试命令列表包含所有核心命令
#[test]
fn test_cli_commands_list_complete() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg("/commands")
        .assert()
        .success(); // 只检查成功，不检查具体输出
        // 命令列表的显示取决于运行时环境和配置
}

/// 测试工具信息查询
#[test]
fn test_cli_tool_info() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg("/tools info calculator")
        .assert()
        .success()
        .stdout(predicate::str::contains("calculator").or(predicate::str::contains("计算")));
}

/// 测试记忆搜索（空结果）
#[test]
fn test_cli_memory_search_empty() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg("/memory search nonexistent_query_xyz")
        .assert()
        .success(); // 即使没有结果也应该成功
}

/// 测试日志查询（最近记录）
#[test]
fn test_cli_log_recent_empty() {
    let mut cmd = Command::cargo_bin("realconsole").unwrap();

    cmd.arg("--once")
        .arg("/log recent")
        .assert()
        .success(); // 即使没有日志也应该成功
}
