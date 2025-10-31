//! Memory 数据迁移工具 (v1.16.0 Phase 3)
//!
//! 将旧的 Memory JSONL 数据迁移到 UnifiedTracer 系统
//!
//! # 使用场景
//!
//! - 从旧版 Memory 系统迁移到新的 UnifiedTracer
//! - 批量导入历史记忆数据
//! - 数据格式升级
//!
//! # 迁移流程
//!
//! 1. 读取 JSONL 文件（每行一个 MemoryEntry）
//! 2. 解析并验证每个条目
//! 3. 转换为 TraceEntry
//! 4. 添加到 UnifiedTracer
//! 5. 生成迁移报告

use super::manager::MemoryManager;
use super::memory_core::MemoryEntry;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// 迁移报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    /// 总条目数
    pub total: usize,

    /// 成功迁移数
    pub migrated: usize,

    /// 跳过数（格式错误等）
    pub skipped: usize,

    /// 失败数
    pub failed: usize,

    /// 错误详情
    pub errors: Vec<String>,
}

impl MigrationReport {
    /// 创建新的迁移报告
    pub fn new() -> Self {
        Self {
            total: 0,
            migrated: 0,
            skipped: 0,
            failed: 0,
            errors: Vec::new(),
        }
    }

    /// 记录成功迁移
    pub fn record_success(&mut self) {
        self.total += 1;
        self.migrated += 1;
    }

    /// 记录跳过
    pub fn record_skip(&mut self, reason: String) {
        self.total += 1;
        self.skipped += 1;
        self.errors.push(format!("Skipped: {}", reason));
    }

    /// 记录失败
    pub fn record_failure(&mut self, error: String) {
        self.total += 1;
        self.failed += 1;
        self.errors.push(format!("Failed: {}", error));
    }

    /// 判断是否成功
    pub fn is_success(&self) -> bool {
        self.failed == 0 && self.migrated > 0
    }

    /// 成功率
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.migrated as f64 / self.total as f64) * 100.0
        }
    }

    /// 格式化报告
    pub fn format(&self) -> String {
        let mut lines = vec![
            "━━━━━ Memory 数据迁移报告 ━━━━━".to_string(),
            format!("总条目数: {}", self.total),
            format!("✅ 成功迁移: {}", self.migrated),
        ];

        if self.skipped > 0 {
            lines.push(format!("⏭️  跳过: {}", self.skipped));
        }

        if self.failed > 0 {
            lines.push(format!("❌ 失败: {}", self.failed));
        }

        lines.push(format!("成功率: {:.1}%", self.success_rate()));

        if !self.errors.is_empty() {
            lines.push(String::new());
            lines.push("错误详情:".to_string());
            for (i, error) in self.errors.iter().take(10).enumerate() {
                lines.push(format!("  {}. {}", i + 1, error));
            }
            if self.errors.len() > 10 {
                lines.push(format!("  ... 还有 {} 个错误", self.errors.len() - 10));
            }
        }

        lines.push("━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
        lines.join("\n")
    }
}

impl Default for MigrationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory 数据迁移器
pub struct MemoryMigrator {
    /// MemoryManager 实例
    manager: MemoryManager,
}

impl MemoryMigrator {
    /// 创建新的迁移器
    ///
    /// # 参数
    /// - `manager`: MemoryManager 实例
    pub fn new(manager: MemoryManager) -> Self {
        Self { manager }
    }

    /// 从 JSONL 文件迁移数据
    ///
    /// # 参数
    /// - `file_path`: JSONL 文件路径
    ///
    /// # 返回
    /// - `Ok(MigrationReport)`: 迁移报告
    /// - `Err(...)`: 文件读取错误
    ///
    /// # 示例
    /// ```rust,no_run
    /// use realconsole::memory::migration::MemoryMigrator;
    /// use realconsole::memory::manager::MemoryManager;
    ///
    /// # async {
    /// let migrator = MemoryMigrator::new(manager);
    /// let report = migrator.migrate_from_file("memory.jsonl").await?;
    /// println!("{}", report.format());
    /// # Ok::<(), anyhow::Error>(())
    /// # };
    /// ```
    pub async fn migrate_from_file<P: AsRef<Path>>(&self, file_path: P) -> Result<MigrationReport> {
        let path = file_path.as_ref();
        let mut report = MigrationReport::new();

        // 检查文件是否存在
        if !path.exists() {
            return Err(anyhow::anyhow!("文件不存在: {}", path.display()));
        }

        // 打开文件
        let file = File::open(path)
            .with_context(|| format!("无法打开文件: {}", path.display()))?;

        let reader = BufReader::new(file);

        // 逐行读取
        for (line_num, line) in reader.lines().enumerate() {
            let line_number = line_num + 1;

            // 读取行
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    report.record_failure(format!("行 {}: 读取失败: {}", line_number, e));
                    continue;
                }
            };

