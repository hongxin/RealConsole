feat: v1.28.0 - 对话回合可视化（Conversation Round Visualization）

## 核心功能

### 1. Jupyter-like 对话回合卡片
- 每个对话轮次显示为独立的回合卡片
- 包含元数据：状态、执行时间、工具使用、模型名称
- 支持折叠/展开历史回合
- 赛博朋克主题风格

### 2. 双视图模式切换
- 📊 回合模式（默认）：结构化卡片显示
- 📜 传统模式：流式输出
- 右上角切换按钮，即时切换
- 零数据丢失，历史完整保留

### 3. 执行反馈优化
- 回合模式：飞轮动画 + 模型名称 + "思考中..."
- 传统模式：保留原有飞轮动画
- 状态图标：⏸ Pending、⏳ Running、✓ Success、✗ Error

## 技术实现

### 后端 (+237 行)
- 新增 `ConversationRound` 数据结构（9 个字段）
- 新增 `RoundStatus` enum（4 种状态）
- 扩展 WebSocket 协议（4 种新消息类型）
- Session 回合管理（8 个新方法）
- 执行时间追踪（`Instant::now()`）
- 工具使用提取（从 `__DEBUG__` 解析）

### 前端 (+651 行)
- 8 个核心 JavaScript 回合管理方法
- 280 行赛博朋克主题 CSS
- 视图模式切换机制
- 飞轮动画 CSS 动画
- 系统消息标志（欢迎消息等）

## Bug 修复

1. **状态图标显示 `?`**
   - 原因：Rust enum 序列化格式不一致
   - 修复：添加 `normalizeStatus()` 方法

2. **输入输出格式错误**
   - 原因：HTML 结构与 CSS 不匹配
   - 修复：更新 HTML 使用正确的 class 名称

3. **传统模式重复输出**
   - 原因：`createRound()` 创建元素默认可见
   - 修复：根据视图模式设置初始显示状态

4. **欢迎消息不显示**
   - 原因：回合模式下 `writePlainText()` 被跳过
   - 修复：添加系统消息标志参数

## 代码统计

- **代码变更**: 4 个文件，+899 行，-11 行
- **文档新增**: 6 个文档，2,500+ 行
  - v1.28.0-implementation-plan.md (500 行)
  - v1.28.0-testing-guide.md (450 行)
  - v1.28.0-bugfix-report.md (420 行)
  - v1.28.0-view-mode-toggle.md (380 行)
  - v1.28.0-ux-improvements.md (350 行)
  - v1.28.0-release-notes.md (400 行)

## 影响范围

- ✅ Web 终端：新增回合可视化功能
- ✅ CLI 版本：无影响
- ✅ 配置：无需变更
- ✅ 向后兼容：完全兼容

## 测试验证

- ✅ 回合模式：卡片显示正确
- ✅ 传统模式：无重复输出
- ✅ 模式切换：流畅无缝
- ✅ 飞轮动画：正常显示
- ✅ 状态图标：正确匹配
- ✅ 元数据：完整显示

## 里程碑意义

这是 RealConsole v1 → v2 过渡计划的第一个版本，为未来的 Cell 执行模型
和 Notebook 体验奠定了基础。

下一步：v1.29.0 - 回合操作增强（删除、重执行、导出、快捷键）

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
