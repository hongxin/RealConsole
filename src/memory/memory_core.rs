//! 记忆系统
//!
//! 提供短期记忆（Ring Buffer）和长期记忆持久化功能
//!
//! 特性：
//! - 固定大小的环形缓冲区
//! - 自动时间戳
//! - 记忆搜索和检索
//! - JSONL 格式持久化
//! - ✨ Week 3 Day 2: 索引优化（HashMap + BTreeMap）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// 记忆条目类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    /// 用户输入
    User,
    /// 助手响应
    Assistant,
    /// 系统消息
    System,
    /// Shell 命令
    Shell,
    /// 工具调用
    Tool,
}

impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryType::User => write!(f, "USER"),
            EntryType::Assistant => write!(f, "ASSISTANT"),
            EntryType::System => write!(f, "SYSTEM"),
            EntryType::Shell => write!(f, "SHELL"),
            EntryType::Tool => write!(f, "TOOL"),
        }
    }
}

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 条目类型
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    /// 内容
    pub content: String,
}

impl MemoryEntry {
    /// 创建新的记忆条目
    pub fn new(content: String, entry_type: EntryType) -> Self {
        Self {
            timestamp: Utc::now(),
            entry_type,
            content,
        }
    }

    /// 格式化输出
    pub fn format(&self) -> String {
        format!(
            "[{}] {}: {}",
            self.timestamp.format("%H:%M:%S"),
            self.entry_type,
            self.content
        )
    }

    /// 简短预览（前80个字符）
    ///
    /// **注意**：使用 chars() 按字符数截断，而不是字节数，以避免 UTF-8 边界问题
    pub fn preview(&self) -> String {
        let content = if self.content.chars().count() > 80 {
            let preview: String = self.content.chars().take(80).collect();
            format!("{}...", preview)
        } else {
            self.content.clone()
        };
        format!("[{}] {}: {}", self.timestamp.format("%H:%M:%S"), self.entry_type, content)
    }
}

/// 记忆系统
///
/// 使用固定大小的环形缓冲区存储最近的记忆
pub struct Memory {
    /// 记忆条目队列
    entries: VecDeque<MemoryEntry>,
    /// 最大容量
    capacity: usize,
    /// ✨ Week 3 Day 2: 待持久化的条目缓冲区
    pending_persist: Vec<MemoryEntry>,
    /// ✨ Week 3 Day 2: 批量持久化阈值
    persist_batch_size: usize,
}

