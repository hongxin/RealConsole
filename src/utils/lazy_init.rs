//! v1.89.0: 延迟初始化工具
//!
//! 提供通用的延迟初始化支持，减少启动时间：
//! - **LazyInit<T>**: 通用延迟初始化包装器
//! - **StartupTimer**: 启动时间测量工具
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::utils::lazy_init::{LazyInit, StartupTimer};
//!
//! // 延迟初始化
//! let lazy_client = LazyInit::new(|| async {
//!     ExpensiveClient::connect().await
//! });
//!
//! // 首次使用时才初始化
//! let client = lazy_client.get().await?;
//!
//! // 启动计时
//! let timer = StartupTimer::start("Agent");
//! // ... 初始化代码 ...
//! timer.checkpoint("Memory loaded");
//! timer.checkpoint("Tools registered");
//! timer.finish(); // 打印总时间
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{OnceCell, RwLock};

// ============================================================================
// 延迟初始化错误
// ============================================================================

/// 延迟初始化错误
#[derive(Debug, Clone)]
pub struct LazyInitError {
    pub message: String,
}

impl std::fmt::Display for LazyInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lazy initialization failed: {}", self.message)
    }
}

impl std::error::Error for LazyInitError {}

// ============================================================================
// 工厂类型
// ============================================================================

/// 异步工厂函数类型
type AsyncFactory<T> = Box<
    dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<T, String>> + Send>> + Send + Sync,
>;

// ============================================================================
// 统计信息
// ============================================================================

/// 延迟初始化统计
#[derive(Debug, Default)]
pub struct LazyStats {
    /// 初始化尝试次数
    init_attempts: AtomicU64,
    /// 初始化成功次数
    init_success: AtomicU64,
    /// 初始化失败次数
    init_failures: AtomicU64,
    /// 访问次数（初始化前）
    pre_init_accesses: AtomicU64,
    /// 初始化耗时（微秒）
    init_duration_us: AtomicU64,
}

impl LazyStats {
    /// 获取统计快照
    pub fn snapshot(&self, initialized: bool) -> LazyStatsSnapshot {
        LazyStatsSnapshot {
            init_attempts: self.init_attempts.load(Ordering::Relaxed),
            init_success: self.init_success.load(Ordering::Relaxed),
            init_failures: self.init_failures.load(Ordering::Relaxed),
            pre_init_accesses: self.pre_init_accesses.load(Ordering::Relaxed),
            init_duration_us: self.init_duration_us.load(Ordering::Relaxed),
            is_initialized: initialized,
        }
    }
}

/// 统计快照
#[derive(Debug, Clone)]
pub struct LazyStatsSnapshot {
    pub init_attempts: u64,
    pub init_success: u64,
    pub init_failures: u64,
    pub pre_init_accesses: u64,
    pub init_duration_us: u64,
    pub is_initialized: bool,
}

impl LazyStatsSnapshot {
    /// 获取初始化耗时
    pub fn init_duration(&self) -> Duration {
        Duration::from_micros(self.init_duration_us)
    }
}

// ============================================================================
// LazyInit 实现
// ============================================================================

/// 通用延迟初始化包装器
///
/// 支持任意类型 T 的延迟初始化，首次访问时才执行初始化
pub struct LazyInit<T: Send + Sync> {
    /// 存储值（延迟初始化）
    value: OnceCell<Arc<T>>,
    /// 工厂函数
    factory: RwLock<Option<AsyncFactory<T>>>,
    /// 是否已初始化
    initialized: AtomicBool,
    /// 统计信息
    stats: Arc<LazyStats>,
    /// 组件名称（用于日志）
    name: String,
}

impl<T: Send + Sync + 'static> LazyInit<T> {
    /// 使用异步工厂函数创建
    pub fn new<F, Fut>(name: impl Into<String>, factory: F) -> Self
    where
        F: FnOnce() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, String>> + Send + 'static,
    {
        let boxed_factory: AsyncFactory<T> = Box::new(move || Box::pin(factory()));

        Self {
            value: OnceCell::new(),
            factory: RwLock::new(Some(boxed_factory)),
            initialized: AtomicBool::new(false),
            stats: Arc::new(LazyStats::default()),
            name: name.into(),
        }
    }

    /// 从已存在的值创建（已初始化状态）
    pub fn from_value(name: impl Into<String>, value: T) -> Self {
        let cell = OnceCell::new();
        let _ = cell.set(Arc::new(value));

        Self {
            value: cell,
            factory: RwLock::new(None),
            initialized: AtomicBool::new(true),
            stats: Arc::new(LazyStats::default()),
            name: name.into(),
        }
    }

    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    /// 获取组件名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取统计信息
    pub fn stats(&self) -> LazyStatsSnapshot {
        self.stats.snapshot(self.is_initialized())
    }

    /// 获取值（触发初始化）
    pub async fn get(&self) -> Result<Arc<T>, LazyInitError> {
        // 快速路径：已初始化
        if let Some(value) = self.value.get() {
            return Ok(Arc::clone(value));
        }

        self.stats.pre_init_accesses.fetch_add(1, Ordering::Relaxed);
        self.stats.init_attempts.fetch_add(1, Ordering::Relaxed);

        let start = Instant::now();

        // 获取工厂并初始化
        let mut factory_guard = self.factory.write().await;

        // 双重检查
        if let Some(value) = self.value.get() {
            return Ok(Arc::clone(value));
        }

        let result = if let Some(factory) = factory_guard.take() {
            factory().await
        } else {
            Err("Factory already consumed".to_string())
        };

        match result {
            Ok(value) => {
                let arc_value = Arc::new(value);
                let _ = self.value.set(Arc::clone(&arc_value));
                self.initialized.store(true, Ordering::SeqCst);

                let duration = start.elapsed();
                self.stats
                    .init_duration_us
                    .store(duration.as_micros() as u64, Ordering::Relaxed);
                self.stats.init_success.fetch_add(1, Ordering::Relaxed);

                Ok(arc_value)
            }
            Err(e) => {
                self.stats.init_failures.fetch_add(1, Ordering::Relaxed);
                Err(LazyInitError { message: e })
            }
        }
    }

    /// 尝试获取值（不触发初始化）
    pub fn try_get(&self) -> Option<Arc<T>> {
        self.value.get().cloned()
    }

    /// 强制初始化
    pub async fn initialize(&self) -> Result<(), LazyInitError> {
        self.get().await.map(|_| ())
    }
}

