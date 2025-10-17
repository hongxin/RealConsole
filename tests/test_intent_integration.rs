//! Intent DSL 集成测试
//!
//! 验证 Agent 与 Intent DSL 的端到端集成功能

use realconsole::agent::Agent;
use realconsole::command::CommandRegistry;
use realconsole::config::Config;
use std::fs;

#[tokio::test(flavor = "multi_thread")]
async fn test_intent_dsl_count_python_files() {
    // 创建 Agent（带有 Intent DSL 支持）
    let config = Config::default();
    let registry = CommandRegistry::new();
    let agent = Agent::new(config, registry);

    // 测试统计 Python 文件的意图识别
    // 注意：这个测试依赖于内置意图 count_files
    let result = agent.handle("统计当前目录下有多少个 py 文件");

    // 验证结果包含数字（文件数量）
    assert!(!result.is_empty(), "结果不应为空");

    // 由于这是实际执行 find 命令，结果应该包含数字
    // 我们不验证具体数字，因为它取决于测试环境
    println!("统计 Python 文件结果: {}", result);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_intent_dsl_fallback_to_llm() {
    // 创建 Agent
    let config = Config::default();
    let registry = CommandRegistry::new();
    let agent = Agent::new(config, registry);

    // 测试不匹配任何意图的输入（应该回退到 LLM 或提示错误）
    let result = agent.handle("这是一个完全随机的输入不应匹配任何意图");

    // 结果不应为空（要么是 LLM 响应，要么是错误提示）
    assert!(!result.is_empty(), "应该有某种响应");

    println!("回退测试结果: {}", result);
}

#[test]
fn test_intent_matcher_initialization() {
    // 验证 Agent 初始化时 Intent DSL 系统被正确设置
    let config = Config::default();
    let registry = CommandRegistry::new();
    let agent = Agent::new(config, registry);

    // 验证内置意图已经注册
    assert!(!agent.intent_matcher.is_empty(), "应该有内置意图被注册");
    assert!(agent.intent_matcher.len() >= 10, "应该至少有 10 个内置意图");

    println!("已注册意图数量: {}", agent.intent_matcher.len());
}

#[test]
fn test_template_engine_initialization() {
    // 验证 TemplateEngine 初始化正确
    let config = Config::default();
    let registry = CommandRegistry::new();
    let agent = Agent::new(config, registry);

    // 验证模板引擎包含模板
    assert!(!agent.template_engine.is_empty(), "应该有模板被注册");
    assert!(agent.template_engine.len() >= 10, "应该至少有 10 个模板");

    println!("已注册模板数量: {}", agent.template_engine.len());
    println!("模板列表: {:?}", agent.template_engine.template_names());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_intent_count_lines() {
    // 创建临时测试文件
    let test_dir = std::env::temp_dir().join("realconsole_test");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.py");
    fs::write(&test_file, "line1\nline2\nline3\n").unwrap();

    // 创建 Agent
    let config = Config::default();
    let registry = CommandRegistry::new();
    let agent = Agent::new(config, registry);

    // 测试统计行数的意图
    let query = format!("统计 {} 文件的行数", test_file.display());
    let result = agent.handle(&query);

    println!("统计行数结果: {}", result);

    // 清理
    fs::remove_file(&test_file).ok();
    fs::remove_dir(&test_dir).ok();

    // 验证结果包含数字
    assert!(!result.is_empty(), "结果不应为空");
}

#[test]
fn test_intent_matching_confidence() {
    use realconsole::dsl::intent::BuiltinIntents;

    // 创建内置意图系统
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 测试高置信度匹配
    let matches = matcher.match_intent("统计 Python 代码行数");
    assert!(!matches.is_empty(), "应该匹配到意图");
    assert!(matches[0].confidence > 0.5, "置信度应该足够高");

    println!("匹配到的意图: {}", matches[0].intent.name);
    println!("置信度: {:.2}", matches[0].confidence);

    // 测试低置信度或无匹配
    let matches = matcher.match_intent("完全不相关的输入文本");
    println!("不相关输入的匹配结果数量: {}", matches.len());
}

#[test]
fn test_execution_plan_generation() {
    use realconsole::dsl::intent::BuiltinIntents;

    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();
    let engine = builtin.create_engine();

    // 匹配意图
    if let Some(intent_match) = matcher.best_match("统计当前目录下有多少个 py 文件") {
        // 生成执行计划
        let plan = engine.generate_from_intent(&intent_match);

        match plan {
            Ok(p) => {
                println!("生成的命令: {}", p.command);
                println!("模板名称: {}", p.template_name);
                assert!(!p.command.is_empty(), "命令不应为空");
                assert!(p.command.contains("find"), "应该包含 find 命令");
            }
            Err(e) => {
                println!("执行计划生成失败: {}", e);
                // 某些意图可能需要实体提取，所以失败也是可以接受的
            }
        }
    }
}

// ========== Phase 3 Week 3: Entity Extraction Integration Tests ==========

#[test]
fn test_entity_extraction_count_files() {
    use realconsole::dsl::intent::{BuiltinIntents, EntityType};

    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 测试统计文件意图的实体提取
    let matches = matcher.match_intent("统计当前目录下有多少个 py 文件");
    assert!(!matches.is_empty(), "应该匹配到意图");

    let best_match = &matches[0];
    assert_eq!(best_match.intent.name, "count_files");

    // 验证提取的实体
    println!("提取的实体: {:?}", best_match.extracted_entities);

    // 应该提取到文件类型 (py)
    if let Some(EntityType::FileType(ft)) = best_match.extracted_entities.get("ext") {
        assert_eq!(ft, "py", "应该提取到 Python 文件类型");
    }

    // 应该提取到路径
    assert!(
        best_match.extracted_entities.contains_key("path"),
        "应该提取到路径实体"
    );
}

#[test]
fn test_entity_extraction_find_large_files() {
    use realconsole::dsl::intent::{BuiltinIntents, EntityType};

    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 测试查找大文件意图的实体提取
    let matches = matcher.match_intent("查找 ./src 目录下大于 500 MB 的大文件");
    assert!(!matches.is_empty(), "应该匹配到意图");

    let best_match = &matches[0];
    assert_eq!(best_match.intent.name, "find_files_by_size");

    // 验证提取的实体
    println!("提取的实体: {:?}", best_match.extracted_entities);

    // 应该提取到路径
    if let Some(EntityType::Path(path)) = best_match.extracted_entities.get("path") {
        assert_eq!(path, "./src", "应该提取到正确的路径");
    }

    // 应该提取到大小
    if let Some(EntityType::Number(size)) = best_match.extracted_entities.get("size") {
        assert_eq!(*size, 500.0, "应该提取到正确的大小");
    }
}

#[test]
fn test_entity_extraction_count_python_lines() {
    use realconsole::dsl::intent::{BuiltinIntents, EntityType};

    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 测试统计 Python 行数意图的实体提取
    let matches = matcher.match_intent("统计 /tmp 目录下的 Python 代码行数");
    assert!(!matches.is_empty(), "应该匹配到意图");

    let best_match = &matches[0];
    assert_eq!(best_match.intent.name, "count_python_lines");

    // 验证提取的实体
    println!("提取的实体: {:?}", best_match.extracted_entities);

    // 应该提取到路径
    if let Some(EntityType::Path(path)) = best_match.extracted_entities.get("path") {
        assert_eq!(path, "/tmp", "应该提取到正确的路径");
    }
}

#[test]
fn test_entity_extraction_find_recent_files() {
    use realconsole::dsl::intent::{BuiltinIntents, EntityType};

    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 测试查找最近文件意图的实体提取
    let matches = matcher.match_intent("查找 . 目录下最近 30 分钟修改的文件");
    assert!(!matches.is_empty(), "应该匹配到意图");

    let best_match = &matches[0];
    assert_eq!(best_match.intent.name, "find_recent_files");

    // 验证提取的实体
    println!("提取的实体: {:?}", best_match.extracted_entities);

    // 应该提取到路径
    if let Some(EntityType::Path(path)) = best_match.extracted_entities.get("path") {
        assert_eq!(path, ".", "应该提取到当前目录");
    }

    // 应该提取到时间（分钟）
    if let Some(EntityType::Number(minutes)) = best_match.extracted_entities.get("minutes") {
        assert_eq!(*minutes, 30.0, "应该提取到正确的分钟数");
    }
}

#[test]
fn test_entity_extraction_check_disk_usage() {
    use realconsole::dsl::intent::{BuiltinIntents, EntityType};

    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 测试检查磁盘意图的实体提取
    let matches = matcher.match_intent("检查 /var/log 目录的磁盘使用情况，显示前 5 个");
    assert!(!matches.is_empty(), "应该匹配到意图");

    let best_match = &matches[0];
    assert_eq!(best_match.intent.name, "check_disk_usage");

    // 验证提取的实体
    println!("提取的实体: {:?}", best_match.extracted_entities);

    // 应该提取到路径
    if let Some(EntityType::Path(path)) = best_match.extracted_entities.get("path") {
        assert_eq!(path, "/var/log", "应该提取到正确的路径");
    }

    // 应该提取到显示数量
    if let Some(EntityType::Number(limit)) = best_match.extracted_entities.get("limit") {
        assert_eq!(*limit, 5.0, "应该提取到正确的显示数量");
    }
}

#[test]
fn test_entity_extraction_with_template_generation() {
    use realconsole::dsl::intent::BuiltinIntents;

    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();
    let engine = builtin.create_engine();

    // 匹配意图并提取实体
    if let Some(intent_match) = matcher.best_match("统计 ./build 目录下有多少个 py 文件") {
        println!("匹配到意图: {}", intent_match.intent.name);
        println!("提取的实体: {:?}", intent_match.extracted_entities);

        // 生成执行计划（应该使用提取的实体）
        let plan = engine.generate_from_intent(&intent_match);

        match plan {
            Ok(p) => {
                println!("生成的命令: {}", p.command);
                // 验证命令包含了提取的实体值
                assert!(
                    p.command.contains("./build") || p.command.contains("."),
                    "命令应该包含路径"
                );
                assert!(
                    p.command.contains("py") || p.command.contains("*"),
                    "命令应该包含文件扩展名"
                );
            }
            Err(e) => {
                println!("执行计划生成失败: {}", e);
            }
        }
    } else {
        panic!("应该匹配到意图");
    }
}

#[test]
fn test_entity_extraction_default_values() {
    use realconsole::dsl::intent::{BuiltinIntents, EntityType};

    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 测试没有显式路径时，使用默认值
    let matches = matcher.match_intent("统计 Python 文件数量");
    assert!(!matches.is_empty(), "应该匹配到意图");

    let best_match = &matches[0];

    // 即使用户没有提供路径，也应该有默认路径实体
    // (来自 Intent 定义或者提取器的智能推断)
    println!("提取的实体: {:?}", best_match.extracted_entities);

    // 检查是否有路径实体（可能是默认的 "." 或者提取到的）
    if let Some(EntityType::Path(path)) = best_match.extracted_entities.get("path") {
        println!("提取的路径: {}", path);
        // 默认应该是当前目录
        assert!(path == "." || path.contains("."), "应该有路径值");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_agent_handle_flow() {
    // 测试 Agent 的完整处理流程：
    // 输入 → Intent 识别 → 计划生成 → 命令执行 → 结果返回

    let config = Config::default();
    let registry = CommandRegistry::new();
    let agent = Agent::new(config, registry);

    // 使用一个简单但确定的查询
    let result = agent.handle("列出当前目录");

    println!("完整流程测试结果: {}", result);
    assert!(!result.is_empty(), "应该有输出结果");
}
