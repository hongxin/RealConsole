# RealConsole v1.28.1 发布说明

**发布日期**: 2025-11-07
**版本**: v1.28.1
**类型**: Bug 修复版本

---

## 🎯 版本概述

v1.28.1 是对 v1.28.0 的重要修复版本，解决了**统一回合系统**实施过程中发现的关键问题。

本版本完成了 v1.28.0 的核心目标：**所有交互类型（LLM、Shell、System）统一使用回合卡片显示**，并确保双视图模式（回合/传统）的完整兼容性。

---

## 🐛 Bug 修复

### 修复1: 回合模式下 Shell/System 命令无输出

**问题描述**：
- v1.28.0 回合模式下，执行 `ls`、`pwd` 等 Shell 命令或 `/help` 等系统命令时，回合卡片显示但无输出内容

**根本原因**：
- v1.28.0 的回合系统只覆盖了 LLM 对话
- Shell/System 命令仍使用传统的 `ServerMessage::Output` 消息
- 前端在回合模式下过滤了 `Output` 消息，导致无输出

**解决方案**：
- **后端**：Shell/System 命令统一使用回合系统（`RoundStart`/`RoundComplete` 消息）
- **前端**：根据回合类型（`llm`/`shell`/`system`）选择不同的渲染方式
  - LLM 对话：Markdown 渲染
  - Shell/System：`<pre>` 保留格式

**影响文件**：
- `src/web/session.rs`: 添加 `RoundType` 枚举
- `src/web/websocket.rs`: 重写 `execute_shell_command()`、`execute_system_command()`
- `src/web/server.rs`: 添加 `getRoundTypeConfig()`、修改 `completeRound()`

### 修复2: 传统模式下命令重复显示

**问题描述**：
- 传统模式下输入命令（如 `pwd`），命令行显示两次

**根本原因**：
- `handleSubmit()` 立即显示命令（第一次）
- 收到 `round_start` 消息后，又在传统模式下显示命令（第二次）

**解决方案**：
- 只保留 `handleSubmit()` 中的立即显示
- `round_start` 消息处理中移除传统模式的命令显示逻辑

**影响文件**：
- `src/web/server.rs`: 修改 `case 'round_start'` 处理逻辑

### 修复3: 传统模式下 LLM 对话输出重复

**问题描述**：
- 传统模式下与 LLM 对话时，AI 响应显示两次

**根本原因**：
- LLM 对话通过 `stream` 消息流式显示（第一次）
- 收到 `round_complete` 消息后，又在传统模式下显示完整输出（第二次）
- Shell/System 命令没有 `stream` 消息，只通过 `round_complete` 显示，所以正常

**解决方案**：
- `round_complete` 传统模式显示时，检查回合类型
- 只有 Shell/System 命令才额外显示输出
- LLM 对话已通过 `stream` 显示，跳过重复显示

**影响文件**：
- `src/web/server.rs`: 修改 `case 'round_complete'` 处理逻辑

---

## 🏗️ 技术改进

### 统一回合系统架构

**数据层**（后端）：
```rust
pub enum RoundType { Llm, Shell, System }

pub struct ConversationRound {
    pub round_type: RoundType,  // 新增类型字段
    // ... 其他字段
}
```

**协议层**（WebSocket）：
- 所有交互类型统一使用 `RoundStart`/`RoundComplete` 消息
- 不再使用 `Output` 消息（除了特殊情况如欢迎消息）

**显示层**（前端）：
- **回合模式**：显示回合卡片，根据类型选择图标和标签
  - LLM: `Round #1`
  - Shell: `💻 Shell #1`
  - System: `⚙️ System #1`
- **传统模式**：流式输出，双路显示策略
  - 数据：总是维护回合数据（后台）
  - 显示：根据消息类型选择显示方式

### 双路显示策略

