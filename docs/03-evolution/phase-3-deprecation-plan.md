# Phase 3: API 迁移与弃用标记

**版本**: v1.3.0-beta
**作者**: RealConsole Contributors
**日期**: 2025-10-20
**状态**: 进行中

## 概述

Phase 3 是服务层架构重构的第三阶段，主要目标是引导用户从旧 API 迁移到新的服务层 API。通过添加 `#[deprecated]` 标记，我们在保持 100% 向后兼容的同时，为 v2.0.0 的完全迁移做准备。

## 设计原则

1. **非破坏性迁移** - 所有旧 API 继续工作，只添加编译警告
2. **清晰的迁移路径** - 每个废弃项都有明确的替代方案
3. **内部代码现代化** - Agent 内部优先使用服务层 API
4. **文档完善** - 更新所有相关文档指向新 API

## 废弃计划

### 3.1 字段访问器废弃

以下访问器方法将被标记为 `#[deprecated]`：

| 旧 API | 新 API | 说明 |
|--------|--------|------|
| `agent.memory()` | `agent.state_manager().memory()` | 记忆系统访问 |
| `agent.exec_logger()` | `agent.state_manager().exec_logger()` | 执行日志访问 |
| `agent.history()` | `agent.state_manager().history()` | 命令历史访问 |
| `agent.stats_collector()` | `agent.state_manager().stats_collector()` | 统计收集器访问 |
| `agent.context_tracker()` | `agent.state_manager().context_tracker()` | 上下文追踪器访问 |

**废弃消息模板**:
```rust
#[deprecated(
    since = "1.3.0",
    note = "Use `state_manager().{component}()` instead for better encapsulation"
)]
```

### 3.2 不废弃的访问器

以下访问器**暂不废弃**（Phase 4 再考虑）：

- `llm_manager()` - LLM 底层管理器，某些场景仍需直接访问
- `tool_registry()` - 工具注册表，命令注册时需要
- `conversation_manager()` - 对话管理器，服务层暂未封装

**理由**: 这些组件的服务封装尚未完全覆盖所有用例，贸然废弃会增加迁移成本。

### 3.3 公共字段废弃

以下 `pub` 字段将在 v2.0.0 移除（Phase 3 添加文档警告）：

```rust
// ⚠ v2.0.0 will be private - use state_manager() instead
pub memory: Arc<RwLock<Memory>>,
pub exec_logger: Arc<RwLock<ExecutionLogger>>,
pub history: Arc<RwLock<HistoryManager>>,
pub stats_collector: Arc<StatsCollector>,
pub context_tracker: Arc<RwLock<ContextTracker>>,
```

**注意**: Rust 目前不支持 `#[deprecated]` 修饰 struct 字段，只能通过文档注释警告。

## 实现步骤

### Step 1: 添加废弃标记 ✅

**文件**: `src/agent.rs`

为旧访问器添加 `#[deprecated]` 属性：

```rust
#[deprecated(since = "1.3.0", note = "Use `state_manager().memory()` instead")]
pub fn memory(&self) -> Arc<RwLock<Memory>> {
    Arc::clone(&self.memory)
}

#[deprecated(since = "1.3.0", note = "Use `state_manager().exec_logger()` instead")]
pub fn exec_logger(&self) -> Arc<RwLock<ExecutionLogger>> {
    Arc::clone(&self.exec_logger)
}

#[deprecated(since = "1.3.0", note = "Use `state_manager().history()` instead")]
pub fn history(&self) -> Arc<RwLock<HistoryManager>> {
    Arc::clone(&self.history)
}

#[deprecated(since = "1.3.0", note = "Use `state_manager().stats_collector()` instead")]
pub fn stats_collector(&self) -> Arc<StatsCollector> {
    Arc::clone(&self.stats_collector)
}

#[deprecated(since = "1.3.0", note = "Use `state_manager().context_tracker()` instead")]
pub fn context_tracker(&self) -> Arc<RwLock<ContextTracker>> {
    Arc::clone(&self.context_tracker)
}
```

### Step 2: 更新 main.rs

**文件**: `src/main.rs`

将命令注册从直接访问改为通过 StateManager：

