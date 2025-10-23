# 示例代码

本目录包含 RealConsole 各个功能模块的使用示例代码，帮助开发者快速理解和使用 RealConsole 的核心功能。

---

## 📋 示例索引

### Rust 示例程序

#### 对话与上下文管理
- **context_manager_demo.rs** - ContextManager 使用示例
  - 对话上下文管理
  - 消息历史追踪
  - 上下文窗口控制
  - Token 计数与管理
  - **运行**: `cargo run --example context_manager_demo`

- **conversation_demo.rs** - Conversation 系统使用示例
  - 对话流程管理
  - 消息发送与接收
  - 对话状态维护
  - **运行**: `cargo run --example conversation_demo`

#### UI 组件
- **spinner_demo.rs** - Spinner 加载动画示例
  - 异步操作进度指示
  - 自定义 Spinner 样式
  - 多种动画效果
  - **运行**: `cargo run --example spinner_demo`

#### 交互界面
- **wizard_demo.rs** - 配置向导示例
  - 交互式配置引导
  - 用户输入验证
  - 配置文件生成
  - **运行**: `cargo run --example wizard_demo`

### Markdown 文档

#### 任务系统
- **task_system_usage.md** - 任务系统使用指南
  - 任务创建与管理
  - 任务编排流程
  - 任务状态追踪
  - 最佳实践

- **task_visualization.md** - 任务可视化文档
  - 任务流程图
  - 状态转换图
  - 可视化工具
  - 图表说明

---

## 🚀 运行示例

### 运行单个示例
```bash
# ContextManager 示例
cargo run --example context_manager_demo

# Conversation 示例
cargo run --example conversation_demo

# Spinner 动画示例
cargo run --example spinner_demo

# 配置向导示例
cargo run --example wizard_demo
```

### 运行所有示例
```bash
for example in context_manager_demo conversation_demo spinner_demo wizard_demo; do
    echo "=== Running $example ==="
    cargo run --example $example
    echo ""
done
```

---

## 📝 示例说明

### 1. context_manager_demo.rs

**功能演示**：
- 创建 ContextManager 实例
- 添加用户消息和助手响应
- 自动管理上下文窗口大小
- Token 计数和优化
- 上下文序列化与持久化

**核心代码示例**：
```rust
use realconsole::conversation::context_manager::ContextManager;
use realconsole::config::ConversationConfig;

// 创建 ContextManager
let config = ConversationConfig::default();
let mut manager = ContextManager::new(config);

// 添加消息
manager.add_user_message("Hello!".to_string());
manager.add_assistant_message("Hi there!".to_string());

// 获取上下文
let context = manager.build_context();
```

### 2. conversation_demo.rs

**功能演示**：
- 完整的对话流程
- 消息发送与接收
- 上下文状态管理
- 对话历史查询

**适用场景**：
- 理解对话系统工作原理
- 学习如何集成对话功能
- 测试对话流程

### 3. spinner_demo.rs

**功能演示**：
- 多种 Spinner 动画效果
- 异步操作进度提示
- 自定义 Spinner 文本
- 完成/失败状态显示

**动画效果**：
```
⠋ Loading...
⠙ Loading...
⠹ Loading...
⠸ Loading...
⠼ Loading...
⠴ Loading...
⠦ Loading...
⠧ Loading...
```

### 4. wizard_demo.rs

**功能演示**：
- 交互式配置引导
- 步骤式配置流程
- 输入验证与反馈
- 配置文件生成

**配置流程**：
1. 选择 LLM Provider
2. 输入 API Key
3. 配置其他选项
4. 保存配置文件

---

## 🎯 学习路径

### 初学者
1. 先阅读 Markdown 文档了解概念
2. 运行 `wizard_demo.rs` 了解交互流程
3. 运行 `spinner_demo.rs` 学习 UI 组件

### 中级开发者
1. 深入学习 `conversation_demo.rs` 理解对话系统
2. 研究 `context_manager_demo.rs` 掌握上下文管理
3. 参考示例代码开发自己的功能

### 高级开发者
1. 阅读示例源代码理解实现细节
2. 修改示例代码测试不同场景
3. 贡献新的示例到项目

---

## 📚 相关文档

- [用户指南](../docs/02-practice/user/user-guide.md)
- [开发者指南](../docs/02-practice/developer/developer-guide.md)
- [测试文档](../tests/README.md)
- [测试脚本](../scripts/test/README.md)

---

## 🔧 添加新示例

### 创建 Rust 示例

1. 在 `examples/` 目录创建 `<feature>_demo.rs`
2. 编写示例代码
3. 在 `Cargo.toml` 中注册示例（通常自动识别）
4. 更新本 README.md
5. 测试运行：`cargo run --example <feature>_demo`

### 创建 Markdown 文档

1. 在 `examples/` 目录创建 `<feature>_<topic>.md`
2. 按照统一格式编写文档
3. 添加图表和示例代码
4. 更新本 README.md

---

## 💡 最佳实践

### 示例代码原则
- **简洁性**: 代码应该简洁易懂
- **完整性**: 示例应该可以独立运行
- **注释性**: 关键代码应该有清晰注释
- **实用性**: 示例应该展示实际使用场景

### 文档编写原则
- **清晰性**: 文档结构清晰，层次分明
- **完整性**: 包含背景、示例、说明
- **实用性**: 提供实际可用的代码片段
- **更新性**: 保持文档与代码同步

---

## 🐛 问题反馈

示例运行出错时：

1. 确认 Rust 版本（需要 1.90+）
2. 更新依赖：`cargo update`
3. 清理重建：`cargo clean && cargo build --examples`
4. 查看完整输出：`RUST_LOG=debug cargo run --example <name>`

如果问题持续存在，请提交 Issue。

---

**最后更新**: 2025-10-23
**维护者**: RealConsole Contributors
