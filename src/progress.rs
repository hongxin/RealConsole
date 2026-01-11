//! v1.86.0: 统一进度指示器模块
//!
//! 提供多种进度显示方式：
//! - **Spinner**: 不确定进度的旋转指示器
//! - **ProgressBar**: 确定进度的进度条
//! - **TimedSpinner**: 带计时的旋转指示器
//! - **MultiProgress**: 多任务并行进度
//!
//! # 设计原则
//!
//! 1. **用户反馈**: 长时间操作必须有视觉反馈
//! 2. **信息丰富**: 显示进度、已用时间、预计剩余时间
//! 3. **易于集成**: 简单的 API，支持 async/await

use colored::Colorize;
use indicatif::{ProgressBar as IndicatifBar, ProgressStyle, MultiProgress as IndicatifMulti};
use std::time::{Duration, Instant};

/// 进度指示器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressType {
    /// 不确定进度（旋转指示器）
    Indeterminate,
    /// 确定进度（进度条）
    Determinate,
    /// 带计时的不确定进度
    TimedIndeterminate,
}

/// 进度指示器配置
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    /// 进度类型
    pub progress_type: ProgressType,
    /// 显示消息
    pub message: String,
    /// 总步数（仅用于确定进度）
    pub total: u64,
    /// 刷新间隔（毫秒）
    pub tick_interval_ms: u64,
    /// 是否显示已用时间
    pub show_elapsed: bool,
    /// 是否显示预计剩余时间
    pub show_eta: bool,
    /// 进度条宽度
    pub bar_width: usize,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            progress_type: ProgressType::Indeterminate,
            message: String::new(),
            total: 100,
            tick_interval_ms: 100,
            show_elapsed: true,
            show_eta: true,
            bar_width: 40,
        }
    }
}

impl ProgressConfig {
    /// 创建不确定进度配置
    pub fn spinner(message: impl Into<String>) -> Self {
        Self {
            progress_type: ProgressType::Indeterminate,
            message: message.into(),
            ..Default::default()
        }
    }

    /// 创建确定进度配置
    pub fn progress_bar(message: impl Into<String>, total: u64) -> Self {
        Self {
            progress_type: ProgressType::Determinate,
            message: message.into(),
            total,
            ..Default::default()
        }
    }

    /// 创建带计时的旋转指示器配置
    pub fn timed_spinner(message: impl Into<String>) -> Self {
        Self {
            progress_type: ProgressType::TimedIndeterminate,
            message: message.into(),
            ..Default::default()
        }
    }
}

/// 统一进度指示器
///
/// 封装 indicatif 提供统一的进度显示接口
pub struct ProgressIndicator {
    bar: IndicatifBar,
    start_time: Instant,
    config: ProgressConfig,
}

