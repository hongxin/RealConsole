//! 多轮对话演示
//!
//! 展示如何使用 RealConsole 的智能参数补全功能
//!
//! 运行方式：
//! ```bash
//! cargo run --example conversation_demo
//! ```

use realconsole::conversation::{
    ConversationManager, ParameterAnalyzer, ParameterSpec, ParameterType, ParameterValue,
};

#[tokio::main]
async fn main() {
    println!("🎯 RealConsole 多轮对话演示");
    println!("{}", "=".repeat(60));
    println!();

    // 演示场景：日志分析任务
    demo_log_analysis_scenario().await;

    println!();
    println!("{}", "=".repeat(60));
    println!("✅ 演示完成！");
}

/// 演示场景：日志分析
///
/// 用户想要分析日志文件，系统会智能地询问缺失的参数
async fn demo_log_analysis_scenario() {
    println!("📋 场景：智能日志分析");
    println!("{}", "-".repeat(60));
    println!();

    // 1. 创建会话管理器
    let mut manager = ConversationManager::new(300); // 5分钟超时
    println!("1️⃣ 创建会话管理器");

    // 2. 启动对话
    let conversation_id = manager
        .start_conversation("analyze_logs")
        .expect("启动对话失败");
    println!("2️⃣ 启动对话 ID: {}", conversation_id);
    println!();

    // 3. 定义需要收集的参数
    let params = vec![
        ParameterSpec::new("log_file", ParameterType::Path, "日志文件路径")
            .with_hint("请提供日志文件的完整路径")
            .with_example("/var/log/application.log"),
        ParameterSpec::new("keyword", ParameterType::String, "搜索关键词")
            .with_hint("支持正则表达式")
            .with_example("ERROR|WARN"),
        ParameterSpec::new(
            "time_range",
            ParameterType::String,
            "时间范围（可选）",
        )
        .optional()
        .with_hint("格式：YYYY-MM-DD 或 '最近24小时'")
        .with_example("2025-01-15"),
    ];

    println!("3️⃣ 添加参数规格：");
    for (i, spec) in params.iter().enumerate() {
        manager
            .add_parameter_spec(&conversation_id, spec.clone())
            .expect("添加参数失败");
        println!(
            "   {}. {} ({:?}) {}",
            i + 1,
            spec.name,
            spec.param_type,
            if spec.is_optional { "[可选]" } else { "[必需]" }
        );
    }
    println!();

    // 4. 模拟用户输入（不完整的）
    let user_input_1 = "我想分析 /var/log/app.log 中的错误";
    println!("4️⃣ 用户输入：{}", user_input_1);
    println!();

    // 5. 分析器可以从输入中提取参数
    // let _analyzer = ParameterAnalyzer::new();
    println!("5️⃣ 参数分析：");

    // 模拟参数提取（实际使用中会调用 LLM）
    println!("   ✓ 从用户输入中识别到：");
    println!("     - log_file = /var/log/app.log");
    println!("     - keyword = 错误");
    println!();

    // 6. 收集第一个参数
    manager
        .collect_parameter(
            &conversation_id,
            "log_file",
            ParameterValue::String("/var/log/app.log".to_string()),
        )
        .expect("收集参数失败");
    println!("6️⃣ 收集参数: log_file = /var/log/app.log");

    // 7. 收集第二个参数
    manager
        .collect_parameter(
            &conversation_id,
            "keyword",
            ParameterValue::String("ERROR".to_string()),
        )
        .expect("收集参数失败");
    println!("7️⃣ 收集参数: keyword = ERROR");
    println!();

    // 8. 检查是否所有必需参数已收集
    let missing = manager
        .detect_missing_parameters(&conversation_id)
        .expect("检测失败");

    if missing.is_empty() {
        println!("8️⃣ ✅ 所有必需参数已收集完成！");
        println!();

        // 9. 显示收集到的参数
        let context = manager.get_context(&conversation_id).unwrap();
        println!("9️⃣ 参数摘要：");
        for (name, value) in &context.parameters {
            println!("   • {} = {:?}", name, value);
        }
        println!();

        // 10. 状态转换：收集参数 -> 验证 -> 确认 -> 执行
        println!("🔨 准备执行日志分析命令...");

        // 10.1 转换到验证状态
        {
            let context = manager.get_context_mut(&conversation_id).unwrap();
            context.state
                .transition(realconsole::conversation::StateEvent::AllParametersCollected)
                .expect("转换到验证状态失败");
            println!("   ✓ 转换到验证状态");
        }

        // 10.2 验证通过，转换到确认状态
        {
            let context = manager.get_context_mut(&conversation_id).unwrap();
            context.state
                .transition(realconsole::conversation::StateEvent::ValidationPassed)
                .expect("转换到确认状态失败");
            println!("   ✓ 验证通过，转换到确认状态");
        }

        // 10.3 用户确认，转换到执行状态
        let result = manager
            .confirm_execution(&conversation_id, true)
            .expect("确认执行失败");

        if matches!(result, realconsole::conversation::Response::ReadyToExecute) {
            println!("   ✓ 用户确认，开始执行");

            let command = format!(
                "grep -i 'ERROR' /var/log/app.log | tail -50"
            );
            println!("   命令: {}", command);
            println!();

            // 10.4 执行完成
            manager
                .complete_execution(&conversation_id, true, "分析完成".to_string())
                .expect("标记完成失败");
            println!("✅ 对话已完成");
        }
    } else {
        println!("8️⃣ ⚠️ 还有 {} 个参数未收集", missing.len());
        for spec in &missing {
            println!("   - {}: {}", spec.name, spec.description);
        }
    }
}

/// 演示参数验证功能
#[allow(dead_code)]
fn demo_parameter_validation() {
    let analyzer = ParameterAnalyzer::new();

    println!("🔍 参数验证演示");
    println!("{}", "-".repeat(60));

    // 测试字符串类型
    let spec = ParameterSpec::new("name", ParameterType::String, "用户名");
    let value = ParameterValue::String("Alice".to_string());
    match analyzer.validate_parameter(&spec, &value) {
        Ok(()) => println!("✅ 字符串参数验证通过"),
        Err(e) => println!("❌ 验证失败: {}", e),
    }

    // 测试整数类型
    let spec = ParameterSpec::new("age", ParameterType::Integer, "年龄");
    let value = ParameterValue::Integer(25);
    match analyzer.validate_parameter(&spec, &value) {
        Ok(()) => println!("✅ 整数参数验证通过"),
        Err(e) => println!("❌ 验证失败: {}", e),
    }

    // 测试类型不匹配
    let spec = ParameterSpec::new("count", ParameterType::Integer, "数量");
    let value = ParameterValue::String("not_a_number".to_string());
    match analyzer.validate_parameter(&spec, &value) {
        Ok(()) => println!("✅ 参数验证通过"),
        Err(e) => println!("❌ 类型不匹配（预期）: {}", e),
    }
}
