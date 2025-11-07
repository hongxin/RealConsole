fix: v1.28.1 - 统一回合系统完整实施（Unified Round System）

## 核心修复

### 1. Shell/System 命令回合化
- 添加 RoundType 枚举（Llm/Shell/System）
- Shell/System 命令统一使用回合消息协议
- 前端根据类型适配显示（图标、标签、渲染方式）

### 2. 双视图模式完整兼容
- 回合模式：所有类型统一显示为卡片
- 传统模式：双路显示策略（数据 + 显示分离）
- 修复命令重复显示问题
- 修复 LLM 输出重复显示问题

## Bug 修复清单

**Bug 1: 回合模式下 Shell/System 命令无输出**
- 原因：v1.28.0 回合系统未覆盖 Shell/System
- 修复：统一所有交互类型为回合系统
- 文件：session.rs (+9), websocket.rs (+80), server.rs (+15)

**Bug 2: 传统模式下命令重复显示**
- 原因：handleSubmit() 和 round_start 都显示命令
- 修复：移除 round_start 的重复显示逻辑
- 文件：server.rs (case 'round_start')

**Bug 3: 传统模式下 LLM 输出重复显示**
- 原因：stream 消息 + round_complete 都显示输出
- 修复：round_complete 只显示 Shell/System 输出
- 文件：server.rs (case 'round_complete')

## 技术架构

### 数据层（统一）
```rust
pub enum RoundType { Llm, Shell, System }
pub struct ConversationRound {
    pub round_type: RoundType,
    // ... 其他字段
}
```

### 协议层（统一）
- 所有交互使用 RoundStart/RoundComplete 消息
- 不再使用 Output 消息（特殊情况除外）

### 显示层（分离）
- 回合模式：卡片显示，类型适配
- 传统模式：流式输出，双路策略

## 代码统计

- **修改文件**: 4 个
- **新增代码**: ~119 行
- **关键修复**: 3 处
- **测试场景**: 9 个

## 影响范围

- ✅ Web 终端：完整的统一回合系统
- ✅ CLI 版本：无影响
- ✅ 配置：无需变更
- ✅ 向后兼容：完全兼容

## 测试验证

### 回合模式
- ✅ Shell 命令（pwd, ls）：卡片显示正确
- ✅ System 命令（/help）：卡片显示正确
- ✅ LLM 对话（hello）：卡片显示正确

### 传统模式
- ✅ Shell 命令：无重复输出
- ✅ LLM 对话：无重复输出
- ✅ 模式切换：历史完整保留

## 里程碑意义

v1.28.1 完成了 v1.28.0 的核心目标：**真正的统一回合系统**。

这为未来的 Cell 执行模型（v1.31.0）和 Notebook 体验（v1.32.0）奠定了坚实的架构基础。

下一步：v1.29.0 - 回合操作增强（删除、重执行、导出、快捷键）

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
