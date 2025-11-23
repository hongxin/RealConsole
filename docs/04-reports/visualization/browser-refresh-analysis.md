# 浏览器刷新场景分析

**日期**: 2025-01-23
**问题**: 用户报告"保存→加载"路径已修复，但"刷新浏览器页面"路径仍有问题

## 🤔 需要确认的问题

在深入修复前，我需要确认以下几点：

### 问题 1: 刷新后是否显示历史内容？

**场景**:
1. 用户生成一个图表
2. 用户按 F5 或 Cmd+R 刷新页面
3. 页面重新加载

**问题**: 刷新后，页面是否显示之前的对话历史和图表？

- **选项 A**: 页面完全空白，显示欢迎消息（这是预期行为）
- **选项 B**: 页面显示之前的对话历史（文本重复/图表丢失）

### 问题 2: 如何触发的问题？

**请确认触发问题的具体步骤**：

**路径 A: 完全刷新（F5/Cmd+R）**
```
1. 生成图表 → 图表显示正常
2. 按 F5 刷新页面
3. 页面重新加载
4. 结果：???
```

**路径 B: 保存后刷新**
```
1. 生成图表 → 图表显示正常
2. 点击"保存会话"
3. 按 F5 刷新页面
4. 页面重新加载（会话列表中有保存的会话）
5. 不点击"加载会话"，直接观察
6. 结果：???
```

**路径 C: 自动保存机制（如果存在）**
```
1. 生成图表 → 图表显示正常
2. 直接按 F5 刷新页面（系统可能自动保存了当前会话？）
3. 页面加载时自动恢复之前的会话？
4. 结果：显示历史但图表有问题
```

## 🔍 当前架构分析

### WebSocket 连接生命周期

```rust
// 每次 WebSocket 连接都创建新 Session
pub async fn new(socket: WebSocket, config: Config, registry: CommandRegistry) -> Self {
    let session = Arc::new(Session::new(config, registry).await);  // 新 Session
    Self { socket, session }
}
```

**意味着**：
- 每次刷新页面 → WebSocket 重新连接 → **新的 Session 对象**
- 新 Session 的 `rounds` 和 `chart_history` 都是**空的**

### 前端状态管理

```javascript
this.rounds = [];  // 前端维护的回合列表（在内存中）
```

**意味着**：
- 刷新页面 → JavaScript 重新加载 → `this.rounds` 被清空
- 除非有持久化机制（localStorage/sessionStorage），否则历史数据会丢失

### ServerMessage::RoundHistory（目前未使用）

```rust
// src/web/session.rs:263-264
#[serde(rename = "round_history")]
RoundHistory { rounds: Vec<ConversationRound> },
```

**注意**: 这个消息类型虽然定义了（用于"初始加载或重连"），但**在当前代码中没有找到发送的地方**！

## 💡 可能的场景

### 场景 A: 当前实现不支持刷新恢复（预期行为）

**描述**: 刷新页面后，页面完全空白，不显示任何历史

**原因**:
- 新 Session 没有历史数据
- 前端没有持久化机制
- 这是**当前的设计行为**

**解决方案**:
- 用户需要使用"保存会话"+"加载会话"功能来恢复历史
- 或者实现自动会话恢复机制（见下文）

### 场景 B: 存在某种缓存导致部分数据残留

**描述**: 刷新页面后，显示部分历史文本但图表丢失/重复

**可能原因**:
1. 浏览器缓存了部分 DOM 内容（不太可能）
2. Service Worker 缓存（如果启用）
3. 某个我们没有发现的持久化机制
4. HTTP 缓存导致旧的 HTML/JS 被加载

**解决方案**: 需要具体诊断

### 场景 C: 自动会话恢复机制（可能存在但不完整）

**描述**: 系统尝试自动恢复会话，但实现不完整

**可能原因**:
1. 服务器端有某种会话持久化（我们没有发现）
2. 前端有 localStorage 保存会话ID
3. RoundHistory 消息被发送了（但我们没找到）

**解决方案**: 需要完整审查代码

## 🧪 诊断步骤

### 步骤 1: 清除所有缓存测试

```bash
# 1. 清除会话文件
rm -rf ~/.realconsole/sessions/*

# 2. 启动服务器
DEEPSEEK_API_KEY="your-key" ./target/release/realconsole web --port 7788

# 3. 在浏览器中测试
# - 打开开发者工具 → Network 标签 → 勾选"Disable cache"
# - 打开开发者工具 → Application 标签 → 清除所有 Storage
# - 生成一个测试图表
# - 按 F5 刷新
# - 观察结果
```

### 步骤 2: 检查 WebSocket 消息

```javascript
// 在浏览器控制台运行
// 监听所有 WebSocket 消息
const originalOnMessage = ws.onmessage;
ws.onmessage = function(event) {
    console.log('[WS Received]', JSON.parse(event.data));
    originalOnMessage.call(this, event);
};
```

### 步骤 3: 检查 localStorage

```javascript
// 在浏览器控制台运行
console.log('localStorage keys:', Object.keys(localStorage));
Object.keys(localStorage).forEach(key => {
    console.log(`${key}:`, localStorage.getItem(key));
});
```

## 🔧 可能的解决方案

### 方案 A: 明确告知用户刷新不保留历史（最简单）

**实现**:
- 在文档中说明：刷新页面会清空当前会话
- 用户需要使用"保存会话"功能来持久化数据

**优点**:
- 无需修改代码
- 行为明确

**缺点**:
- 用户体验不够好

### 方案 B: 实现自动会话恢复（推荐）

**实现步骤**:

1. **前端保存当前会话ID到 localStorage**
   ```javascript
   // 收到 SessionLoaded 或新建会话时
   localStorage.setItem('realconsole_current_session_id', session.id);
   ```

2. **页面加载时自动恢复上次会话**
   ```javascript
   ws.onopen = () => {
       const lastSessionId = localStorage.getItem('realconsole_current_session_id');
       if (lastSessionId) {
           // 发送 load_session 消息
           ws.send(JSON.stringify({
               type: 'load_session',
               session_id: lastSessionId
           }));
       }
   };
   ```

3. **服务器发送 RoundHistory + Chart 消息**
   - 已经在 `handle_load_session()` 中实现了！
   - 这个方案应该可以直接工作

**优点**:
- 用户体验好（无缝恢复）
- 复用已有的 load_session 逻辑

**缺点**:
- 需要修改前端代码

### 方案 C: WebSocket 重连时自动发送历史（复杂）

**实现**:
- 服务器维护全局 Session 池（按 session_id 索引）
- WebSocket 连接时，从池中获取已有 Session（如果存在）
- 自动发送 RoundHistory + Chart 消息

**优点**:
- 完全自动化
- 支持多标签页

**缺点**:
- 需要实现 Session 池管理
- 内存管理复杂（何时清理旧 Session？）
- 需要修改服务器端架构

## 📝 下一步

**请用户确认**:
1. 具体的复现步骤（见"问题 1"和"问题 2"）
2. 刷新后页面的实际行为
3. 是否有任何自动恢复机制

**基于用户反馈**，我们可以选择合适的解决方案。

---

**作者**: Claude Code Agent
**状态**: 待用户确认问题细节
