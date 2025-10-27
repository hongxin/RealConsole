//! 离坎炼化炉 - 循环核心
//!
//! ## 哲学
//!
//! 炼丹炉是阴阳转换的枢纽：
//! - 坎（☵）：收集原料，提取精华
//! - 离（☲）：炼化转换，输出成丹
//! - 循环：自主触发，永续动力
//!
//! ## 实现
//!
//! 极简循环，三步完成：
//! 1. 坎阶段：提取模式
//! 2. 离阶段：更新增强器
//! 3. 反馈：记录报告

use super::kan::KanExtractor;
use super::li::LiEnhancer;
use super::types::{CycleReport, FurnaceConfig};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// 离坎炼化炉
///
/// 系统自主学习的心脏
pub struct LiKanFurnace {
    /// 坎：模式提取器
    kan: KanExtractor,

    /// 离：建议增强器
    li: Arc<RwLock<LiEnhancer>>,

    /// 配置
    config: FurnaceConfig,

    /// 上次循环时间
    last_cycle_time: Option<Instant>,

    /// 循环历史（最多保留最近10次）
    cycle_history: Vec<CycleReport>,
}

impl LiKanFurnace {
    /// 创建新的炼化炉
    pub fn new(config: FurnaceConfig) -> Self {
        Self {
            kan: KanExtractor::new(config.clone()),
            li: Arc::new(RwLock::new(LiEnhancer::new())),
            config,
            last_cycle_time: None,
            cycle_history: Vec::new(),
        }
    }

    /// 获取离增强器的引用
    ///
    /// 用于 SuggestionEngine 集成
    pub fn li_enhancer(&self) -> Arc<RwLock<LiEnhancer>> {
        Arc::clone(&self.li)
    }

    /// 获取炼化炉配置
    pub fn config(&self) -> &FurnaceConfig {
        &self.config
    }

    /// 检查是否应该触发循环
    ///
    /// 简化版：只检查时间间隔
    pub fn should_cycle(&self) -> bool {
        match self.last_cycle_time {
            None => true, // 第一次循环
            Some(last) => {
                let elapsed = last.elapsed().as_secs();
                elapsed >= self.config.cycle_interval_secs
            }
        }
    }

    /// 执行一次完整的炼化循环
    ///
    /// 坎（提取）→ 离（更新）→ 反馈（记录）
    pub async fn cycle_once(
        &mut self,
        trace_entries: &[crate::tracer::entry::TraceEntry],
        suggestion_stats: &std::collections::HashMap<
            String,
            crate::suggestion::feedback::SuggestionStats, // 使用公开的 re-export
        >,
    ) -> Result<CycleReport> {
        let started_at = Utc::now();

        // 1. 坎阶段：提取模式
        let patterns = self.kan.extract_patterns(trace_entries, suggestion_stats);

        // 2. 离阶段：更新增强器
        {
            let mut li = self.li.write().await;
            li.update_patterns(patterns.clone());
        }

        // 3. 反馈：生成报告
        let report = CycleReport::new(&patterns, started_at);

        // 4. 更新状态
        self.last_cycle_time = Some(Instant::now());
        self.cycle_history.push(report.clone());

        // 保持历史数量（最多10次）
        if self.cycle_history.len() > 10 {
            self.cycle_history.remove(0);
        }

        Ok(report)
    }

    /// 获取循环历史
    pub fn cycle_history(&self) -> &[CycleReport] {
        &self.cycle_history
    }

    /// 获取最后一次循环报告
    pub fn last_cycle_report(&self) -> Option<&CycleReport> {
        self.cycle_history.last()
    }

    /// 获取距离上次循环的时间（秒）
    pub fn time_since_last_cycle(&self) -> Option<u64> {
        self.last_cycle_time
            .map(|t| t.elapsed().as_secs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracer::entry::TraceEntry;
    use crate::tracer::types::{Dimension, EntryType, Status};
    use std::collections::HashMap;

    fn create_test_entry(command: &str, status: Status) -> TraceEntry {
        use uuid::Uuid;
        TraceEntry {
            id: Uuid::new_v4(),
            dimension: Dimension::Statistics,
            entry_type: EntryType::ShellCommand,
            timestamp: Utc::now(),
            content: command.to_string(), // 使用 content 字段
            status,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_furnace_cycle_once() {
        let mut furnace = LiKanFurnace::new(FurnaceConfig::default());

        let entries = vec![
            create_test_entry("cargo build", Status::Success),
            create_test_entry("cargo build", Status::Success),
            create_test_entry("cargo build", Status::Success),
            create_test_entry("cargo test", Status::Success),
        ];

        let stats = HashMap::new();

        // 执行一次循环
        let report = furnace.cycle_once(&entries, &stats).await.unwrap();

        // 应该发现一些模式
        assert!(report.patterns_found > 0);

        // 历史应该记录
        assert_eq!(furnace.cycle_history().len(), 1);
    }

    #[test]
    fn test_should_cycle() {
        let mut furnace = LiKanFurnace::new(FurnaceConfig {
            cycle_interval_secs: 1, // 1秒间隔
            ..Default::default()
        });

        // 第一次应该触发
        assert!(furnace.should_cycle());

        // 设置刚刚循环过
        furnace.last_cycle_time = Some(Instant::now());

        // 立即检查，不应该触发
        assert!(!furnace.should_cycle());

        // 等待1秒后应该触发
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert!(furnace.should_cycle());
    }

    #[test]
    fn test_time_since_last_cycle() {
        let mut furnace = LiKanFurnace::new(FurnaceConfig::default());

        // 初始时无上次循环
        assert!(furnace.time_since_last_cycle().is_none());

        // 设置循环时间
        furnace.last_cycle_time = Some(Instant::now());
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 应该有时间差
        let elapsed = furnace.time_since_last_cycle().unwrap();
        assert!(elapsed >= 0);
    }

    #[tokio::test]
    async fn test_cycle_history_limit() {
        let mut furnace = LiKanFurnace::new(FurnaceConfig::default());
        let entries = vec![create_test_entry("test", Status::Success)];
        let stats = HashMap::new();

        // 执行11次循环
        for _ in 0..11 {
            furnace.cycle_once(&entries, &stats).await.unwrap();
        }

        // 应该只保留10次
        assert_eq!(furnace.cycle_history().len(), 10);
    }
}