// ============================================================================
// StartupTimer 启动计时器
// ============================================================================

/// 启动阶段记录
#[derive(Debug, Clone)]
pub struct StartupCheckpoint {
    /// 阶段名称
    pub name: String,
    /// 从开始到此阶段的时间
    pub elapsed: Duration,
    /// 此阶段耗时
    pub duration: Duration,
}

/// 启动时间测量工具
///
/// 用于测量和分析启动过程中各阶段的耗时
pub struct StartupTimer {
    /// 组件名称
    name: String,
    /// 开始时间
    start: Instant,
    /// 上一个检查点时间
    last_checkpoint: Instant,
    /// 检查点列表
    checkpoints: Vec<StartupCheckpoint>,
    /// 是否启用详细输出
    verbose: bool,
}

impl StartupTimer {
    /// 开始计时
    pub fn start(name: impl Into<String>) -> Self {
        let now = Instant::now();
        Self {
            name: name.into(),
            start: now,
            last_checkpoint: now,
            checkpoints: Vec::new(),
            verbose: false,
        }
    }

    /// 启用详细输出
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// 记录检查点
    pub fn checkpoint(&mut self, name: impl Into<String>) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start);
        let duration = now.duration_since(self.last_checkpoint);

        let checkpoint = StartupCheckpoint {
            name: name.into(),
            elapsed,
            duration,
        };

        if self.verbose {
            eprintln!(
                "  [{:>6.2}ms] {} (+{:.2}ms)",
                elapsed.as_secs_f64() * 1000.0,
                checkpoint.name,
                duration.as_secs_f64() * 1000.0
            );
        }

        self.checkpoints.push(checkpoint);
        self.last_checkpoint = now;
    }

    /// 完成计时并返回总耗时
    pub fn finish(mut self) -> StartupReport {
        let total = self.start.elapsed();

        if self.verbose {
            eprintln!(
                "  [TOTAL] {} startup: {:.2}ms",
                self.name,
                total.as_secs_f64() * 1000.0
            );
        }

        StartupReport {
            name: self.name,
            total,
            checkpoints: std::mem::take(&mut self.checkpoints),
        }
    }

    /// 获取当前已过去的时间
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// 启动报告
#[derive(Debug, Clone)]
pub struct StartupReport {
    /// 组件名称
    pub name: String,
    /// 总耗时
    pub total: Duration,
    /// 检查点列表
    pub checkpoints: Vec<StartupCheckpoint>,
}

impl StartupReport {
    /// 获取总耗时（毫秒）
    pub fn total_ms(&self) -> f64 {
        self.total.as_secs_f64() * 1000.0
    }

    /// 找到最慢的阶段
    pub fn slowest_checkpoint(&self) -> Option<&StartupCheckpoint> {
        self.checkpoints.iter().max_by_key(|c| c.duration)
    }

    /// 生成报告摘要
    pub fn summary(&self) -> String {
        let mut s = format!("{} startup: {:.2}ms", self.name, self.total_ms());

        if let Some(slowest) = self.slowest_checkpoint() {
            s.push_str(&format!(
                " (slowest: {} at {:.2}ms)",
                slowest.name,
                slowest.duration.as_secs_f64() * 1000.0
            ));
        }

        s
    }
}

// ============================================================================
// 全局启动时间追踪
// ============================================================================

/// 全局启动报告收集器
#[derive(Debug, Default)]
pub struct StartupReports {
    reports: RwLock<Vec<StartupReport>>,
}

impl StartupReports {
    /// 创建新的收集器
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加报告
    pub async fn add(&self, report: StartupReport) {
        let mut reports = self.reports.write().await;
        reports.push(report);
    }

    /// 获取所有报告
    pub async fn all(&self) -> Vec<StartupReport> {
        self.reports.read().await.clone()
    }