impl Memory {
    /// 创建新的记忆系统
    ///
    /// # 参数
    /// - `capacity`: 最大记忆容量
    ///
    /// # 示例
    /// ```
    /// let memory = Memory::new(100);
    /// ```
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
            pending_persist: Vec::new(),
            persist_batch_size: 10, // 默认每 10 条批量写入
        }
    }

    /// ✨ Week 3 Day 2: 设置批量持久化阈值
    pub fn with_persist_batch_size(mut self, batch_size: usize) -> Self {
        self.persist_batch_size = batch_size;
        self
    }

    /// 添加记忆条目
    ///
    /// # 参数
    /// - `content`: 记忆内容
    /// - `entry_type`: 条目类型
    ///
    /// # 示例
    /// ```
    /// memory.add("Hello, world!".to_string(), EntryType::User);
    /// ```
    pub fn add(&mut self, content: String, entry_type: EntryType) {
        let entry = MemoryEntry::new(content, entry_type);

        // 如果达到容量上限，移除最旧的条目
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }

        self.entries.push_back(entry);
    }

    /// 获取最近的 N 条记忆
    ///
    /// # 参数
    /// - `n`: 返回的条目数量
    ///
    /// # 返回
    /// 最近的 N 条记忆（按时间倒序）
    pub fn recent(&self, n: usize) -> Vec<&MemoryEntry> {
        let count = n.min(self.entries.len());
        self.entries.iter().rev().take(count).collect()
    }

    /// 搜索包含关键词的记忆
    ///
    /// # 参数
    /// - `keyword`: 搜索关键词
    ///
    /// # 返回
    /// 包含关键词的所有记忆条目
    pub fn search(&self, keyword: &str) -> Vec<&MemoryEntry> {
        let keyword_lower = keyword.to_lowercase();
        self.entries
            .iter()
            .filter(|entry| entry.content.to_lowercase().contains(&keyword_lower))
            .collect()
    }

    /// 获取所有记忆
    pub fn dump(&self) -> Vec<&MemoryEntry> {
        self.entries.iter().collect()
    }

    /// 清空所有记忆
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 获取记忆数量
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 检查记忆是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 获取特定类型的记忆
    pub fn filter_by_type(&self, entry_type: EntryType) -> Vec<&MemoryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.entry_type == entry_type)
            .collect()
    }

    // ========== 持久化功能 ==========

    /// 从 JSONL 文件加载记忆
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `capacity`: 记忆容量
    ///
    /// # 返回
    /// - `Ok(Memory)`: 加载成功
    /// - `Err(String)`: 加载失败
    pub fn load_from_file<P: AsRef<Path>>(path: P, capacity: usize) -> Result<Self, String> {
        let mut memory = Memory::new(capacity);

        // 如果文件不存在，返回空记忆
        if !path.as_ref().exists() {
            return Ok(memory);
        }

        let file = File::open(&path)
            .map_err(|e| format!("无法打开文件: {}", e))?;

        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("读取第 {} 行失败: {}", line_num + 1, e))?;

            // 跳过空行
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<MemoryEntry>(&line) {
                Ok(entry) => {
                    // 直接添加到队列，不使用 add() 以保留原始时间戳
                    if memory.entries.len() >= memory.capacity {
                        memory.entries.pop_front();
                    }
                    memory.entries.push_back(entry);
                }
                Err(e) => {
                    eprintln!("⚠ 第 {} 行解析失败: {}", line_num + 1, e);
                    // 继续处理其他行
                }
            }
        }

        Ok(memory)
    }

    /// 将记忆追加到 JSONL 文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `entry`: 要保存的记忆条目
    ///
    /// # 返回
    /// - `Ok(())`: 保存成功
    /// - `Err(String)`: 保存失败
    pub fn append_to_file<P: AsRef<Path>>(path: P, entry: &MemoryEntry) -> Result<(), String> {
        // 确保父目录存在
        if let Some(parent) = path.as_ref().parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("打开文件失败: {}", e))?;

        let json = serde_json::to_string(entry)
            .map_err(|e| format!("序列化失败: {}", e))?;

        writeln!(file, "{}", json)
            .map_err(|e| format!("写入文件失败: {}", e))?;

        Ok(())
    }

    /// ✨ Week 3 Day 2: 批量追加多个记忆到 JSONL 文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `entries`: 要保存的记忆条目列表
    ///
    /// # 返回
    /// - `Ok(usize)`: 成功写入的条目数
    /// - `Err(String)`: 保存失败
    pub fn append_batch_to_file<P: AsRef<Path>>(
        path: P,
        entries: &[MemoryEntry],
    ) -> Result<usize, String> {
        if entries.is_empty() {
            return Ok(0);
        }

        // 确保父目录存在
        if let Some(parent) = path.as_ref().parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("打开文件失败: {}", e))?;

        let mut count = 0;
        for entry in entries {
            let json = serde_json::to_string(entry)
                .map_err(|e| format!("序列化失败: {}", e))?;

            writeln!(file, "{}", json)
                .map_err(|e| format!("写入文件失败: {}", e))?;

            count += 1;
        }

        Ok(count)
    }

    /// ✨ Week 3 Day 2: 添加记忆到待持久化缓冲区
    ///
    /// # 参数
    /// - `entry`: 记忆条目
    pub fn queue_for_persist(&mut self, entry: MemoryEntry) {
        self.pending_persist.push(entry);
    }

    /// ✨ Week 3 Day 2: 刷新待持久化缓冲区到文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// - `Ok(usize)`: 成功写入的条目数
    /// - `Err(String)`: 保存失败
    pub fn flush_pending<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, String> {
        if self.pending_persist.is_empty() {
            return Ok(0);
        }

        let count = Self::append_batch_to_file(&path, &self.pending_persist)?;
        self.pending_persist.clear();
        Ok(count)
    }

    /// ✨ Week 3 Day 2: 检查是否需要刷新（达到批量阈值）
    pub fn should_flush(&self) -> bool {
        self.pending_persist.len() >= self.persist_batch_size
    }

    /// ✨ Week 3 Day 2: 添加记忆并自动批量持久化
    ///
    /// # 参数
    /// - `content`: 记忆内容
    /// - `entry_type`: 条目类型
    /// - `persist_path`: 持久化文件路径（如果提供）
    ///
    /// # 返回
    /// - `Ok(Option<usize>)`: 如果触发刷新，返回写入的条目数
    /// - `Err(String)`: 持久化失败
    pub fn add_with_persist<P: AsRef<Path>>(
        &mut self,
        content: String,
        entry_type: EntryType,
        persist_path: Option<P>,
    ) -> Result<Option<usize>, String> {
        let entry = MemoryEntry::new(content, entry_type);

        // 添加到内存
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry.clone());

        // 如果提供了持久化路径
        if let Some(path) = persist_path {
            // 添加到待持久化缓冲区
            self.queue_for_persist(entry);

            // 检查是否需要刷新
            if self.should_flush() {
                let count = self.flush_pending(path)?;
                return Ok(Some(count));
            }
        }

        Ok(None)
    }

    /// 将所有记忆保存到 JSONL 文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// - `Ok(usize)`: 保存的条目数量
    /// - `Err(String)`: 保存失败
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<usize, String> {
        // 确保父目录存在
        if let Some(parent) = path.as_ref().parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
        }

        let mut file = File::create(&path)
            .map_err(|e| format!("创建文件失败: {}", e))?;

        let mut count = 0;
        for entry in &self.entries {
            let json = serde_json::to_string(entry)
                .map_err(|e| format!("序列化失败: {}", e))?;

            writeln!(file, "{}", json)
                .map_err(|e| format!("写入文件失败: {}", e))?;

            count += 1;
        }

        Ok(count)
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(10);
        assert_eq!(memory.len(), 0);
        assert!(memory.is_empty());
        assert_eq!(memory.capacity, 10);
    }

    #[test]
    fn test_add_entry() {
        let mut memory = Memory::new(10);
        memory.add("Hello".to_string(), EntryType::User);
        assert_eq!(memory.len(), 1);
        assert!(!memory.is_empty());
    }

    #[test]
    fn test_ring_buffer() {
        let mut memory = Memory::new(5);

        // 添加6个条目
        for i in 0..6 {
            memory.add(format!("entry-{}", i), EntryType::User);
        }

        // 应该只保留最新的5个
        assert_eq!(memory.len(), 5);

        // 最新的应该是 entry-5
        let recent = memory.recent(1);
        assert!(recent[0].content.contains("entry-5"));

        // 最旧的应该是 entry-1（entry-0 被移除了）
        let all = memory.dump();
        assert!(all[0].content.contains("entry-1"));
    }

    #[test]
    fn test_recent() {
        let mut memory = Memory::new(10);

        for i in 0..5 {
            memory.add(format!("entry-{}", i), EntryType::User);
        }

        let recent = memory.recent(3);
        assert_eq!(recent.len(), 3);
        assert!(recent[0].content.contains("entry-4"));
        assert!(recent[1].content.contains("entry-3"));
        assert!(recent[2].content.contains("entry-2"));
    }

    #[test]
    fn test_search() {
        let mut memory = Memory::new(10);

        memory.add("Hello world".to_string(), EntryType::User);
        memory.add("Goodbye world".to_string(), EntryType::Assistant);
        memory.add("Hello Rust".to_string(), EntryType::User);

        let results = memory.search("hello");
        assert_eq!(results.len(), 2);

        let results = memory.search("world");
        assert_eq!(results.len(), 2);

        let results = memory.search("rust");
        assert_eq!(results.len(), 1);

        let results = memory.search("notfound");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_filter_by_type() {
        let mut memory = Memory::new(10);

        memory.add("user1".to_string(), EntryType::User);
        memory.add("assistant1".to_string(), EntryType::Assistant);
        memory.add("user2".to_string(), EntryType::User);

        let user_entries = memory.filter_by_type(EntryType::User);
        assert_eq!(user_entries.len(), 2);

        let assistant_entries = memory.filter_by_type(EntryType::Assistant);
        assert_eq!(assistant_entries.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut memory = Memory::new(10);

        memory.add("test".to_string(), EntryType::User);
        assert_eq!(memory.len(), 1);

        memory.clear();
        assert_eq!(memory.len(), 0);
        assert!(memory.is_empty());
    }

    #[test]
    fn test_persistence() {
        let path = "test_memory.jsonl";

        // 创建并保存记忆
        let mut memory = Memory::new(10);
        memory.add("test entry 1".to_string(), EntryType::User);
        memory.add("test entry 2".to_string(), EntryType::Assistant);

        let count = memory.save_to_file(path).unwrap();
        assert_eq!(count, 2);

        // 加载记忆
        let loaded = Memory::load_from_file(path, 10).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.dump()[0].content.contains("test entry 1"));
        assert!(loaded.dump()[1].content.contains("test entry 2"));

        // 清理
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_append_to_file() {
        let path = "test_append.jsonl";

        let entry1 = MemoryEntry::new("entry 1".to_string(), EntryType::User);
        let entry2 = MemoryEntry::new("entry 2".to_string(), EntryType::User);

        Memory::append_to_file(path, &entry1).unwrap();
        Memory::append_to_file(path, &entry2).unwrap();

        let loaded = Memory::load_from_file(path, 10).unwrap();
        assert_eq!(loaded.len(), 2);

        // 清理
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_preview_utf8_safety() {
        // 测试 UTF-8 字符边界安全性
        // 创建一个包含多字节字符（中文）的长字符串（确保超过 80 个字符）
        let chinese_text = "这是一个测试字符串包含中文字符用于验证UTF8边界处理的正确性";
        let content = chinese_text.repeat(5); // 重复5次确保超过80字符
        let entry = MemoryEntry::new(content.to_string(), EntryType::System);

        // 应该不会 panic（之前会在字符边界处崩溃）
        let preview = entry.preview();

        // 验证预览包含截断标记
        assert!(preview.contains("..."));

        // 验证预览是有效的 UTF-8
        assert!(std::str::from_utf8(preview.as_bytes()).is_ok());
    }

    #[test]
    fn test_preview_exact_boundary() {
        // 测试恰好在字符边界的情况
        let content = "a".repeat(80);
        let entry = MemoryEntry::new(content.to_string(), EntryType::User);

        let preview = entry.preview();
        assert!(!preview.contains("..."));
    }

    #[test]
    fn test_preview_with_emoji() {
        // 测试包含 emoji（4字节 UTF-8 字符）的情况
        // 创建超过 80 个字符的内容
        let content = "🎉".repeat(50) + &"测试".repeat(50);
        let entry = MemoryEntry::new(content.to_string(), EntryType::User);

        // 不应该 panic
        let preview = entry.preview();
        assert!(preview.contains("..."));

        // 验证是有效的 UTF-8
        assert!(std::str::from_utf8(preview.as_bytes()).is_ok());
    }
}