            // 跳过空行
            if line.trim().is_empty() {
                report.record_skip(format!("行 {}: 空行", line_number));
                continue;
            }

            // 解析 JSON
            let entry: MemoryEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(e) => {
                    report.record_failure(format!("行 {}: JSON 解析失败: {}", line_number, e));
                    continue;
                }
            };

            // 迁移条目
            match self.migrate_entry(&entry).await {
                Ok(_) => report.record_success(),
                Err(e) => {
                    report.record_failure(format!(
                        "行 {}: 迁移失败: {}",
                        line_number, e
                    ));
                }
            }
        }

        Ok(report)
    }

    /// 迁移单个条目
    async fn migrate_entry(&self, entry: &MemoryEntry) -> Result<()> {
        self.manager
            .add_with_importance(
                entry.content.clone(),
                entry.entry_type,
                entry.importance,
            )
            .await;

        Ok(())
    }

    /// 批量迁移条目
    ///
    /// # 参数
    /// - `entries`: 要迁移的条目列表
    ///
    /// # 返回
    /// 迁移报告
    pub async fn migrate_entries(&self, entries: Vec<MemoryEntry>) -> MigrationReport {
        let mut report = MigrationReport::new();

        for (i, entry) in entries.iter().enumerate() {
            match self.migrate_entry(entry).await {
                Ok(_) => report.record_success(),
                Err(e) => {
                    report.record_failure(format!("条目 {}: {}", i + 1, e));
                }
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::ConversationConfig;
    use crate::conversation::context_manager::ContextManager;
    use crate::execution_logger::ExecutionLogger;
    use crate::history::HistoryManager;
    use crate::memory::memory_core::{EntryType, Importance};
    use crate::tracer::unified_tracer::UnifiedTracer;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use tokio::sync::RwLock;

    fn create_test_manager() -> MemoryManager {
        create_test_manager_with_capacity(100)
    }

    fn create_test_manager_with_capacity(capacity: usize) -> MemoryManager {
        let history = Arc::new(RwLock::new(HistoryManager::new(
            PathBuf::from("/tmp/test_migration_history.jsonl"),
            capacity,
        )));
        let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(capacity)));
        let context = Arc::new(RwLock::new(ContextManager::new(
            ConversationConfig::default(),
        )));

        let tracer = Arc::new(UnifiedTracer::with_custom_capacity(
            history,
            exec_logger,
            None,
            context,
            capacity,
        ));
        MemoryManager::new(tracer, capacity)
    }

    #[test]
    fn test_migration_report_new() {
        let report = MigrationReport::new();
        assert_eq!(report.total, 0);
        assert_eq!(report.migrated, 0);
        assert_eq!(report.skipped, 0);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn test_migration_report_record() {
        let mut report = MigrationReport::new();

        report.record_success();
        assert_eq!(report.total, 1);
        assert_eq!(report.migrated, 1);

        report.record_skip("test".to_string());
        assert_eq!(report.total, 2);
        assert_eq!(report.skipped, 1);

        report.record_failure("error".to_string());
        assert_eq!(report.total, 3);
        assert_eq!(report.failed, 1);
    }

    #[test]
    fn test_migration_report_success_rate() {
        let mut report = MigrationReport::new();
        report.record_success();
        report.record_success();
        report.record_skip("test".to_string());

        assert!((report.success_rate() - 66.67).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_migrate_from_file_not_exist() {
        let manager = create_test_manager();
        let migrator = MemoryMigrator::new(manager);

        let result = migrator.migrate_from_file("/tmp/nonexistent.jsonl").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_migrate_from_file_success() {
        let manager = create_test_manager();
        let migrator = MemoryMigrator::new(manager);

        // 创建临时测试文件
        let mut temp_file = NamedTempFile::new().unwrap();

        // 写入测试数据
        let entry1 = MemoryEntry::new("Test 1".to_string(), EntryType::User);
        let entry2 = MemoryEntry::new_with_importance(
            "Test 2".to_string(),
            EntryType::Assistant,
            Importance::Important,
        );

        writeln!(temp_file, "{}", serde_json::to_string(&entry1).unwrap()).unwrap();
        writeln!(temp_file, "{}", serde_json::to_string(&entry2).unwrap()).unwrap();
        writeln!(temp_file).unwrap(); // 空行
        writeln!(temp_file, "invalid json").unwrap(); // 无效 JSON

        temp_file.flush().unwrap();

        // 执行迁移
        let report = migrator.migrate_from_file(temp_file.path()).await.unwrap();

        // 验证报告
        assert_eq!(report.total, 4);
        assert_eq!(report.migrated, 2);
        assert_eq!(report.skipped, 1); // 空行
        assert_eq!(report.failed, 1); // 无效 JSON
    }

    #[tokio::test]
    async fn test_migrate_entries() {
        let manager = create_test_manager();
        let migrator = MemoryMigrator::new(manager);

        let entries = vec![
            MemoryEntry::new("Entry 1".to_string(), EntryType::User),
            MemoryEntry::new("Entry 2".to_string(), EntryType::Shell),
        ];

        let report = migrator.migrate_entries(entries).await;

        assert_eq!(report.total, 2);
        assert_eq!(report.migrated, 2);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn test_migration_report_format() {
        let mut report = MigrationReport::new();
        report.record_success();
        report.record_success();
        report.record_failure("test error".to_string());

        let formatted = report.format();
        assert!(formatted.contains("总条目数: 3"));
        assert!(formatted.contains("成功迁移: 2"));
        assert!(formatted.contains("失败: 1"));
        assert!(formatted.contains("test error"));
    }

    // ━━━━━ v1.16.5 Phase 4 Task B.3: 压力测试 ━━━━━

    #[tokio::test]
    async fn test_large_file_migration() {
        let manager = create_test_manager_with_capacity(12000);
        let migrator = MemoryMigrator::new(manager);

        let mut temp_file = NamedTempFile::new().unwrap();

        // 写入 10,000 条数据
        for i in 0..10_000 {
            let entry = MemoryEntry::new(
                format!("Test entry {} with some content for testing", i),
                EntryType::User,
            );
            writeln!(temp_file, "{}", serde_json::to_string(&entry).unwrap()).unwrap();
        }
        temp_file.flush().unwrap();

        // 执行迁移并测量时间
        let start = std::time::Instant::now();
        let report = migrator
            .migrate_from_file(temp_file.path())
            .await
            .unwrap();
        let duration = start.elapsed();

        assert_eq!(report.total, 10_000);
        assert_eq!(report.migrated, 10_000);
        assert_eq!(report.failed, 0);
        assert!(
            duration.as_secs() < 10,
            "迁移耗时过长: {:?}",
            duration
        );
    }

    #[tokio::test]
    async fn test_migration_error_recovery() {
        let manager = create_test_manager_with_capacity(200);
        let migrator = MemoryMigrator::new(manager);

        let mut temp_file = NamedTempFile::new().unwrap();

        // 写入 100 条数据，其中 10 条无效
        for i in 0..100 {
            if i % 10 == 0 {
                writeln!(temp_file, "{{invalid json}}").unwrap();
            } else {
                let entry = MemoryEntry::new(
                    format!("Entry {}", i),
                    EntryType::User,
                );
                writeln!(temp_file, "{}", serde_json::to_string(&entry).unwrap()).unwrap();
            }
        }
        temp_file.flush().unwrap();

        let report = migrator
            .migrate_from_file(temp_file.path())
            .await
            .unwrap();

        assert_eq!(report.total, 100);
        assert_eq!(report.migrated, 90);
        assert_eq!(report.failed, 10);
        assert!((report.success_rate() - 90.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_migration_performance_benchmark() {
        let manager = create_test_manager_with_capacity(5500);
        let migrator = MemoryMigrator::new(manager);

        let mut temp_file = NamedTempFile::new().unwrap();

        // 写入 5,000 条数据
        for i in 0..5_000 {
            let entry = MemoryEntry::new(
                format!("Entry {} - Lorem ipsum dolor sit amet", i),
                if i % 3 == 0 {
                    EntryType::Assistant
                } else {
                    EntryType::User
                },
            );
            writeln!(temp_file, "{}", serde_json::to_string(&entry).unwrap()).unwrap();
        }
        temp_file.flush().unwrap();

        // 性能基准测试
        let start = std::time::Instant::now();
        let report = migrator
            .migrate_from_file(temp_file.path())
            .await
            .unwrap();
        let duration = start.elapsed();

        assert_eq!(report.total, 5_000);
        assert_eq!(report.migrated, 5_000);

        // 计算吞吐量
        let throughput = report.migrated as f64 / duration.as_secs_f64();

        println!("迁移性能: {:.0} entries/sec", throughput);
        println!("总耗时: {:?}", duration);

        // 应该能够达到至少 500 entries/sec
        assert!(
            throughput > 500.0,
            "迁移吞吐量过低: {:.0} entries/sec",
            throughput
        );
    }
}
