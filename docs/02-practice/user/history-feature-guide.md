# 命令历史记录功能使用指南

**功能版本**: RealConsole v0.7.0
**实现阶段**: Phase 8 Day 5-7
**最后更新**: 2025-10-16

## 概述

RealConsole 提供了强大的命令历史记录功能，支持：
- ✨ **持久化存储** - 历史记录在会话间保持
- ✨ **智能排序** - 结合命令频率和时间新鲜度
- ✨ **交互式搜索** - Ctrl+R 反向搜索（类似 zsh）
- ✨ **统计分析** - 查看命令使用统计
- ✨ **关键词高亮** - 搜索结果高亮显示

## 功能特性

### 1. 自动记录

所有执行的命令都会自动记录到历史中，包括：
- Shell 命令（`!command`）
- 自然语言对话
- 系统命令（`/command`）会被排除

### 2. 智能排序算法

历史记录使用智能得分算法排序：
```
Score = Frequency × 0.7 + Recency × 0.3
```

- **频率得分**：基于命令执行次数（对数归一化）
- **时间得分**：基于最后执行时间（7天半衰期）

这样既考虑了常用命令，又不会让旧命令永远占据顶部。

### 3. 持久化存储

历史记录保存在：
```
~/.realconsole/history.json
```

格式示例：
```json
[
  {
    "command": "!git status",
    "first_timestamp": "2025-10-16T08:00:00Z",
    "last_timestamp": "2025-10-16T09:30:00Z",
    "count": 5,
    "last_success": true
  }
]
```

## 使用方法

### `/history` 命令

#### 查看最近历史

```bash
/history
```

显示最近 20 条历史记录（智能排序）：
```
最近的历史记录 (智能排序)

  1. !git status (3x) 10-16 09:30
  2. !cargo test 10-16 09:25
  3. !ls -la 10-16 09:20
```

#### 搜索历史

```bash
/history search <keyword>
```

示例：
```bash
/history search git
```

输出：
```
搜索结果: 'git' (找到 3 条) (智能排序)

  1. !git status (3x) 10-16 09:30
  2. !git log --oneline 10-16 09:15
  3. !git diff 10-16 08:45
```

关键词会在结果中高亮显示。

#### 查看统计信息

```bash
/history stats
```

输出：
```
历史记录统计

  总记录数:     25
  总执行次数:   47
  唯一命令数:   25
  平均执行次数: 1.9
```

#### 清空历史

```bash
/history clear
```

**警告**：此操作不可撤销！

### Ctrl+R 交互式搜索

RealConsole 集成了 rustyline 的反向历史搜索功能（类似 bash/zsh 的 Ctrl+R）。

#### 使用步骤

1. **启动搜索**：按 `Ctrl+R`
2. **输入关键词**：如 `git`
3. **浏览匹配项**：
   - 再按 `Ctrl+R` 查看下一个匹配项
   - 按 `Ctrl+S` 查看上一个匹配项（如果支持）
4. **执行或取消**：
   - 按 `Enter` 执行当前命令
   - 按 `Ctrl+G` 或 `Esc` 取消搜索

#### 示例会话

```
$ ./realconsole
RealConsole v0.7.0 | 直接输入问题或 /help | Ctrl-D 退出

hongxin real-console % [Ctrl+R]
(reverse-i-search)`git': !git status

[再按 Ctrl+R]
(reverse-i-search)`git': !git log --oneline

[按 Enter 执行]
```

### 其他历史导航

- **↑ / ↓** - 浏览上一个/下一个命令
- **Ctrl+P / Ctrl+N** - 上一个/下一个命令（Emacs 风格）

## 技术细节

### 架构设计

遵循"一分为三"设计哲学：

1. **存储层** (`src/history.rs`)
   - JSON 文件持久化
   - 智能得分算法
   - 自动保存机制

2. **逻辑层** (`src/commands/history_cmd.rs`)
   - `/history` 命令实现
   - 搜索和排序逻辑
   - 关键词高亮

3. **展示层** (`src/repl.rs`, `src/agent.rs`)
   - Agent 集成
   - rustyline 历史同步
   - Ctrl+R 交互式搜索

### 配置参数

在 `src/history.rs` 中：
```rust
pub fn default() -> Self {
    Self::new("~/.realconsole/history.json", 1000)
    //                                        ^^^^ 最大历史记录数
}
```

在 `src/repl.rs` 中：
```rust
rl.set_max_history_size(1000)?;  // rustyline 历史大小
rl.set_history_ignore_dups(true)?;  // 忽略连续重复
rl.set_auto_add_history(true);  // 自动添加历史
```

### 数据结构

```rust
pub struct HistoryEntry {
    pub command: String,
    pub first_timestamp: DateTime<Utc>,
    pub last_timestamp: DateTime<Utc>,
    pub count: u32,
    pub last_success: bool,
}
```

### 得分算法

```rust
pub fn score(&self) -> f64 {
    // 频率得分（对数归一化）
    let frequency_score = (self.count as f64).ln() / 10.0;

    // 时间新鲜度得分（7天半衰期）
    let age_days = (Utc::now() - self.last_timestamp).num_seconds() as f64 / 86400.0;
    let recency_score = (-age_days / 7.0).exp();

    // 综合得分
    frequency_score * 0.7 + recency_score * 0.3
}
```

## 测试验证

### 单元测试

```bash
# 运行历史模块测试
cargo test --lib history

# 输出
running 16 tests
test history::tests::test_history_entry_creation ... ok
test history::tests::test_history_manager_add ... ok
test history::tests::test_history_manager_search ... ok
test commands::history_cmd::tests::test_show_recent_history ... ok
... (all 16 tests pass)
```

### 功能测试

使用提供的测试脚本：
```bash
./scripts/test_history_integration.sh
./scripts/test_ctrl_r_demo.sh
```

## 故障排除

### 历史文件损坏

如果遇到 JSON 解析错误：
```bash
rm ~/.realconsole/history.json
```

历史会自动重新创建。

### Ctrl+R 不工作

1. 确认使用的是最新版本：`./realconsole --version`
2. 确认 rustyline 版本：`grep rustyline Cargo.toml`（应该是 14.0）
3. 尝试重新启动 REPL

### 历史未持久化

检查目录权限：
```bash
ls -la ~/.realconsole/
chmod 700 ~/.realconsole
```

## 最佳实践

1. **定期备份历史**
   ```bash
   cp ~/.realconsole/history.json ~/backups/
   ```

2. **搜索技巧**
   - 使用简短的关键词（如 `git` 而不是 `git status`）
   - Ctrl+R 支持增量搜索

3. **清理旧记录**
   ```bash
   /history clear  # 清空全部
   ```

## 相关文档

- [开发者指南](../developer/developer-guide.md)
- [Phase 8 开发日志](../../03-evolution/phases/phase8-summary.md)
- [History API 文档](../../01-understanding/api/history-api.md)

## 反馈与改进

遇到问题或有改进建议？
- 查看 [GitHub Issues](https://github.com/hongxin/realconsole/issues)
- 阅读 [CLAUDE.md](../../../CLAUDE.md) 了解项目哲学
