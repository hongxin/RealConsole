//! 命令历史记录管理
//!
//! 功能：
//! - 记录用户输入的命令历史
//! - 持久化到 JSON 文件
//! - 支持搜索和智能排序
//! - 记录执行次数和时间戳
//!
//! 设计理念（一分为三）：
//! - 存储层：JSON 文件持久化
//! - 逻辑层：历史记录管理（搜索、排序）
//! - 展示层：命令和 REPL 集成

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 历史记录项
///
/// 记录单条命令的执行信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    /// 命令内容
    pub command: String,

    /// 首次执行时间
    pub first_timestamp: DateTime<Utc>,

    /// 最后执行时间
    pub last_timestamp: DateTime<Utc>,

    /// 执行次数
    pub count: u32,

    /// 最后执行是否成功
    #[serde(default)]
    pub last_success: bool,
}

impl HistoryEntry {
    /// 创建新的历史记录项
    pub fn new(command: String) -> Self {
        let now = Utc::now();
        Self {
            command,
            first_timestamp: now,
            last_timestamp: now,
            count: 1,
            last_success: true,
        }
    }

    /// 更新执行信息（增加计数，更新时间）
    pub fn update(&mut self, success: bool) {
        self.count += 1;
        self.last_timestamp = Utc::now();
        self.last_success = success;
    }

    /// 计算综合得分（用于排序）
    ///
    /// 得分算法：
    /// - 执行次数权重：70%
    /// - 时间新鲜度权重：30%
    ///
    /// 时间新鲜度：最近 1 天 = 1.0，随时间衰减
    pub fn score(&self) -> f64 {
        // 频率得分（归一化到 0-1）
        let frequency_score = (self.count as f64).ln() / 10.0;

        // 时间新鲜度得分
        let age_seconds = (Utc::now() - self.last_timestamp).num_seconds() as f64;
        let age_days = age_seconds / 86400.0;
        let recency_score = (-age_days / 7.0).exp(); // 7 天半衰期

        // 综合得分
        frequency_score * 0.7 + recency_score * 0.3
    }
}

/// 排序策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortStrategy {
    /// 按时间排序（最新优先）
    Time,
    /// 按频率排序（最常用优先）
    Frequency,
    /// 智能排序（综合时间和频率）
    Smart,
}

/// 历史记录管理器
///
/// 管理所有历史记录的增删改查和持久化
pub struct HistoryManager {
    /// 历史记录列表
    entries: Vec<HistoryEntry>,

    /// 命令到索引的映射（用于快速查找）
    command_index: HashMap<String, usize>,

    /// 持久化文件路径
    file_path: PathBuf,

    /// 最大历史记录数量
    max_entries: usize,

    /// 是否自动持久化
    auto_save: bool,
}

impl HistoryManager {
    /// 创建新的历史记录管理器
    ///
    /// # 参数
    /// - `file_path`: 持久化文件路径
    /// - `max_entries`: 最大历史记录数量（默认 1000）
    ///
    /// # 返回
    /// - 初始化的 HistoryManager
    pub fn new(file_path: impl Into<PathBuf>, max_entries: usize) -> Self {
        let file_path = file_path.into();
        let mut manager = Self {
            entries: Vec::new(),
            command_index: HashMap::new(),
            file_path,
            max_entries,
            auto_save: true,
        };

        // 尝试从文件加载历史记录
        if let Err(e) = manager.load() {
            eprintln!("警告: 加载历史记录失败: {}", e);
        }

        manager
    }

    /// 默认配置（使用 ~/.realconsole/history.json）
    pub fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home.join(".realconsole");
        let history_file = config_dir.join("history.json");

        // 确保目录存在
        if let Err(e) = fs::create_dir_all(&config_dir) {
            eprintln!("警告: 创建配置目录失败: {}", e);
        }

