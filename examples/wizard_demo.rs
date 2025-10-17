//! 配置向导演示程序
//!
//! 运行方式:
//! ```bash
//! cargo run --example wizard_demo
//! ```
//!
//! 或在 sandbox 目录中：
//! ```bash
//! cd sandbox/wizard-test
//! cargo run --example wizard_demo
//! ```

use realconsole::wizard::{ConfigWizard, WizardMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== RealConsole 配置向导演示 ===\n");
    println!("提示: 这是一个演示程序，生成的配置文件在当前目录\n");

    // 创建快速配置模式的向导
    let wizard = ConfigWizard::new(WizardMode::Quick);

    // 运行向导
    match wizard.run().await {
        Ok(config) => {
            println!("\n配置收集完成！\n");

            // 生成并保存配置文件
            if let Err(e) = wizard.generate_and_save(&config) {
                eprintln!("✗ 保存配置失败: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("\n✗ 配置向导失败: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
