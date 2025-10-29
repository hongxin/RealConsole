//! 离坎炼化炉状态栏
//!
//! 使用 crossterm 实现真正固定在终端底部的状态栏

use crossterm::{
    cursor::{self, MoveTo, RestorePosition, SavePosition},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 炼化炉状态
#[derive(Debug, Clone)]
pub struct FurnaceStatus {
    /// 上次循环时间
    pub last_cycle: Option<Instant>,

    /// 当前模式数量
    pub pattern_count: usize,

    /// 高置信度模式数量
    pub high_confidence_count: usize,

    /// 循环间隔（秒）
    pub cycle_interval_secs: u64,

    /// 是否启用状态栏
    pub enabled: bool,
}

impl Default for FurnaceStatus {
    fn default() -> Self {
        Self {
            last_cycle: None,
            pattern_count: 0,
            high_confidence_count: 0,
            cycle_interval_secs: 300,
            enabled: false, // 暂时禁用，等待更好的集成方案
        }
    }
}

/// 离坎炼化炉状态栏
///
/// 固定在终端最底部一行，使用 ANSI 转义码精确控制位置
pub struct LiKanStatusBar {
    /// 当前状态
    status: Arc<RwLock<FurnaceStatus>>,

    /// 上次渲染的内容（用于避免重复渲染）
    last_rendered: Arc<RwLock<String>>,
}

impl Default for LiKanStatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl LiKanStatusBar {
    /// 创建新的状态栏
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(FurnaceStatus::default())),
            last_rendered: Arc::new(RwLock::new(String::new())),
        }
    }

    /// 获取状态引用（用于后台更新）
    pub fn status(&self) -> Arc<RwLock<FurnaceStatus>> {
        Arc::clone(&self.status)
    }

    /// 渲染状态栏到终端底部
    ///
    /// 使用 crossterm 精确控制光标位置
    pub async fn render(&self) {
        let status = self.status.read().await;

        // 如果禁用，不渲染
        if !status.enabled {
            return;
        }

        let msg = self.format_message(&status);

        // 检查是否与上次内容相同（避免闪烁）
        {
            let last = self.last_rendered.read().await;
            if *last == msg {
                return;
            }
        }

        // 更新上次渲染内容
        {
            let mut last = self.last_rendered.write().await;
            *last = msg.clone();
        }

        // 渲染到终端底部
        if let Err(e) = self.render_to_bottom(&msg) {
            // 静默失败，不影响主程序
            eprintln!("状态栏渲染失败: {}", e);
        }
    }

    /// 渲染消息到终端底部
    fn render_to_bottom(&self, msg: &str) -> io::Result<()> {
        let mut stdout = io::stderr(); // 使用 stderr 避免干扰 stdout

        // 获取终端大小，如果失败则直接返回
        let (_, rows) = match terminal::size() {
            Ok(size) => size,
            Err(_) => return Ok(()), // 静默失败，不影响主程序
        };
        let bottom_row = rows.saturating_sub(1); // 最底部一行（从0开始）

        // 1. 保存当前光标位置
        execute!(stdout, SavePosition)?;

        // 2. 移动到底部行
        execute!(stdout, MoveTo(0, bottom_row))?;

        // 3. 清除该行
        execute!(stdout, Clear(ClearType::CurrentLine))?;

        // 4. 设置状态栏样式（深色背景，浅色前景）
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White)
        )?;

        // 5. 写入状态消息
        execute!(stdout, Print(msg))?;

        // 6. 重置颜色
        execute!(stdout, ResetColor)?;

        // 7. 恢复光标位置
        execute!(stdout, RestorePosition)?;

        // 8. 刷新输出
        stdout.flush()?;

        Ok(())
    }

    /// 格式化状态消息（极简）
    fn format_message(&self, status: &FurnaceStatus) -> String {
        match status.last_cycle {
            Some(last) => {
                let elapsed = last.elapsed().as_secs();
                let next_in = status.cycle_interval_secs.saturating_sub(elapsed);

                // 上次循环时间（人类可读）
                let ago = format_duration(elapsed);

                // 下次循环倒计时
                let next = if next_in == 0 {
                    "soon".to_string()
                } else {
                    format_duration(next_in)
                };

                // 模式数量（如果有高置信度，显示高亮）
                let patterns = if status.high_confidence_count > 0 {
                    format!("{} ({} ⭐)", status.pattern_count, status.high_confidence_count)
                } else {
                    format!("{}", status.pattern_count)
                };

                format!("🌊🔥 [{}] {} patterns | next: {}", ago, patterns, next)
            }
            None => {
                // 还未触发首次循环
                let next = format_duration(status.cycle_interval_secs);
                format!("🌊🔥 [waiting] 0 patterns | next: {}", next)
            }
        }
    }

    /// 清除状态栏
    pub fn clear(&self) -> io::Result<()> {
        let mut stdout = io::stderr();

        // 获取终端大小，如果失败则直接返回
        let (_, rows) = match terminal::size() {
            Ok(size) => size,
            Err(_) => return Ok(()), // 静默失败，避免在清理时出错
        };
        let bottom_row = rows.saturating_sub(1);

        // 使用 ? 操作符，但所有错误都会被外层捕获
        execute!(stdout, SavePosition)?;
        execute!(stdout, MoveTo(0, bottom_row))?;
        execute!(stdout, Clear(ClearType::CurrentLine))?;
        execute!(stdout, RestorePosition)?;
        stdout.flush()?;

        Ok(())
    }

    /// 更新状态并重新渲染
    pub async fn update(&self) {
        self.render().await;
    }

    /// 完成并清除状态栏
    pub async fn finish(&self) {
        // 禁用状态
        {
            let mut status = self.status.write().await;
            status.enabled = false;
        }

        // 清除显示
        let _ = self.clear();
    }
}

impl Drop for LiKanStatusBar {
    fn drop(&mut self) {
        // 尝试清除状态栏，但如果失败也不要 panic
        // 使用 catch_unwind 来防止在清理过程中发生 panic
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = self.clear();
        }));
    }
}

/// 格式化持续时间为人类可读格式
///
/// 极简风格：
/// - < 60s: "30s"
/// - < 1h: "15m"
/// - >= 1h: "2h"
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h", secs / 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m");
        assert_eq!(format_duration(300), "5m");
        assert_eq!(format_duration(3600), "1h");
        assert_eq!(format_duration(7200), "2h");
    }

    #[tokio::test]
    async fn test_statusbar_creation() {
        let statusbar = LiKanStatusBar::new();
        let status = statusbar.status();

        let s = status.read().await;
        assert_eq!(s.pattern_count, 0);
        assert!(!s.enabled); // 默认禁用，由配置控制
    }

    #[tokio::test]
    async fn test_status_update() {
        let statusbar = LiKanStatusBar::new();
        let status = statusbar.status();

        {
            let mut s = status.write().await;
            s.pattern_count = 10;
            s.high_confidence_count = 3;
            s.last_cycle = Some(Instant::now());
        }

        // 格式化测试
        let s = status.read().await;
        let msg = statusbar.format_message(&s);
        assert!(msg.contains("10"));
        assert!(msg.contains("3 ⭐"));
    }
}
