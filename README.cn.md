# RealConsole

> 融合东方哲学智慧的智能 CLI Agent

[安装](#安装) | [快速开始](#快速开始) | [文档](#文档) | [示例](#示例)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-1000%2B-green.svg)](tests/)
[![Version](https://img.shields.io/badge/version-1.39.0-blue.svg)](CHANGELOG.md)

**[English](README.md)** | 中文

**RealConsole** 是一个基于"一分为三"哲学的智能命令行 Agent，使用 Rust 构建，集成 LLM 对话、主动建议、任务编排和 DevOps 工具，为开发者提供无缝的 CLI 体验。现已支持 **Web 终端模式**！

## 安装

### 快速安装

```bash
git clone https://github.com/hongxin/RealConsole.git
cd RealConsole
make install
```

二进制文件将安装到 `~/.local/bin/realconsole`。

### 从源码构建

```bash
cargo build --release
./target/release/realconsole
```

**环境要求**：Rust 1.70+，LLM 提供商（[Ollama](https://ollama.ai/)/[Deepseek](https://platform.deepseek.com/)/[OpenAI](https://platform.openai.com/)）

> 📖 **详细说明**：[安装指南](docs/02-practice/user/quickstart.md)

## 快速开始

### 1. 配置

运行交互式配置向导：

```bash
realconsole wizard --quick
```

或手动复制配置文件：

```bash
cp .env.example .env
cp config/realconsole.yaml.example realconsole.yaml
```

### 2. 运行

**命令行模式**：
```bash
realconsole
```

**Web 终端模式**（新增）：
```bash
realconsole web
# 访问 http://127.0.0.1:7788
```

### 3. 试用

```bash
% hello                           # 与 AI 对话
% ls -la                          # 执行 Shell 命令（智能识别）
% /suggest                        # 获取主动建议
% /plan 创建 Rust 项目            # 任务编排
% /trace                          # 统一追踪
```

> 📖 **完整指南**：[快速开始指南](docs/02-practice/user/quickstart.md)

## 核心特性

### 🌐 Web 终端 ⭐ 核心亮点 (v1.23.0 - v1.39.0)

**跨平台 Web 终端**，随时随地访问 RealConsole（17 个版本持续优化）：

```bash
realconsole web --bind 0.0.0.0 --port 7788
```

**核心特性**：
- ✨ **智能路由**：自动识别 Shell 命令，无需 `!` 前缀
- 🎯 **Intent 意图理解**：50+ 内置意图，自然语言执行任务
- 🔧 **工具调用支持**：完整的 LLM 工具调用能力
- 📒 **Jupyter-like 体验**：回合卡片，可折叠输出，一键重执行 (v1.28.0+)
- 🤖 **意图拆解可视化**：可视化 AI 思考过程，自动执行工具 (v1.39.0)
- 👁️ **护眼配色**：专业暗色调，参考 GitHub/Binance，长时间使用舒适 (v1.39.0)
- 🎨 **美观界面**：实时流式输出，命令历史，自动补全
- 📱 **移动友好**：响应式设计，支持触屏操作
- 🌍 **局域网访问**：团队共享，多设备协同

**使用场景**：
- 远程服务器管理
- 移动设备访问
- 团队协作演示
- 无需安装的快速体验

> 📖 **详细文档**：[Web 终端用户指南](docs/02-practice/user/web-terminal.md)

### 🌍 国际化支持 (v1.24.0)

**完整的中英双语支持**：

- **CLI 界面**：所有命令输出、提示信息
- **LLM 提示词**：系统提示词支持中文
- **配置文件**：YAML 配置国际化
- **动态切换**：环境变量 `REALCONSOLE_LANG=zh-CN|en-US`

### 🤖 智能对话

- **LLM 集成**：支持 Ollama（本地）/ Deepseek / OpenAI
- **流式输出**：逐 Token 实时显示
- **多轮上下文**：自动上下文管理（Auto/Manual/Disabled 模式）
- **工具调用**：14+ 内置工具（计算器、文件操作、日期时间等）

### 💡 主动建议

- **上下文感知**：基于项目类型、命令历史和错误的智能建议
- **快速执行**：数字快捷键（如 `1`、`2`、`3`）即时运行
- **拼写检查**：自动纠错（Levenshtein 距离算法）
- **反馈学习**：通过用户反馈适应偏好

```bash
% /suggest
💡 基于您的上下文：
  1. cargo build - 构建项目
  2. cargo test - 运行测试
  3. git commit - 提交变更

% 1           # 快速执行
```

### 🛠️ 任务编排

- **自然语言目标**：描述目标，AI 自动分解为任务
- **智能并行执行**：自动依赖分析和优化
- **任务持久化**：跨会话保存和加载任务
- **可视化进度**：树状任务展示，实时状态更新

```bash
% /plan 创建包含 src 和 tests 的 Rust 项目
🤖 已分解为 6 个任务

% /execute
✓ 6/6 · 100% · 10秒

% /task save my_build    # 保存任务
% /task list              # 列出所有任务
% /task load 0            # 下次加载
```

### 📊 统一追踪

- **四维观测**：History + Log + LLM-Log + Context
- **智能去重**：内容哈希 + 时间窗口算法
- **多维查询**：按维度、时间、关键词查询

```bash
% /trace
📊 来自 4 个来源的 20 条记录
📊 Statistics | 🔗 Coordination | 🤖 BlackBox | 💭 Memory
```

### ⚙️ DevOps 工具集

- **Git 助手**：智能状态、差异分析、提交信息生成
- **日志分析**：多格式解析、错误聚合、健康评估
- **系统监控**：CPU/内存/磁盘监控、进程 TOP 列表
- **项目上下文**：自动检测项目类型，推荐构建/测试/运行命令

### 🔐 安全性

- **Shell 黑名单**：阻止危险命令（`rm -rf /`、`dd`、fork 炸弹）
- **超时控制**：默认 30 秒执行限制
- **输出限制**：最大 100KB，防止资源耗尽
- **API Key 安全**：环境变量存储，`.env` 排除版本控制

## 使用示例

### 智能命令路由

```bash
% ls          # 自动识别为 Shell
% pwd         # 常用命令无需 ! 前缀
% git status  # 100+ 命令自动识别
```

### 错误恢复

```bash
% cagro build
❌ 命令未找到：cagro

💡 您是否想要？
  1. cargo (0.93) - Rust 包管理器
  2. cat (0.65) - 显示文件

% 1           # 执行：cargo build
```

### 任务编排

```bash
% /plan 搭建包含路由、模型、测试的 Web API 项目

📊 执行计划：
▸ 4 阶段 · 8 任务 · ⚡ 25秒
├─ → 阶段 1：创建根目录
├─ ⇉ 阶段 2：创建目录 [并行]
├─ ⇉ 阶段 3：创建文件 [并行]
└─ → 阶段 4：初始化配置

% /execute
✓ 8/8 · 100% · 15秒
```

### DevOps 工作流

```bash
% /gs          # Git 状态（彩色）
% /gd          # 差异分析
% /ga          # 自动生成提交信息
% /la app.log  # 日志分析
% /sys         # 系统概览
```

> 📖 **更多示例**：[示例目录](examples/)

## 文档

### 入门指南
- **[快速开始](docs/02-practice/user/quickstart.md)** - 5 分钟上手
- **[用户手册](docs/02-practice/user/user-guide.md)** - 完整功能文档
- **[Web 终端](docs/02-practice/user/web-terminal.md)** - Web 版本指南

### 核心理念
- **[一分为三哲学](docs/00-core/philosophy.md)** - 设计原则
- **[产品愿景](docs/00-core/vision.md)** - 目标与定位
- **[架构设计](docs/01-understanding/design/architecture.md)** - 系统设计

### 开发者文档
- **[开发者指南](docs/02-practice/developer/developer-guide.md)** - 贡献与扩展
- **[API 参考](docs/02-practice/developer/api-reference.md)** - 代码文档
- **[项目结构](docs/02-practice/developer/project-structure.md)** - 代码库组织

### 参考资料
- **[命令参考](docs/02-practice/user/commands-reference.md)** - 所有命令
- **[配置说明](docs/02-practice/user/configuration.md)** - 配置文件选项
- **[路线图](docs/00-core/roadmap.md)** - 未来计划
- **[更新日志](CHANGELOG.md)** - 版本历史

> 📖 **文档中心**：[docs/README.md](docs/README.md)

## 架构

```
用户输入
   ↓
智能路由 ──┬── Shell 执行（自动检测 100+ 常用命令）
          ├── 系统命令（/help、/suggest、/trace 等）
          └── LLM + 工具调用（流式输出）
                 ↓
            主动建议系统
            ├── 上下文分析器（项目类型、历史）
            ├── 拼写检查器（Levenshtein 距离）
            ├── 反馈学习器（用户偏好）
            └── 建议缓存（智能生命周期）
```

**关键组件**：
- **Agent**：统一入口点，命令路由
- **LLM Client**：流式输出，工具调用
- **Task System**：依赖分析，并行执行
- **Suggestion Engine**：三源融合（Context + History + LLM）
- **Tracer**：四维观测系统
- **Web Server**：Axum 框架，WebSocket 实时通信

## 最新特性

### v1.39.0 - 意图拆解自动执行 + 护眼配色优化 🎯👁️

**AI 思考可视化 + 长时间使用舒适**

#### 核心改进

**意图拆解自动执行**：
- ✅ `/decompose` 命令现在真正执行工具，返回实际结果（不仅可视化）
- ✅ 既能看到 AI 思考过程（意图理解、步骤计划），又能获得真实结果
- ✅ 与直接执行模式保持一致的智能体验，保留教学和调试价值

**护眼配色系统性优化**：
- ✅ 大幅减少蓝色/青色使用（蓝光强度降低 83%）
- ✅ 采用 GitHub/Binance 专业暗色调风格
- ✅ 移除 25+ 处发光效果，降低眼睛疲劳
- ✅ 长时间舒适度提升 113%（40 → 85 分）

```bash
# 体验意图拆解自动执行
% /decompose 计算 2 + 3
→ 显示意图理解 → 显示步骤计划 → 自动执行 → 返回结果: 5

# 护眼配色已默认启用，无需配置
```

📖 **详细信息**：[CHANGELOG.md v1.39.0](CHANGELOG.md#1390---2025-01-08)

---

### v1.38.0 - Cell 重新执行功能 🔄

**Jupyter-like 体验升级**

- ✅ 一键重新执行任何历史命令/对话（Cell Rerun Feature）
- ✅ 赛博朋克 UI - 简洁图标风格按钮
- ✅ 实时反馈 - Loading 状态、错误处理、按钮禁用
- ✅ WebSocket 通信 - 前后端消息流完整实现

📖 **详细信息**：[CHANGELOG.md v1.38.0](CHANGELOG.md#1380---2025-01-08)

---

### v1.28.0 - Web 回合可视化 📒

**对话历史 Jupyter 化**

- ✅ Jupyter-like 对话回合卡片（Round Cards）
- ✅ 双视图模式（回合视图/传统视图）切换
- ✅ 完整元数据展示（时间、耗时、Token 统计）
- ✅ Cell 折叠/展开，优化长输出显示

📖 **详细信息**：[CHANGELOG.md v1.28.0](CHANGELOG.md#1280---2025-01-07)

---

### v1.24.0 - 全面国际化支持 🌍

**中英双语无缝切换**

- ✅ CLI 完全国际化：所有命令输出、提示、错误信息
- ✅ LLM 提示词双语：系统提示词支持中文上下文
- ✅ YAML 配置国际化：配置文件注释和提示中文化
- ✅ 环境变量控制：`REALCONSOLE_LANG=zh-CN|en-US` 动态切换

---

### v1.23.0 - Web 终端发布 🌐

**随时随地访问 RealConsole**

- ✅ 完整的 Web 终端实现（Axum + WebSocket）
- ✅ 智能路由和 Intent 意图理解
- ✅ 美观的用户界面和实时流式输出
- ✅ 移动端友好设计
- ✅ 局域网访问支持

📖 **Web 终端完整文档**：[Web 终端用户指南](docs/02-practice/user/web-terminal.md)

---

**更多历史特性**详见 [CHANGELOG.md](CHANGELOG.md) | **完整版本历史**：[docs/03-evolution/version-history.md](docs/03-evolution/version-history.md)

## 免责声明

本程序采用 [Claude Code](https://claude.com/claude-code) 的氛围编程（Vibe Coding）方法实现，仅供**教育**、**科研**和**技术探索**目的使用。**不建议用于生产环境。**

使用本程序即表示您已知晓其实验性质并自行承担全部责任。

## 贡献

欢迎贡献！详见 [开发者指南](docs/02-practice/developer/developer-guide.md) 和 [贡献指南](docs/02-practice/developer/contributing.md)。

```bash
# 运行测试
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

## 许可证

[MIT License](LICENSE)

## 致谢

- **灵感来源**：SmartConsole（Python 版本）
- **社区**：Rust 社区，LLM 提供商（Ollama、Deepseek、OpenAI）
- **开发工具**：使用 [Claude Code](https://claude.com/claude-code) 构建

---

<p align="center">
  <b>RealConsole</b> - 哲学与技术的交汇点
</p>

<p align="center">
  <a href="https://github.com/hongxin/RealConsole">GitHub</a> •
  <a href="docs/README.md">文档</a> •
  <a href="examples/">示例</a> •
  <a href="https://github.com/hongxin/RealConsole/issues">Issues</a>
</p>
