//! Spinner 显示效果演示
//!
//! 运行方式：
//! ```bash
//! cargo run --example spinner_demo
//! ```

use realconsole::spinner::{simplify_model_name, Spinner};
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== RealConsole Spinner 显示效果演示 ===\n");

    // 场景 1: 无标签 Spinner
    println!("场景 1: 无标签 Spinner（模拟快速响应）");
    let spinner = Spinner::new();
    thread::sleep(Duration::from_secs(2));
    spinner.stop();
    println!("✓ 完成\n");

    thread::sleep(Duration::from_millis(500));

    // 场景 2: 带 Deepseek 标签
    println!("场景 2: Deepseek 模型（模拟中等响应）");
    let model = simplify_model_name("deepseek-chat");
    let spinner = Spinner::with_label(&model);
    thread::sleep(Duration::from_secs(3));
    spinner.stop();
    println!("✓ 完成\n");

    thread::sleep(Duration::from_millis(500));

    // 场景 3: 带 Ollama 本地模型标签
    println!("场景 3: Ollama 本地模型（qwen2.5）");
    let model = simplify_model_name("qwen2.5:latest");
    let spinner = Spinner::with_label(&model);
    thread::sleep(Duration::from_secs(3));
    spinner.stop();
    println!("✓ 完成\n");

    thread::sleep(Duration::from_millis(500));

    // 场景 4: Claude 模型
    println!("场景 4: Claude 模型（模拟长时间响应）");
    let model = simplify_model_name("claude-3-opus-20240229");
    let spinner = Spinner::with_label(&model);
    thread::sleep(Duration::from_secs(4));
    spinner.stop();
    println!("✓ 完成\n");

    thread::sleep(Duration::from_millis(500));

    // 场景 5: GPT 模型
    println!("场景 5: GPT 模型");
    let model = simplify_model_name("gpt-4-turbo-preview");
    let spinner = Spinner::with_label(&model);
    thread::sleep(Duration::from_secs(3));
    spinner.stop();
    println!("✓ 完成\n");

    println!("=== 演示完成 ===");
    println!("\nSpinner 特性：");
    println!("  • 橙色旋转飞轮（⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏）");
    println!("  • 80ms 刷新间隔，流畅动画");
    println!("  • 模型名称自动简化并以灰色显示");
    println!("  • 极简主义设计，不影响阅读");
}