```rust
// 旧代码（触发废弃警告）
let stats_collector = agent.stats_collector();
let memory = agent.memory();
let exec_logger = agent.exec_logger();
let history = agent.history();

// 新代码（推荐）
let state_manager = agent.state_manager();
let stats_collector = state_manager.stats_collector();
let memory = state_manager.memory();
let exec_logger = state_manager.exec_logger();
let history = state_manager.history();
```

### Step 3: 更新 commands 模块

**影响文件**:
- `src/commands/mod.rs`
- `src/commands/memory_cmd.rs`
- `src/commands/log_cmd.rs`
- `src/commands/stats_cmd.rs`
- `src/commands/history_cmd.rs`

所有命令注册函数从：
```rust
pub fn register_memory_commands(registry: &mut CommandRegistry, memory: Arc<RwLock<Memory>>)
```

改为：
```rust
pub fn register_memory_commands(registry: &mut CommandRegistry, state_manager: &StateManager)
```

### Step 4: 更新文档

**文件**: `docs/02-practice/developer/services-guide.md`

添加废弃警告章节：

```markdown
## ⚠️ API 废弃警告（v1.3.0）

以下旧 API 已被标记为废弃，将在 v2.0.0 中移除：

| 废弃 API | 替代 API | 迁移难度 |
|----------|----------|----------|
| `agent.memory()` | `agent.state_manager().memory()` | 简单 |
| `agent.exec_logger()` | `agent.state_manager().exec_logger()` | 简单 |
| `agent.history()` | `agent.state_manager().history()` | 简单 |
| `agent.stats_collector()` | `agent.state_manager().stats_collector()` | 简单 |
| `agent.context_tracker()` | `agent.state_manager().context_tracker()` | 简单 |

**迁移示例**:

```rust
// ❌ 废弃写法
let memory = agent.memory();
memory.write().await.add(EntryType::UserQuery, "test");

// ✅ 推荐写法
let memory = agent.state_manager().memory();
memory.write().await.add(EntryType::UserQuery, "test");
```
```

### Step 5: 测试验证

**验证清单**:
- [ ] 所有 674 个测试通过
- [ ] `cargo build` 产生预期的废弃警告
- [ ] `cargo clippy` 无新增警告（允许废弃警告）
- [ ] 用户代码仍可正常使用旧 API（100% 向后兼容）

## 预期影响

### 编译输出

用户使用旧 API 时会看到：

```
warning: use of deprecated method `agent::Agent::memory`: Use `state_manager().memory()` instead
  --> src/main.rs:334:30
   |
334|     let memory = agent.memory();
   |                        ^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default
```

### 代码现代化

- **main.rs**: 5 处旧 API 调用 → 新 API
- **commands/**: ~15 处函数签名更新
- **测试代码**: 保持不变（允许使用废弃 API 以测试兼容性）

### 文档更新

- services-guide.md: +60 行废弃警告说明
- CHANGELOG.md: 添加 v1.3.0-beta 条目

## 兼容性保证

### Phase 3（当前）
- ✅ 所有旧 API 正常工作
- ⚠️ 编译时显示废弃警告
- ✅ 旧代码无需修改即可运行

### Phase 4（v2.0.0）
- ❌ 移除废弃的访问器方法
- ❌ 部分 pub 字段改为 private
- 🔄 提供自动迁移脚本

## 时间线

| 阶段 | 版本 | 状态 | 说明 |
|------|------|------|------|
| Phase 2 | v1.3.0-alpha | ✅ 完成 | 服务层基础架构 |
| Phase 3 | v1.3.0-beta | 🚧 进行中 | API 废弃标记 |
| Phase 4 | v2.0.0 | 📅 计划中 | 移除旧 API |

## 回滚计划

如果 Phase 3 引入问题：

1. **移除废弃标记** - 简单删除 `#[deprecated]` 属性
2. **恢复 main.rs** - Git revert 到 Phase 2 状态
3. **保留服务层** - StateManager 和服务依然可用

## 参考资料

- [Phase 2 重构设计](./agent-refactoring-v1.3.md)
- [服务层使用指南](../02-practice/developer/services-guide.md)
- [Rust Deprecation Best Practices](https://rust-lang.github.io/api-guidelines/future-proofing.html)

---

**维护者**: RealConsole Contributors
**许可**: MIT
