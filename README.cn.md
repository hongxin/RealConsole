# RealConsole

> 融合东方哲学智慧的智能 CLI Agent

[安装](#安装) | [快速开始](#快速开始) | [文档](#文档) | [示例](#示例)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-1000%2B-green.svg)](tests/)
[![Version](https://img.shields.io/badge/version-1.9.3-blue.svg)](CHANGELOG.md)

**[English](README.md)** | 中文

**RealConsole** 是一个基于"一分为三"哲学的智能命令行 Agent，使用 Rust 构建，集成 LLM 对话、主动建议、任务编排和 DevOps 工具，为开发者提供无缝的 CLI 体验。

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

> 📖 **详细说明**：[安装指南](docs/QUICKSTART.md#installation)

### 1. 配置

运行交互式配置向导：

```bash
realconsole wizard --quick
```

或手动复制配置文件：

```bash
cp .env.example .env
cp config/realconsole.yaml.example realconsole.yaml
# 编辑 .env 和 realconsole.yaml
```

### 2. 运行

```bash
realconsole
```

### 3. 试用

```bash
% hello                           # 与 AI 对话
% !ls -la                         # 执行 Shell 命令
% /suggest                        # 获取主动建议 ⭐
% /plan 创建 Rust 项目            # 任务编排
% /trace                          # 统一追踪
```

> 📖 **完整指南**：[快速开始指南](docs/QUICKSTART.md)

## 核心特性

### 🤖 智能对话
- **LLM 集成**：支持 Ollama（本地）/ Deepseek / OpenAI
- **流式输出**：逐 Token 实时显示
- **多轮上下文**：自动上下文管理（Auto/Manual/Disabled 模式）
- **工具调用**：14+ 内置工具（计算器、文件操作、日期时间等）

### 💡 主动建议 ⭐ 新增 (v1.8.0)
- **上下文感知**：基于项目类型、命令历史和错误的智能建议
- **快速执行**：数字快捷键（如 `1`、`2`、`3`）即时运行建议
- **拼写检查**：基于 Levenshtein 距离算法的自动纠错
- **反馈学习**：通过用户反馈适应偏好（接受率 + 位置权重）
- **智能缓存**：最近建议缓存，2.5-5 分钟生命周期

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
- **可视化进度**：树状任务展示，实时状态更新

```bash
% /plan 创建包含 src 和 tests 的 Rust 项目
🤖 已分解为 6 个任务

% /execute
✓ 6/6 · 100% · 10秒
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

### 智能命令路由

```bash
% ls          # 自动识别为 Shell
% pwd         # 常用命令无需 ! 前缀
% git status  # 80+ 命令自动识别
```

### 错误恢复

```bash
% !cagro build
❌ 命令未找到：cagro

💡 您是否想要？
  1. cargo (0.93) - Rust 包管理器
  2. cat (0.65) - 显示文件

% 1           # 执行：cargo build
```

### 反馈学习

```bash
# RealConsole 从您的选择中学习
% /suggest
1. cargo check (0.85) - 快速语法检查
2. cargo build (0.80) - 完整构建

% 1  # 选择 cargo check

# 多次使用后
% /suggest
1. cargo check (0.92) ⬆️ # 得到提升！
2. cargo build (0.80)
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
- **[快速开始](docs/QUICKSTART.md)** - 5 分钟上手指南
- **[用户手册](docs/02-practice/user/user-guide.md)** - 完整功能文档
- **[常见问题](docs/02-practice/user/faq.md)** - 常见问题解答

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
智能路由 ──┬── Shell 执行（! 前缀或自动检测）
          ├── 系统命令（/help、/suggest、/trace 等）
          └── LLM + 工具调用（流式输出）
                 ↓
            主动建议系统 ⭐
            ├── 上下文分析器（项目类型、历史）
            ├── 拼写检查器（Levenshtein 距离）
            ├── 反馈学习器（用户偏好）
            └── 建议缓存（2.5-5 分钟生命周期）
```

**关键组件**：
- **Agent**：统一入口点，命令路由
- **LLM Client**：流式输出，工具调用
- **Task System**：依赖分析，并行执行
- **Suggestion Engine**：三源融合（Context + History + LLM）
- **Tracer**：四维观测系统

## v1.9.0 新特性 ⭐

### 两仪演化系统 - 体用合一

**完整实现"先天八卦·竖看"哲学** - 时间维度的状态演化系统：

#### 核心组件

1. **太极·两仪·四象 体系**
   - 太极（Taiji）：阴阳能量连续模型（0.0-1.0）
   - 两仪（Liangyyi）：太阴☽ / 太阳☉ 二元状态
   - 四象（Sixiang）：老阴/少阳/少阴/老阳 四态循环

2. **状态追踪器（StateTracker）**
   - 实时追踪系统状态演化
   - 维护状态历史（最近 100 个快照）
   - 智能活动水平计算
   - 趋势分析（趋向阴/趋向阳/稳定）

3. **体用合一集成**
   - 自动状态更新：用户操作 → 事件分类 → 状态演化
   - 八卦连接：状态快照 → 艮☶维度，状态趋势 → 巽☴维度
   - 状态感知建议：建议系统获得时间维度感知

#### 哲学实现

```
体（Liangyyi）        用（Bagua）
     ↓                    ↓
  时间演化            空间存储
     ↓                    ↓
StateTracker  ←──→  BaguaPalace
     ↓                    ↓
  当前状态   ──→   艮/巽维度
     ↓
SuggestionEngine
```

**代码统计**：1152 行，24 个测试，100% 通过

📖 **详细信息**：[两仪系统报告](docs/04-reports/) | [设计文档](docs/01-understanding/design/liangyyi-state-evolution-design.md)

---

## v1.8.0 特性

### 主动建议系统

**Phase 4.2 完成** - 三大功能：

1. **P0 - 快速执行与增强错误分析**
   - 数字快捷键即时执行建议
   - 11 种错误模式（命令未找到、权限拒绝等）

2. **P1 - 拼写检查与建议缓存**
   - Levenshtein 距离算法，100+ 命令字典
   - 三状态缓存生命周期（Fresh/Stale/Expired）

3. **P2.1 - 反馈学习系统**
   - 三状态反馈（Accepted/Skipped/Rejected）
   - 质量评分 = 70% 接受率 + 30% 位置权重
   - 自动分数调整（0.5x-1.5x 范围）

📖 **详细信息**：[CHANGELOG.md](CHANGELOG.md) | [完成报告](docs/04-reports/)

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