    /// 获取总启动时间
    pub async fn total_time(&self) -> Duration {
        let reports = self.reports.read().await;
        reports.iter().map(|r| r.total).sum()
    }

    /// 生成汇总报告
    pub async fn summary(&self) -> String {
        let reports = self.reports.read().await;
        let total: Duration = reports.iter().map(|r| r.total).sum();

        let mut lines = vec![format!(
            "Startup Summary: {:.2}ms total",
            total.as_secs_f64() * 1000.0
        )];

        for report in reports.iter() {
            lines.push(format!("  - {}", report.summary()));
        }

        lines.join("\n")
    }
}

// ============================================================================
// 便捷宏
// ============================================================================

/// 创建延迟初始化值的便捷宏
#[macro_export]
macro_rules! lazy_init {
    ($name:expr, || async $body:block) => {
        $crate::utils::lazy_init::LazyInit::new($name, || async $body)
    };
    ($name:expr, $value:expr) => {
        $crate::utils::lazy_init::LazyInit::from_value($name, $value)
    };
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lazy_init_basic() {
        let lazy = LazyInit::new("test", || async { Ok::<_, String>(42) });

        assert!(!lazy.is_initialized());
        assert!(lazy.try_get().is_none());

        let value = lazy.get().await.unwrap();
        assert_eq!(*value, 42);
        assert!(lazy.is_initialized());
        assert!(lazy.try_get().is_some());
    }

    #[tokio::test]
    async fn test_lazy_init_from_value() {
        let lazy = LazyInit::from_value("test", "hello".to_string());

        assert!(lazy.is_initialized());
        let value = lazy.get().await.unwrap();
        assert_eq!(&*value, "hello");
    }

    #[tokio::test]
    async fn test_lazy_init_error() {
        let lazy = LazyInit::<i32>::new("test", || async { Err("init failed".to_string()) });

        let result = lazy.get().await;
        assert!(result.is_err());
        assert!(!lazy.is_initialized());

        let stats = lazy.stats();
        assert_eq!(stats.init_failures, 1);
    }

    #[tokio::test]
    async fn test_lazy_init_stats() {
        let lazy = LazyInit::new("test", || async { Ok::<_, String>(100) });

        // 初始化前
        let stats = lazy.stats();
        assert_eq!(stats.init_attempts, 0);
        assert!(!stats.is_initialized);

        // 触发初始化
        let _ = lazy.get().await;

        let stats = lazy.stats();
        assert_eq!(stats.init_attempts, 1);
        assert_eq!(stats.init_success, 1);
        assert!(stats.is_initialized);
        assert!(stats.init_duration_us > 0);
    }

    #[tokio::test]
    async fn test_lazy_init_concurrent() {
        use std::sync::atomic::AtomicUsize;

        let init_count = Arc::new(AtomicUsize::new(0));
        let init_count_clone = Arc::clone(&init_count);

        let lazy = Arc::new(LazyInit::new("test", move || {
            let count = Arc::clone(&init_count_clone);
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok::<_, String>(42)
            }
        }));

        // 并发访问
        let mut handles = vec![];
        for _ in 0..10 {
            let lazy = Arc::clone(&lazy);
            handles.push(tokio::spawn(async move { lazy.get().await }));
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(*result.unwrap(), 42);
        }

        // 应该只初始化一次
        assert_eq!(init_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_startup_timer() {
        let mut timer = StartupTimer::start("Test");

        std::thread::sleep(Duration::from_millis(5));
        timer.checkpoint("Phase 1");

        std::thread::sleep(Duration::from_millis(10));
        timer.checkpoint("Phase 2");

        let report = timer.finish();

        assert_eq!(report.name, "Test");
        assert!(report.total >= Duration::from_millis(15));
        assert_eq!(report.checkpoints.len(), 2);

        // Phase 2 应该是最慢的
        let slowest = report.slowest_checkpoint().unwrap();
        assert_eq!(slowest.name, "Phase 2");
    }

    #[tokio::test]
    async fn test_startup_reports() {
        let reports = StartupReports::new();

        let mut timer1 = StartupTimer::start("Component1");
        timer1.checkpoint("init");
        reports.add(timer1.finish()).await;

        let mut timer2 = StartupTimer::start("Component2");
        timer2.checkpoint("init");
        reports.add(timer2.finish()).await;

        let all = reports.all().await;
        assert_eq!(all.len(), 2);

        let summary = reports.summary().await;
        assert!(summary.contains("Component1"));
        assert!(summary.contains("Component2"));
    }

    #[test]
    fn test_startup_report_summary() {
        let report = StartupReport {
            name: "Test".to_string(),
            total: Duration::from_millis(100),
            checkpoints: vec![
                StartupCheckpoint {
                    name: "Fast".to_string(),
                    elapsed: Duration::from_millis(10),
                    duration: Duration::from_millis(10),
                },
                StartupCheckpoint {
                    name: "Slow".to_string(),
                    elapsed: Duration::from_millis(100),
                    duration: Duration::from_millis(90),
                },
            ],
        };

        let summary = report.summary();
        assert!(summary.contains("100.00ms"));
        assert!(summary.contains("Slow"));
    }
}