        Self::new(history_file, 1000)
    }

    /// 添加命令到历史记录
    ///
    /// 如果命令已存在，则更新执行次数和时间
    ///
    /// # 参数
    /// - `command`: 命令内容
    /// - `success`: 是否执行成功
    pub fn add(&mut self, command: impl Into<String>, success: bool) {
        let command = command.into();

        // 忽略空命令和系统命令
        if command.trim().is_empty() || command.starts_with('/') {
            return;
        }

        // 检查是否已存在
        if let Some(&index) = self.command_index.get(&command) {
            // 更新现有记录
            if let Some(entry) = self.entries.get_mut(index) {
                entry.update(success);
            }
        } else {
            // 添加新记录
            let entry = HistoryEntry::new(command.clone());
            self.entries.push(entry);
            let index = self.entries.len() - 1;
            self.command_index.insert(command, index);

            // 检查是否超过最大数量
            if self.entries.len() > self.max_entries {
                self.prune();
            }
        }

        // 自动持久化
        if self.auto_save {
            if let Err(e) = self.save() {
                eprintln!("警告: 保存历史记录失败: {}", e);
            }
        }
    }

    /// 搜索历史记录
    ///
    /// # 参数
    /// - `keyword`: 搜索关键词
    /// - `strategy`: 排序策略
    ///
    /// # 返回
    /// - 匹配的历史记录列表（已排序）
    pub fn search(&self, keyword: &str, strategy: SortStrategy) -> Vec<HistoryEntry> {
        let keyword_lower = keyword.to_lowercase();

        // 过滤匹配的记录
        let mut results: Vec<HistoryEntry> = self
            .entries
            .iter()
            .filter(|entry| entry.command.to_lowercase().contains(&keyword_lower))
            .cloned()
            .collect();

        // 排序
        self.sort_entries(&mut results, strategy);

        results
    }

    /// 获取最近的历史记录
    ///
    /// # 参数
    /// - `limit`: 返回的最大数量
    /// - `strategy`: 排序策略
    ///
    /// # 返回
    /// - 历史记录列表（已排序）
    pub fn recent(&self, limit: usize, strategy: SortStrategy) -> Vec<HistoryEntry> {
        let mut results = self.entries.clone();
        self.sort_entries(&mut results, strategy);
        results.into_iter().take(limit).collect()
    }

    /// 获取所有历史记录
    ///
    /// # 参数
    /// - `strategy`: 排序策略
    ///
    /// # 返回
    /// - 所有历史记录（已排序）
    pub fn all(&self, strategy: SortStrategy) -> Vec<HistoryEntry> {
        let mut results = self.entries.clone();
        self.sort_entries(&mut results, strategy);
        results
    }

    /// 清空历史记录
    pub fn clear(&mut self) -> Result<(), std::io::Error> {
        self.entries.clear();
        self.command_index.clear();
        self.save()
    }

    /// 删除指定命令
    pub fn delete(&mut self, command: &str) -> bool {
        if let Some(&index) = self.command_index.get(command) {
            self.entries.remove(index);
            self.rebuild_index();

            if self.auto_save {
                let _ = self.save();
            }

            true
        } else {
            false
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> HistoryStats {
        HistoryStats {
            total_entries: self.entries.len(),
            total_executions: self.entries.iter().map(|e| e.count as u64).sum(),
            unique_commands: self.command_index.len(),
        }
    }

    /// 保存历史记录到文件
    pub fn save(&self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        fs::write(&self.file_path, json)?;
        Ok(())
    }

    /// 从文件加载历史记录
    pub fn load(&mut self) -> Result<(), std::io::Error> {
        if !self.file_path.exists() {
            return Ok(()); // 文件不存在，返回空历史
        }

        let content = fs::read_to_string(&self.file_path)?;
        self.entries = serde_json::from_str(&content)?;
        self.rebuild_index();

        Ok(())
    }

    /// 排序历史记录
    fn sort_entries(&self, entries: &mut [HistoryEntry], strategy: SortStrategy) {
        match strategy {
            SortStrategy::Time => {
                entries.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));
            }
            SortStrategy::Frequency => {
                entries.sort_by(|a, b| b.count.cmp(&a.count));
            }
            SortStrategy::Smart => {
                entries.sort_by(|a, b| {
                    b.score()
                        .partial_cmp(&a.score())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
    }

    /// 修剪历史记录（删除得分最低的记录）
    fn prune(&mut self) {
        // 按智能得分排序
        self.entries
            .sort_by(|a, b| b.score().partial_cmp(&a.score()).unwrap());

        // 保留前 max_entries 个
        self.entries.truncate(self.max_entries);

        // 重建索引
        self.rebuild_index();
    }

    /// 重建命令索引
    fn rebuild_index(&mut self) {
        self.command_index.clear();
        for (index, entry) in self.entries.iter().enumerate() {
            self.command_index.insert(entry.command.clone(), index);
        }
    }
}

/// 历史记录统计信息
#[derive(Debug, Clone)]
pub struct HistoryStats {
    /// 总记录数
    pub total_entries: usize,
    /// 总执行次数
    pub total_executions: u64,
    /// 唯一命令数
    pub unique_commands: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_history_entry_creation() {
        let entry = HistoryEntry::new("ls -la".to_string());
        assert_eq!(entry.command, "ls -la");
        assert_eq!(entry.count, 1);
        assert!(entry.last_success);
    }

    #[test]
    fn test_history_entry_update() {
        let mut entry = HistoryEntry::new("echo hello".to_string());
        let original_timestamp = entry.last_timestamp;

        thread::sleep(Duration::from_millis(10));
        entry.update(true);

        assert_eq!(entry.count, 2);
        assert!(entry.last_timestamp > original_timestamp);
    }

    #[test]
    fn test_history_entry_score() {
        let entry = HistoryEntry::new("test".to_string());
        let score = entry.score();
        assert!(score > 0.0);
        assert!(score < 1.0);
    }

    #[test]
    fn test_history_manager_add() {
        let temp_file = std::env::temp_dir().join("test_history.json");
        let mut manager = HistoryManager::new(&temp_file, 100);

        manager.add("ls", true);
        assert_eq!(manager.entries.len(), 1);

        manager.add("ls", true);
        assert_eq!(manager.entries.len(), 1); // Same command, should update
        assert_eq!(manager.entries[0].count, 2);

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_history_manager_search() {
        let temp_file = std::env::temp_dir().join("test_history_search.json");
        let mut manager = HistoryManager::new(&temp_file, 100);

        manager.add("git status", true);
        manager.add("git log", true);
        manager.add("ls -la", true);

        let results = manager.search("git", SortStrategy::Time);
        assert_eq!(results.len(), 2);

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_history_manager_recent() {
        let temp_file = std::env::temp_dir().join("test_history_recent.json");
        let mut manager = HistoryManager::new(&temp_file, 100);

        for i in 0..10 {
            manager.add(format!("command{}", i), true);
        }

        let recent = manager.recent(5, SortStrategy::Time);
        assert_eq!(recent.len(), 5);

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_history_manager_delete() {
        let temp_file = std::env::temp_dir().join("test_history_delete.json");
        let mut manager = HistoryManager::new(&temp_file, 100);

        manager.add("test command", true);
        assert_eq!(manager.entries.len(), 1);

        let deleted = manager.delete("test command");
        assert!(deleted);
        assert_eq!(manager.entries.len(), 0);

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_history_manager_stats() {
        let temp_file = std::env::temp_dir().join("test_history_stats.json");
        let mut manager = HistoryManager::new(&temp_file, 100);

        manager.add("cmd1", true);
        manager.add("cmd2", true);
        manager.add("cmd1", true); // Duplicate, should increase count

        let stats = manager.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_executions, 3);
        assert_eq!(stats.unique_commands, 2);

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_history_manager_save_load() {
        let temp_file = std::env::temp_dir().join("test_history_save_load.json");
        let mut manager1 = HistoryManager::new(&temp_file, 100);

        manager1.add("command1", true);
        manager1.add("command2", true);
        manager1.save().unwrap();

        let mut manager2 = HistoryManager::new(&temp_file, 100);
        manager2.load().unwrap();

        assert_eq!(manager2.entries.len(), 2);
        assert_eq!(manager2.entries[0].command, "command1");
        assert_eq!(manager2.entries[1].command, "command2");

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_sort_strategies() {
        let temp_file = std::env::temp_dir().join("test_history_sort.json");
        let mut manager = HistoryManager::new(&temp_file, 100);

        manager.add("old_frequent", true);
        for _ in 0..10 {
            manager.add("old_frequent", true);
        }

        thread::sleep(Duration::from_millis(10));

        manager.add("new_rare", true);

        // Time sort: new_rare should be first
        let time_sorted = manager.all(SortStrategy::Time);
        assert_eq!(time_sorted[0].command, "new_rare");

        // Frequency sort: old_frequent should be first
        let freq_sorted = manager.all(SortStrategy::Frequency);
        assert_eq!(freq_sorted[0].command, "old_frequent");

        let _ = fs::remove_file(temp_file);
    }
}
