//! 极简主义 Spinner
//!
//! 在 LLM 计算时显示橙色旋转飞轮，可选显示模型名称

use colored::Colorize;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Spinner 符号序列（旋转飞轮）
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// 简化模型名称显示
///
/// 将冗长的模型名称简化为更简短的形式，适合在 spinner 中显示
///
/// # 示例
/// ```
/// # use realconsole::spinner::simplify_model_name;
/// assert_eq!(simplify_model_name("deepseek-chat"), "deepseek");
/// assert_eq!(simplify_model_name("qwen2.5:latest"), "qwen2.5");
/// assert_eq!(simplify_model_name("gpt-4-turbo-preview"), "gpt-4");
/// assert_eq!(simplify_model_name("claude-3-opus-20240229"), "claude-3");
/// ```
pub fn simplify_model_name(model: &str) -> String {
    // 移除常见后缀
    let simplified = model
        .trim()
        .replace(":latest", "")
        .replace(":stable", "")
        .replace("-chat", "")
        .replace("-turbo", "")
        .replace("-preview", "");

    // 先移除日期后缀（如 20240229）
    let mut result = simplified.clone();
    if let Some(pos) = result.rfind('-') {
        if let Some(suffix) = result.get(pos + 1..) {
            // 检查是否是纯数字（日期）
            if suffix.len() >= 6 && suffix.chars().all(|c| c.is_ascii_digit()) {
                result = result[..pos].to_string();
            }
        }
    }

    // 然后限制长度（最多保留前两个部分，用 - 分隔）
    let parts: Vec<&str> = result.split('-').collect();
    if parts.len() > 2 {
        format!("{}-{}", parts[0], parts[1])
    } else {
        result
    }
}

/// 极简 Spinner
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    label: String,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    /// 创建并启动 spinner（无标签）
    pub fn new() -> Self {
        Self::with_label("")
    }

    /// 创建并启动 spinner，带标签（如模型名称）
    ///
    /// # 示例
    /// ```ignore
    /// let spinner = Spinner::with_label("deepseek");
    /// // 显示: ⠋ deepseek
    /// ```
    pub fn with_label(label: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        let label = label.to_string();
        let label_clone = label.clone();

        let handle = thread::spawn(move || {
            let mut frame_idx = 0;

            // 不需要初始换行，让 spinner 紧接在用户输入后显示（极简主义）
            let _ = io::stdout().flush();

            while running_clone.load(Ordering::Relaxed) {
                // 清除当前行
                print!("\r");

                // 显示橙色 spinner
                print!("{}", SPINNER_FRAMES[frame_idx].truecolor(255, 165, 0));

                // 如果有标签，显示灰色标签
                if !label_clone.is_empty() {
                    print!(" {} ", label_clone.bold());
                }

                let _ = io::stdout().flush();

                // 下一帧
                frame_idx = (frame_idx + 1) % SPINNER_FRAMES.len();

                // 每 80ms 更新一次
                thread::sleep(Duration::from_millis(80));
            }

            // 结束时清除 spinner 行
            print!("\r");
            let _ = io::stdout().flush();
        });

        Self {
            running,
            handle: Some(handle),
            label,
        }
    }

    /// 停止 spinner
    pub fn stop(mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new();
        thread::sleep(Duration::from_millis(500));
        spinner.stop();
    }

    #[test]
    fn test_spinner_with_label() {
        let spinner = Spinner::with_label("deepseek");
        thread::sleep(Duration::from_millis(500));
        spinner.stop();
    }

    #[test]
    fn test_simplify_model_name() {
        // 基本简化
        assert_eq!(simplify_model_name("deepseek-chat"), "deepseek");
        assert_eq!(simplify_model_name("qwen2.5:latest"), "qwen2.5");
        assert_eq!(simplify_model_name("qwen2.5:stable"), "qwen2.5");

        // GPT 系列
        assert_eq!(simplify_model_name("gpt-4-turbo-preview"), "gpt-4");
        assert_eq!(simplify_model_name("gpt-3.5-turbo"), "gpt-3.5");

        // Claude 系列（带日期）
        assert_eq!(simplify_model_name("claude-3-opus-20240229"), "claude-3");
        assert_eq!(simplify_model_name("claude-3-sonnet-20240229"), "claude-3");

        // 已经很短的保持不变
        assert_eq!(simplify_model_name("qwen"), "qwen");
        assert_eq!(simplify_model_name("llama"), "llama");

        // 多段名称（保留前两段）
        assert_eq!(
            simplify_model_name("mixtral-8x7b-instruct-v0.1"),
            "mixtral-8x7b"
        );
    }
}
