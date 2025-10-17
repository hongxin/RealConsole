# RealConsole 用户指南

欢迎使用 RealConsole！这里是完整的用户指南和开发文档索引。

---

## 📖 快速开始

### 新用户必读

1. **[快速入门 (QUICKSTART.md)](./QUICKSTART.md)**
   - ⏱️ 5分钟快速上手
   - 安装步骤
   - 基本使用
   - 常用命令

2. **[LLM 配置指南 (LLM_SETUP_GUIDE.md)](./LLM_SETUP_GUIDE.md)**
   - 配置 Deepseek API
   - 配置本地 Ollama
   - 选择合适的模型
   - 常见问题解决

3. **[环境配置 (ENV_FILE_GUIDE.md)](./ENV_FILE_GUIDE.md)**
   - .env 文件配置
   - 环境变量说明
   - 安全最佳实践

---

## 🎯 核心功能指南

### Tool Calling（工具调用）

**适用人群**：所有用户

**核心特性**：
- 14+ 内置工具（文件操作、计算器、日期时间等）
- 自然语言交互
- 自动参数提取

**文档**：
- **[用户指南 (TOOL_CALLING_USER_GUIDE.md)](./TOOL_CALLING_USER_GUIDE.md)**
  - 如何使用工具
  - 实际案例
  - 最佳实践
  - 常见问题

- **[开发指南 (TOOL_CALLING_DEVELOPER_GUIDE.md)](./TOOL_CALLING_DEVELOPER_GUIDE.md)**
  - 如何创建自定义工具
  - Tool Schema 规范
  - 参数验证
  - 测试方法

### Intent DSL（意图识别）

**适用人群**：高级用户、开发者

**核心特性**：
- 50+ 内置意图模板
- 智能参数提取
- 模糊匹配
- 可扩展架构

**文档**：
- **[Intent DSL 指南 (INTENT_DSL_GUIDE.md)](./INTENT_DSL_GUIDE.md)**
  - Intent DSL 概念
  - 内置意图列表
  - 创建自定义意图
  - 实体提取
  - 模糊匹配配置

---

## 📚 进阶主题

### 开发者资源

- **[API 文档](../../target/doc/realconsole/index.html)** (运行 `cargo doc --open` 生成)
- **[架构设计](../design/)** - 系统设计文档
- **[实现记录](../implementation/)** - 技术实现细节

### 使用案例

- **[10个典型案例](../use-cases/10-cases.md)**
- **[20个进阶案例](../use-cases/20-cases.md)**
- **[30个复杂案例](../use-cases/30-cases.md)**
- **[50个综合案例](../use-cases/50-cases.md)**
- **[精选案例](../use-cases/selected-cases.md)**

---

## 🎓 学习路径

### 新手路径（0-7天）

| 天数 | 学习内容 | 文档 |
|------|---------|------|
| Day 1 | 安装和基本使用 | QUICKSTART.md |
| Day 2 | LLM 配置 | LLM_SETUP_GUIDE.md |
| Day 3 | 环境变量配置 | ENV_FILE_GUIDE.md |
| Day 4-5 | Tool Calling 实践 | TOOL_CALLING_USER_GUIDE.md |
| Day 6-7 | 高级功能探索 | 各类案例文档 |

### 进阶路径（1-2周）

| 周数 | 学习内容 | 文档 |
|------|---------|------|
| Week 1 | Intent DSL 学习 | INTENT_DSL_GUIDE.md |
| Week 2 | 自定义工具开发 | TOOL_CALLING_DEVELOPER_GUIDE.md |

### 高级路径（1个月+）

- 深入理解架构设计
- 阅读源代码
- 贡献代码或文档
- 创建复杂的工作流

---

## 🔗 快速链接

### 常用命令速查

```bash
# 查看帮助
/help

# 列出所有工具
/tools list

# 列出所有意图
/intent list

# 查看工具详情
/tools info <tool_name>

# Lazy Mode（自然语言）
/lazy <你的自然语言请求>

# 执行 shell 命令
/shell <command>

# 查看历史
/history

# LLM 诊断
/llm diagnose
```

### 环境变量速查

```bash
# Deepseek API
DEEPSEEK_API_KEY=sk-...
DEEPSEEK_MODEL=deepseek-chat

# Ollama 本地
OLLAMA_ENDPOINT=http://localhost:11434
OLLAMA_MODEL=qwen3:4b
```

---

## ❓ 获取帮助

### 文档问题

- 在文档中搜索关键词
- 查看相关案例
- 阅读常见问题

### 技术支持

- GitHub Issues: [https://github.com/hongxin/realconsole/issues](https://github.com/hongxin/realconsole/issues)
- 项目主页: [https://github.com/hongxin/realconsole](https://github.com/hongxin/realconsole)

### 贡献指南

欢迎贡献！请参考：
- 代码贡献: [CONTRIBUTING.md](../../CONTRIBUTING.md) (待创建)
- 文档贡献: 直接提交 PR

---

## 📈 文档更新记录

| 日期 | 更新内容 |
|------|---------|
| 2025-10-15 | 创建用户指南索引，整理文档结构 |
| 2025-10 | 完善各类用户指南 |

---

**最后更新**: 2025-10-15
**维护者**: RealConsole Team
**项目地址**: https://github.com/hongxin/realconsole
