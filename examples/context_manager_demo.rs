//! ContextManager 功能演示
//!
//! 运行方式：
//! ```bash
//! cargo run --example context_manager_demo
//! ```

use realconsole::config::{ContextMode, ConversationConfig};
use realconsole::conversation::{ContextManager, Turn};

fn main() {
    println!("=== RealConsole ContextManager 功能演示 ===\n");

    // ========== 场景 1: Disabled 模式 ==========
    println!("【场景 1】Disabled 模式（关闭上下文）");
    println!("────────────────────────────────────────");
    demo_disabled_mode();
    println!();

    // ========== 场景 2: Manual 模式 ==========
    println!("【场景 2】Manual 模式（手动控制）");
    println!("────────────────────────────────────────");
    demo_manual_mode();
    println!();

    // ========== 场景 3: Auto 模式 ==========
    println!("【场景 3】Auto 模式（智能检测）");
    println!("────────────────────────────────────────");
    demo_auto_mode();
    println!();

    // ========== 场景 4: 轮次限制 ==========
    println!("【场景 4】轮次限制测试");
    println!("────────────────────────────────────────");
    demo_turn_limits();
    println!();

    // ========== 场景 5: 消息构建 ==========
    println!("【场景 5】消息构建测试");
    println!("────────────────────────────────────────");
    demo_message_building();
    println!();

    println!("=== 演示完成 ===");
}

/// 场景 1: Disabled 模式
fn demo_disabled_mode() {
    let mut config = ConversationConfig::default();
    config.mode = ContextMode::Disabled;
    let mut manager = ContextManager::new(config);

    println!("✓ 创建 ContextManager (Disabled 模式)");
    println!("  模式: {}", manager.mode());
    println!("  活跃: {}", manager.is_active());

    // 尝试各种输入，都不会启用上下文
    let inputs = vec!["显示它的内容", "为什么", "继续"];

    for input in inputs {
        let should_use = manager.should_use_context(input);
        println!("  输入: \"{}\" → 使用上下文: {}", input, should_use);
    }

    println!("\n✅ Disabled 模式：所有输入都不使用上下文");
}

/// 场景 2: Manual 模式
fn demo_manual_mode() {
    let mut config = ConversationConfig::default();
    config.mode = ContextMode::Manual;
    config.max_turns = 5;
    let max_turns = config.max_turns; // 保存值
    let mut manager = ContextManager::new(config);

    println!("✓ 创建 ContextManager (Manual 模式)");
    println!("  模式: {}", manager.mode());
    println!("  活跃: {}", manager.is_active());

    // 启动上下文
    println!("\n→ 执行: /context start");
    manager.start();
    println!("  活跃: {}", manager.is_active());

    // 添加对话轮次
    println!("\n→ 添加对话轮次:");
    manager.add_turn(Turn::new(
        "分析 error.log 中的错误".to_string(),
        "发现 3 种错误类型".to_string(),
    ));
    println!("  轮次数: {}", manager.turn_count());

    manager.add_turn(Turn::new(
        "统计每种错误的数量".to_string(),
        "TypeError: 15次, ValueError: 8次".to_string(),
    ));
    println!("  轮次数: {}", manager.turn_count());

    // 查看状态
    println!("\n→ 执行: /context show");
    println!("  轮次数: {}/{}", manager.turn_count(), max_turns);
    println!("  上下文长度: {} 字符", manager.context_length());

    // 清除上下文
    println!("\n→ 执行: /context clear");
    manager.clear();
    println!("  轮次数: {}", manager.turn_count());
    println!("  活跃: {}", manager.is_active());

    // 停止上下文
    println!("\n→ 执行: /context stop");
    manager.stop();
    println!("  活跃: {}", manager.is_active());

    println!("\n✅ Manual 模式：完全由用户控制");
}

/// 场景 3: Auto 模式
fn demo_auto_mode() {
    let mut config = ConversationConfig::default();
    config.mode = ContextMode::Auto;
    config.max_turns = 5;
    let mut manager = ContextManager::new(config);

    println!("✓ 创建 ContextManager (Auto 模式)");
    println!("  模式: {}", manager.mode());

    // 测试各种触发条件
    let test_cases = vec![
        ("列出当前目录的文件", "普通命令"),
        ("显示它们的大小", "代词检测 (它们)"),
        ("为什么文件这么大", "追问检测 (为什么)"),
        ("刚才说的是什么", "引用检测 (刚才)"),
        ("列出文件", "继续使用上下文"),
    ];

    println!("\n→ 智能检测测试:");
    for (input, description) in test_cases {
        let should_use = manager.should_use_context(input);
        let status = if should_use { "✓" } else { "✗" };
        println!(
            "  {} \"{}\" → {} ({})",
            status, input, should_use, description
        );

        if should_use {
            println!("     [上下文已激活]");
        }
    }

    println!("\n✅ Auto 模式：智能检测并自动管理上下文");
}

/// 场景 4: 轮次限制
fn demo_turn_limits() {
    let mut config = ConversationConfig::default();
    config.mode = ContextMode::Manual;
    config.max_turns = 3; // 最多保留 3 轮
    let max_turns = config.max_turns; // 保存值
    let mut manager = ContextManager::new(config);

    manager.start();

    println!("✓ 创建 ContextManager (max_turns: {})", max_turns);

    // 添加 5 轮对话
    println!("\n→ 添加 5 轮对话:");
    for i in 1..=5 {
        manager.add_turn(Turn::new(
            format!("输入 {}", i),
            format!("响应 {}", i),
        ));
        println!(
            "  添加第 {} 轮 → 当前轮次数: {}",
            i,
            manager.turn_count()
        );
    }

    // 验证只保留最后 3 轮
    println!("\n→ 验证轮次限制:");
    let turns = manager.turns();
    println!("  保留的轮次:");
    for (idx, turn) in turns.iter().enumerate() {
        println!("    [{}] {}", idx + 1, turn.user_input);
    }

    println!("\n✅ 轮次限制：自动移除最早的轮次");
}

/// 场景 5: 消息构建
fn demo_message_building() {
    let mut config = ConversationConfig::default();
    config.mode = ContextMode::Manual;
    let mut manager = ContextManager::new(config);

    manager.start();

    println!("✓ 创建 ContextManager");

    // 添加历史对话
    println!("\n→ 添加历史对话:");
    manager.add_turn(Turn::new("你好".to_string(), "你好！我是 AI 助手".to_string()));
    println!("  添加: 你好 → 你好！我是 AI 助手");

    manager.add_turn(Turn::new(
        "你能做什么".to_string(),
        "我可以帮你执行命令、分析数据等".to_string(),
    ));
    println!("  添加: 你能做什么 → 我可以帮你执行命令、分析数据等");

    // 构建消息列表
    println!("\n→ 构建发送给 LLM 的消息列表:");
    let messages = manager.build_messages("帮我分析日志");

    println!("  消息数量: {}", messages.len());
    for (idx, msg) in messages.iter().enumerate() {
        let preview = msg
            .content
            .as_ref()
            .map(|c| {
                if c.len() > 30 {
                    format!("{}...", &c[..30])
                } else {
                    c.clone()
                }
            })
            .unwrap_or_else(|| "(无内容)".to_string());

        println!("  [{}] {:?}: {}", idx + 1, msg.role, preview);
    }

    println!("\n✅ 消息构建：将历史轮次转换为 LLM API 格式");
}