impl ProgressIndicator {
    /// 创建新的进度指示器
    pub fn new(config: ProgressConfig) -> Self {
        let bar = match config.progress_type {
            ProgressType::Indeterminate => {
                let bar = IndicatifBar::new_spinner();
                bar.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.cyan} {msg}")
                        .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                );
                bar
            }
            ProgressType::TimedIndeterminate => {
                let bar = IndicatifBar::new_spinner();
                bar.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.cyan} {msg} {elapsed_precise}")
                        .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                );
                bar
            }
            ProgressType::Determinate => {
                let bar = IndicatifBar::new(config.total);
                let template = if config.show_elapsed && config.show_eta {
                    "{msg}\n{bar:40.cyan/dim} {pos}/{len} ({percent}%) [{elapsed_precise} / {eta_precise}]"
                } else if config.show_elapsed {
                    "{msg}\n{bar:40.cyan/dim} {pos}/{len} ({percent}%) [{elapsed_precise}]"
                } else {
                    "{msg}\n{bar:40.cyan/dim} {pos}/{len} ({percent}%)"
                };
                bar.set_style(
                    ProgressStyle::default_bar()
                        .template(template)
                        .unwrap_or_else(|_| ProgressStyle::default_bar())
                        .progress_chars("█▓▒░"),
                );
                bar
            }
        };

        bar.set_message(config.message.clone());
        bar.enable_steady_tick(Duration::from_millis(config.tick_interval_ms));

        Self {
            bar,
            start_time: Instant::now(),
            config,
        }
    }

    /// 创建简单的旋转指示器
    pub fn spinner(message: impl Into<String>) -> Self {
        Self::new(ProgressConfig::spinner(message))
    }

    /// 创建带计时的旋转指示器
    pub fn timed_spinner(message: impl Into<String>) -> Self {
        Self::new(ProgressConfig::timed_spinner(message))
    }

    /// 创建进度条
    pub fn progress_bar(message: impl Into<String>, total: u64) -> Self {
        Self::new(ProgressConfig::progress_bar(message, total))
    }

    /// 更新消息
    pub fn set_message(&self, message: impl Into<String>) {
        self.bar.set_message(message.into());
    }

    /// 更新进度（仅用于确定进度）
    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    /// 增加进度
    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    /// 获取已用时间
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 获取已用时间的格式化字符串
    pub fn elapsed_string(&self) -> String {
        format_duration(self.elapsed())
    }

    /// 完成并显示成功消息
    pub fn finish_with_message(&self, message: impl Into<String>) {
        self.bar.finish_with_message(message.into());
    }

    /// 完成并清除
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }

    /// 以成功状态完成
    pub fn succeed(&self, message: impl Into<String>) {
        let elapsed = self.elapsed_string();
        let msg = format!(
            "{} {} {}",
            "✓".green().bold(),
            message.into(),
            format!("({})", elapsed).dimmed()
        );
        self.bar.finish_with_message(msg);
    }

    /// 以失败状态完成
    pub fn fail(&self, message: impl Into<String>) {
        let elapsed = self.elapsed_string();
        let msg = format!(
            "{} {} {}",
            "✗".red().bold(),
            message.into(),
            format!("({})", elapsed).dimmed()
        );
        self.bar.finish_with_message(msg);
    }

    /// 以警告状态完成
    pub fn warn(&self, message: impl Into<String>) {
        let elapsed = self.elapsed_string();
        let msg = format!(
            "{} {} {}",
            "⚠".yellow().bold(),
            message.into(),
            format!("({})", elapsed).dimmed()
        );
        self.bar.finish_with_message(msg);
    }
}

/// 多任务进度管理器
///
/// 用于同时显示多个进度指示器
pub struct MultiProgressManager {
    multi: IndicatifMulti,
}

impl MultiProgressManager {
    /// 创建新的多任务进度管理器
    pub fn new() -> Self {
        Self {
            multi: IndicatifMulti::new(),
        }
    }

    /// 添加进度条
    pub fn add_progress(&self, config: ProgressConfig) -> ProgressIndicator {
        let indicator = ProgressIndicator::new(config);
        self.multi.add(indicator.bar.clone());
        indicator
    }

    /// 添加旋转指示器
    pub fn add_spinner(&self, message: impl Into<String>) -> ProgressIndicator {
        self.add_progress(ProgressConfig::spinner(message))
    }

    /// 添加进度条
    pub fn add_bar(&self, message: impl Into<String>, total: u64) -> ProgressIndicator {
        self.add_progress(ProgressConfig::progress_bar(message, total))
    }
}

impl Default for MultiProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 格式化持续时间
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs == 0 {
        format!("{}ms", millis)
    } else if secs < 60 {
        format!("{}.{}s", secs, millis / 100)
    } else if secs < 3600 {
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        format!("{}m{}s", mins, remaining_secs)
    } else {
        let hours = secs / 3600;
        let remaining_mins = (secs % 3600) / 60;
        format!("{}h{}m", hours, remaining_mins)
    }
}

/// 带进度的任务执行器
///
/// 简化在异步上下文中使用进度指示器
pub struct ProgressTask<T> {
    indicator: ProgressIndicator,
    result: Option<T>,
}

impl<T> ProgressTask<T> {
    /// 创建新的进度任务
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            indicator: ProgressIndicator::timed_spinner(message),
            result: None,
        }
    }

    /// 设置结果并完成
    pub fn complete(mut self, result: T, message: impl Into<String>) -> T {
        self.indicator.succeed(message);
        self.result = Some(result);
        self.result.take().unwrap()
    }

    /// 设置失败并完成
    pub fn fail_with(self, message: impl Into<String>) {
        self.indicator.fail(message);
    }

    /// 获取指示器引用（用于更新消息）
    pub fn indicator(&self) -> &ProgressIndicator {
        &self.indicator
    }
}

/// 运行带进度指示的同步任务
///
/// # 示例
/// ```ignore
/// let result = with_progress("处理文件...", || {
///     // 长时间操作
///     std::thread::sleep(std::time::Duration::from_secs(2));
///     "完成"
/// });
/// ```
pub fn with_progress<T, F>(message: impl Into<String>, f: F) -> T
where
    F: FnOnce() -> T,
{
    let msg = message.into();
    let indicator = ProgressIndicator::timed_spinner(&msg);
    let result = f();
    indicator.succeed(&msg);
    result
}

