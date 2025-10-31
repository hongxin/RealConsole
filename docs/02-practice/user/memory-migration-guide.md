# Memory 数据迁移指南

**版本**: v1.16.0
**适用场景**: 从旧版 Memory 系统迁移到 UnifiedTracer 统一存储
**更新日期**: 2025-10-31

---

## 📚 目录

- [背景说明](#背景说明)
- [为什么要迁移](#为什么要迁移)
- [迁移前准备](#迁移前准备)
- [迁移方法](#迁移方法)
- [验证迁移结果](#验证迁移结果)
- [常见问题](#常见问题)
- [回滚方案](#回滚方案)

---

## 背景说明

### v1.16.0 重大变更

从 **v1.16.0** 开始，RealConsole 的 Memory 模块进行了架构升级：

**旧架构**:
```
Memory ────> 独立的 JSONL 存储
  ├─ memory.jsonl
  └─ 独立的搜索索引
```

**新架构** (v1.16.0):
```
MemoryManager (适配层) ────> UnifiedTracer (统一存储)
                              ├─ Statistics 维度
                              ├─ Coordination 维度
                              ├─ BlackBox 维度
                              └─ Memory 维度 ⭐
```

### 核心变化

| 维度 | 旧实现 | 新实现 (v1.16.0) |
|------|--------|------------------|
| **存储** | 独立 JSONL 文件 | UnifiedTracer 统一存储 |
| **API** | 同步方法 | **异步方法**（添加 `async` 关键字）|
| **功能** | 基础 CRUD | 增强：tags、importance、context_id |
| **并发** | 单线程友好 | **多线程优化**（2-4x 性能提升）|

---

## 为什么要迁移

### ✅ 迁移的好处

1. **统一架构** 🏗️
   - 四维观测体系（Statistics、Coordination、BlackBox、Memory）
   - 统一的查询和分析接口
   - 简化的系统架构

2. **功能增强** ⚡
   - ✨ **重要性标记**: 标记 Normal/Important/Critical 三级重要性
   - ✨ **标签系统**: 为记忆添加自定义标签，支持标签查询
   - ✨ **上下文关联**: 通过 context_id 关联相关记忆

3. **性能提升** 🚀
   - **多线程场景**: 2-4x 性能提升（得益于异步设计）
   - **并发安全**: Arc + RwLock 确保线程安全
   - **查询优化**: 统一的查询引擎

4. **更好的查询能力** 🔍
   ```rust
   // 新增查询方法
   manager.search_by_tag("learning").await?;     // 标签查询
   manager.find_important().await?;               // 重要记忆
   manager.find_by_context("project-x").await?;   // 上下文关联
   ```

### ⚠️ API 变更

所有方法都需要添加 `.await`：

```rust
// 旧代码 (v1.15.x)
manager.add("Hello".to_string(), EntryType::User);
let recent = manager.recent(10)?;

// 新代码 (v1.16.0)
manager.add("Hello".to_string(), EntryType::User).await;
let recent = manager.recent(10).await?;
```

---

## 迁移前准备

### 1. 备份现有数据 ⚠️

**强烈建议**在迁移前备份所有记忆数据！

```bash
# 查找 Memory 数据文件
find ~/.realconsole -name "memory.jsonl" -o -name "*_memory.jsonl"

# 创建备份
cp ~/.realconsole/memory/memory.jsonl ~/.realconsole/memory/memory.jsonl.backup.$(date +%Y%m%d)

# 或者备份整个 memory 目录
tar -czf ~/.realconsole/memory_backup_$(date +%Y%m%d).tar.gz ~/.realconsole/memory/
```

### 2. 检查数据格式

确认旧数据格式正确：

```bash
# 查看前几行
head -5 ~/.realconsole/memory/memory.jsonl

# 验证 JSON 格式
cat ~/.realconsole/memory/memory.jsonl | while read line; do
    echo "$line" | jq . > /dev/null || echo "Invalid JSON: $line"
done
```

**正确的 JSONL 格式**：
```jsonl
{"timestamp":"2025-10-31T10:00:00Z","type":"user","content":"学习 Rust","importance":"normal"}
{"timestamp":"2025-10-31T10:05:00Z","type":"assistant","content":"Rust 是系统编程语言","importance":"important"}
```

### 3. 确认环境版本

```bash
# 检查 RealConsole 版本
realconsole --version

# 应该显示 v1.16.0 或更高版本
# RealConsole v1.16.0
```

如果版本低于 v1.16.0，请先升级：

```bash
# 升级到最新版本
make install
```

---

## 迁移方法

### 方法 1: 使用迁移工具（推荐）✅

v1.16.0 提供了内置的迁移工具，自动处理数据转换。

#### 编程方式迁移

创建迁移脚本 `migrate_memory.rs`：

```rust
use anyhow::Result;
use realconsole::memory::{MemoryManager, MemoryMigrator};
use realconsole::tracer::unified_tracer::UnifiedTracer;
use realconsole::conversation::context_manager::ContextManager;
use realconsole::execution_logger::ExecutionLogger;
use realconsole::history::HistoryManager;
use realconsole::config::settings::ConversationConfig;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建 UnifiedTracer
    let history = Arc::new(RwLock::new(HistoryManager::new(
        PathBuf::from("~/.realconsole/history.jsonl"),
        1000,
    )));
    let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(1000)));
    let context = Arc::new(RwLock::new(ContextManager::new(
        ConversationConfig::default(),
    )));

    let tracer = Arc::new(UnifiedTracer::new(
        history,
        exec_logger,
        None,
        context,
    ));

    // 2. 创建 MemoryManager 和 Migrator
    let manager = MemoryManager::new(tracer, 10000);
    let migrator = MemoryMigrator::new(manager);

    // 3. 执行迁移
    println!("🚀 开始迁移 Memory 数据...");
    let report = migrator
        .migrate_from_file("~/.realconsole/memory/memory.jsonl")
        .await?;

    // 4. 显示迁移报告
    println!("{}", report.format());

    // 5. 检查迁移结果
    if report.is_success() {
        println!("✅ 迁移成功完成！");
        Ok(())
    } else {
        eprintln!("⚠️ 迁移完成但有错误，请检查报告");
        Err(anyhow::anyhow!("迁移失败：{} 个错误", report.failed))
    }
}
```

运行迁移：

```bash
# 编译并运行
cargo run --bin migrate_memory

# 或者使用 cargo script（如果已安装）
cargo +nightly -Zscript migrate_memory.rs
```

#### 迁移报告示例

```
━━━━━ Memory 数据迁移报告 ━━━━━
总条目数: 1523
✅ 成功迁移: 1520
⏭️  跳过: 2
❌ 失败: 1
成功率: 99.8%

错误详情:
  1. Skipped: 行 42: 空行
  2. Skipped: 行 103: 空行
  3. Failed: 行 567: JSON 解析失败: unexpected character

━━━━━━━━━━━━━━━━━━━━━━━━
```

### 方法 2: 手动迁移

如果你只有少量记忆数据，可以手动迁移：

```rust
use realconsole::memory::{MemoryManager, EntryType, Importance};

#[tokio::main]
async fn main() -> Result<()> {
    // 创建 MemoryManager（省略 tracer 创建步骤）
    let manager = MemoryManager::new(tracer, 1000);

    // 手动添加旧记忆
    manager.add(
        "学习 Rust 语言".to_string(),
        EntryType::User,
    ).await;

    // 添加带重要性的记忆
    manager.add_with_importance(
        "重要项目决策".to_string(),
        EntryType::Assistant,
        Importance::Important,
    ).await;

    println!("✅ 手动迁移完成");
    Ok(())
}
```

### 方法 3: 批量迁移

如果有多个 JSONL 文件需要迁移：

```bash
#!/bin/bash
# batch_migrate.sh

MEMORY_DIR="$HOME/.realconsole/memory"
BACKUP_DIR="$HOME/.realconsole/memory_backup_$(date +%Y%m%d)"

# 创建备份
mkdir -p "$BACKUP_DIR"
cp -r "$MEMORY_DIR"/* "$BACKUP_DIR/"

# 迁移所有 JSONL 文件
for file in "$MEMORY_DIR"/*.jsonl; do
    echo "迁移文件: $file"
    cargo run --bin migrate_memory -- "$file"
done

echo "✅ 批量迁移完成"
```

---

## 验证迁移结果

### 1. 检查数据完整性

```rust
use realconsole::memory::MemoryManager;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = /* 创建 manager */;

    // 检查总条目数
    let total = manager.len().await;
    println!("迁移后记忆总数: {}", total);

    // 对比迁移前的行数
    // wc -l ~/.realconsole/memory/memory.jsonl
    // 应该相差不大（允许跳过的空行和失败行）

    // 获取统计信息
    let stats = manager.stats().await?;
    println!("统计信息: {:#?}", stats);

    Ok(())
}
```

### 2. 验证类型保留

确认所有类型（User/Assistant/System/Shell/Tool）都正确保留：

```rust
let all_entries = manager.dump().await?;

let mut type_counts = HashMap::new();
for entry in &all_entries {
    *type_counts.entry(entry.entry_type).or_insert(0) += 1;
}

println!("类型分布:");
for (entry_type, count) in type_counts {
    println!("  {:?}: {} 条", entry_type, count);
}
```

### 3. 测试搜索功能

```rust
// 测试基本搜索
let results = manager.search("Rust").await?;
println!("搜索 'Rust': {} 个结果", results.len());

// 测试新功能（如果使用了）
let important = manager.find_important().await?;
println!("重要记忆: {} 条", important.len());
```

---

## 常见问题

### Q1: 迁移会丢失数据吗？

**A**: 不会。迁移工具采用容错设计：
- ✅ 单个条目失败不影响其他条目
- ✅ 详细的错误报告
- ✅ 建议迁移前备份（保险起见）

### Q2: 迁移需要多长时间？

**A**: 取决于数据量：
- **< 1,000 条**: 几秒内完成
- **1,000 - 10,000 条**: 1-5 秒
- **> 10,000 条**: 每 10,000 条约 5-10 秒

### Q3: Assistant 消息类型会丢失吗？

**A**: **不会**。v1.16.0 Phase 4 已修复此问题：
- 原始类型保存在 metadata 中
- 往返转换完全保留类型
- 包含测试验证

### Q4: 可以回滚吗？

**A**: **可以**。请参考[回滚方案](#回滚方案)。

### Q5: 迁移后旧文件怎么处理？

**A**: 建议保留备份至少 30 天：
```bash
# 30 天后删除备份
find ~/.realconsole/memory_backup_* -mtime +30 -delete
```

### Q6: 迁移失败怎么办？

**A**: 检查错误报告：
1. **JSON 解析失败**: 手动修复 JSONL 文件中的无效行
2. **文件不存在**: 检查文件路径
3. **权限错误**: 确认文件可读权限

```bash
# 修复权限
chmod 644 ~/.realconsole/memory/memory.jsonl
```

### Q7: 性能会变差吗？

**A**: 取决于使用场景：
- **单线程**: 稍慢（2-4x），但仍在微秒级
- **多线程**: **更快**（2-4x）✅
- **推荐**: 多线程/高并发场景更适合

---

## 回滚方案

如果迁移后遇到问题，可以回滚到旧版本：

### 1. 降级 RealConsole

```bash
# 卸载当前版本
make uninstall

# 或手动删除
rm ~/.local/bin/realconsole

# 安装旧版本（假设备份在 ~/realconsole-v1.15.1）
cd ~/realconsole-v1.15.1
make install
```

### 2. 恢复备份数据

```bash
# 恢复 memory.jsonl
cp ~/.realconsole/memory/memory.jsonl.backup.20251031 \
   ~/.realconsole/memory/memory.jsonl

# 或恢复整个目录
tar -xzf ~/.realconsole/memory_backup_20251031.tar.gz -C ~/
```

### 3. 验证回滚

```bash
# 检查版本
realconsole --version

# 测试功能
realconsole
> /memory stats
```

---

## 最佳实践

### 迁移前

1. ✅ **完整备份**所有 memory 数据
2. ✅ 验证 JSONL 格式正确
3. ✅ 记录当前数据量（用于验证）
4. ✅ 在测试环境先试运行

### 迁移中

1. ✅ 使用自动迁移工具（推荐）
2. ✅ 保存迁移报告
3. ✅ 处理错误条目
4. ✅ 验证迁移进度

### 迁移后

1. ✅ 验证数据完整性
2. ✅ 测试核心功能
3. ✅ 保留备份至少 30 天
4. ✅ 更新相关代码（添加 `async`/`.await`）

---

## 技术支持

如果迁移过程中遇到问题：

1. **查看日志**: `~/.realconsole/logs/realconsole.log`
2. **提交 Issue**: https://github.com/hongxin/RealConsole/issues
3. **参考文档**:
   - [开发者指南](../developer/developer-guide.md)
   - [Phase 3 进度报告](../../../04-reports/v1.16.0-phase3-progress.md)
   - [代码审查报告](../../../04-reports/v1.16.0-phase3-code-review.md)

---

## 更新日志

### v1.16.0 (2025-10-31)
- ✨ 创建迁移指南
- ✨ 提供自动迁移工具
- ✨ 修复 Assistant 类型保留问题
- ✨ 添加并发安全性测试

---

**文档维护**: RealConsole Contributors
**最后更新**: 2025-10-31
**适用版本**: v1.16.0+