**关键设计**：
```javascript
// RoundStart: 只创建数据，不额外显示（handleSubmit 已显示命令）
case 'round_start':
    terminal.createRound(msg.round);
    break;

// RoundComplete: 根据类型选择是否额外显示
case 'round_complete':
    terminal.completeRound(msg.round);  // 总是维护数据
    if (terminal.viewMode === 'stream') {
        // Shell/System: 额外显示输出
        // LLM: 已通过 stream 显示，跳过
        if (msg.round.round_type !== 'llm' && msg.round.ai_response) {
            terminal.writeOutput(msg.round.ai_response);
        }
    }
    break;
```

---

## 📊 代码统计

### 修改文件
- `Cargo.toml`: 版本 1.28.0 → 1.28.1
- `src/web/session.rs`: +9 行（RoundType enum）
- `src/web/websocket.rs`: +80 行（Shell/System 回合化）
- `src/web/server.rs`: +30 行（类型适配 + 重复修复）

### 总计
- **新增代码**: ~119 行
- **修改逻辑**: 3 处关键修复
- **影响文件**: 4 个

---

## 🎨 用户体验改进

### 改进前 (v1.28.0)
| 场景 | 回合模式 | 传统模式 |
|------|---------|---------|
| Shell 命令 | ❌ 无输出 | ✅ 正常 |
| System 命令 | ❌ 无输出 | ✅ 正常 |
| LLM 对话 | ✅ 正常 | ❌ 重复输出 |

### 改进后 (v1.28.1)
| 场景 | 回合模式 | 传统模式 |
|------|---------|---------|
| Shell 命令 | ✅ 卡片显示 | ✅ 流式输出 |
| System 命令 | ✅ 卡片显示 | ✅ 流式输出 |
| LLM 对话 | ✅ 卡片显示 | ✅ 流式输出 |

---

## 🚀 升级指南

从 v1.28.0 升级到 v1.28.1：

```bash
# 1. 拉取最新代码
git pull origin main

# 2. 重新编译
cargo build --release

# 3. 启动服务
export DEEPSEEK_API_KEY="your-api-key"
./target/release/realconsole web

# 4. 浏览器访问
http://127.0.0.1:7788
```

**无需配置更改**，完全向后兼容。

---

## 🧪 测试验证

### 回合模式测试
```bash
1. 默认回合模式
2. 输入: pwd        → ✅ 💻 Shell #1 卡片，显示路径
3. 输入: /help      → ✅ ⚙️ System #2 卡片，显示帮助
4. 输入: hello      → ✅ Round #3 卡片，显示 AI 响应
```

### 传统模式测试
```bash
1. 切换到传统模式
2. 输入: ls         → ✅ % ls + 文件列表（无重复）
3. 输入: hello      → ✅ % hello + AI 响应（无重复）
```

### 模式切换测试
```bash
1. 回合模式：执行 pwd、hello
2. 切换传统模式：执行 ls
3. 切换回回合模式
   → ✅ 看到 3 个回合卡片，历史完整
```

---

## 🔮 后续计划

v1.28.1 完成了统一回合系统的基础架构，为后续版本奠定了坚实基础：

### v1.29.0 (计划 2025-11)
- 回合操作增强（删除、重新执行、导出）
- 快捷键支持（Shift+Enter, Ctrl+/）
- 视图偏好持久化

### v1.30.0 (计划 2025-12)
- SQLite 持久化存储
- 会话历史管理

### v1.31.0 (计划 2026-01)
- Cell 执行模型
- In-place 编辑

---

## 📚 相关文档

- **复盘文档**: `docs/04-reports/v1.28.1-retrospective.md`
- **v1.28.0 发布说明**: `docs/04-reports/v1.28.0-release-notes.md`
- **过渡计划**: `docs/03-evolution/v1-to-v2-transition-plan.md`

---

**版本**: v1.28.1
**发布日期**: 2025-11-07
**许可**: MIT
**维护者**: RealConsole Contributors
