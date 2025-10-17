//! 极简主义 Spinner
//!
//! 在 LLM 计算时显示橙色旋转飞轮

use colored::Colorize;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Spinner 符号序列（旋转飞轮）
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// 极简 Spinner
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    /// 创建并启动 spinner
    pub fn new() -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let handle = thread::spawn(move || {
            let mut frame_idx = 0;

            // 打印初始换行，让 spinner 在新行显示
            print!("\n");
            let _ = io::stdout().flush();

            while running_clone.load(Ordering::Relaxed) {
                // 清除当前行
                print!("\r");

                // 显示橙色 spinner
                print!("{}", SPINNER_FRAMES[frame_idx].truecolor(255, 165, 0));
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
}