/// 运行带进度条的迭代任务
///
/// # 示例
/// ```ignore
/// let items = vec![1, 2, 3, 4, 5];
/// with_progress_iter("处理项目...", items.iter(), |item| {
///     // 处理每个项目
///     std::thread::sleep(std::time::Duration::from_millis(100));
/// });
/// ```
pub fn with_progress_iter<T, I, F>(message: impl Into<String>, iter: I, mut f: F)
where
    I: ExactSizeIterator<Item = T>,
    F: FnMut(T),
{
    let total = iter.len() as u64;
    let indicator = ProgressIndicator::progress_bar(message, total);

    for item in iter {
        f(item);
        indicator.inc(1);
    }

    indicator.finish_and_clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_format_duration_millis() {
        let d = Duration::from_millis(500);
        assert_eq!(format_duration(d), "500ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        let d = Duration::from_secs(5) + Duration::from_millis(300);
        assert_eq!(format_duration(d), "5.3s");
    }

    #[test]
    fn test_format_duration_minutes() {
        let d = Duration::from_secs(125);
        assert_eq!(format_duration(d), "2m5s");
    }

    #[test]
    fn test_format_duration_hours() {
        let d = Duration::from_secs(3725);
        assert_eq!(format_duration(d), "1h2m");
    }

    #[test]
    fn test_progress_config_spinner() {
        let config = ProgressConfig::spinner("测试");
        assert_eq!(config.progress_type, ProgressType::Indeterminate);
        assert_eq!(config.message, "测试");
    }

    #[test]
    fn test_progress_config_bar() {
        let config = ProgressConfig::progress_bar("下载", 100);
        assert_eq!(config.progress_type, ProgressType::Determinate);
        assert_eq!(config.total, 100);
    }

    #[test]
    fn test_progress_config_timed() {
        let config = ProgressConfig::timed_spinner("处理中");
        assert_eq!(config.progress_type, ProgressType::TimedIndeterminate);
    }

    #[test]
    fn test_spinner_creation() {
        let indicator = ProgressIndicator::spinner("测试旋转");
        thread::sleep(Duration::from_millis(200));
        indicator.finish_and_clear();
    }

    #[test]
    fn test_timed_spinner() {
        let indicator = ProgressIndicator::timed_spinner("带计时");
        thread::sleep(Duration::from_millis(200));
        let elapsed = indicator.elapsed();
        assert!(elapsed >= Duration::from_millis(200));
        indicator.finish_and_clear();
    }

    #[test]
    fn test_progress_bar() {
        let indicator = ProgressIndicator::progress_bar("处理", 10);
        for i in 0..10 {
            indicator.set_position(i + 1);
            thread::sleep(Duration::from_millis(10));
        }
        indicator.finish_and_clear();
    }

    #[test]
    fn test_succeed_status() {
        let indicator = ProgressIndicator::timed_spinner("测试");
        thread::sleep(Duration::from_millis(100));
        indicator.succeed("完成");
    }

    #[test]
    fn test_fail_status() {
        let indicator = ProgressIndicator::timed_spinner("测试");
        thread::sleep(Duration::from_millis(100));
        indicator.fail("失败");
    }

    #[test]
    fn test_warn_status() {
        let indicator = ProgressIndicator::timed_spinner("测试");
        thread::sleep(Duration::from_millis(100));
        indicator.warn("警告");
    }

    #[test]
    fn test_elapsed_string() {
        let indicator = ProgressIndicator::spinner("测试");
        thread::sleep(Duration::from_millis(100));
        let elapsed = indicator.elapsed_string();
        assert!(!elapsed.is_empty());
        indicator.finish_and_clear();
    }

    #[test]
    fn test_with_progress() {
        let result = with_progress("测试任务", || {
            thread::sleep(Duration::from_millis(100));
            42
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_with_progress_iter() {
        let items = vec![1, 2, 3, 4, 5];
        let mut sum = 0;
        with_progress_iter("处理数字", items.into_iter(), |n| {
            sum += n;
            thread::sleep(Duration::from_millis(10));
        });
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_multi_progress_manager() {
        let manager = MultiProgressManager::new();
        let spinner = manager.add_spinner("任务 1");
        let bar = manager.add_bar("任务 2", 5);

        thread::sleep(Duration::from_millis(50));

        for i in 1..=5 {
            bar.set_position(i);
            thread::sleep(Duration::from_millis(20));
        }

        spinner.finish_and_clear();
        bar.finish_and_clear();
    }
}
