# RealConsole

> 融合东方哲学智慧的智能 CLI Agent

[安装](#安装) | [快速开始](#快速开始) | [文档](#文档) | [示例](#示例)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-1400%2B-green.svg)](tests/)
[![Version](https://img.shields.io/badge/version-1.67.0-blue.svg)](CHANGELOG.md)

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

**环境要求**：Rust 1.70+，LLM 提供商（[Ollama](https://ollama.ai/)/[Deepseek](https://platform.deepseek.com/)/[Gemini](https://aistudio.google.com/)/[OpenAI](https://platform.openai.com/)）

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

### 📊 数据可视化系统 ⭐ 核心亮点 (v1.44.0 - v1.49.0)

**在终端中生成专业图表**，基于 ECharts 5 的强大可视化能力，融合**易经、素书、极简主义**智慧：

```bash
# 基础图表：折线图、柱状图、饼图
!chart line --title "月度销售" --x-labels "1月,2月,3月" --series "销售额:120,132,145"
!chart pie --title "市场份额" --labels "产品A,产品B,产品C" --series "份额:35,25,40"

# 进阶图表：散点图、面积图、气泡图
!chart scatter --title "身高体重" --x-name "身高" --y-name "体重" --series "数据:(170,65),(175,70)"
!chart bubble --title "产品分析" --series "数据:(100,500,5000):(150,400,6000)"

# 高级图表：雷达图、热力图
!chart radar --title "技能评估" --indicators "编程,设计,沟通" --series "员工A:90,85,80"
!chart heatmap --title "用户活跃度" --x-labels "周一,周二" --y-labels "上午,下午"

# CSV 数据源
!chart csv data.csv --type line --x-col "月份" --y-col "销售额"
```

**核心能力**：
- 📈 **8 种图表类型**：折线、柱状、饼图、散点、面积、气泡、雷达、热力
- 💾 **3 种导出格式**：CSV数据、PNG高清图、SVG矢量图
- 📁 **CSV 文件支持**：直接从 CSV 生成图表，支持多列多系列
- 🎨 **主题自适应**：深色/浅色主题，紫绿金三色体系
- 🖱️ **交互式操作**：悬停高亮、图例切换、缩放、导出下拉菜单
- 📱 **响应式设计**：自动适配窗口大小，移动端友好

**使用场景**：
- 📊 数据分析和趋势可视化
- 📝 项目报告和演示
- 📈 日志和监控数据展示
- 🔍 快速数据探索

> 📖 **完全教程**：[数据可视化完全教程](docs/02-practice/user/visualization-tutorial.md) - 融合易经、素书智慧的系统教程

### 🌍 国际化支持 (v1.24.0)

**完整的中英双语支持**：

- **CLI 界面**：所有命令输出、提示信息
- **LLM 提示词**：系统提示词支持中文
- **配置文件**：YAML 配置国际化
- **动态切换**：环境变量 `REALCONSOLE_LANG=zh-CN|en-US`

### 🤖 智能对话

- **LLM 集成**：支持 Ollama（本地）/ Deepseek / Gemini / OpenAI
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

### v1.45.0 - 可视化 Phase 2：饼图、散点图和 CSV 文件 📊

**数据可视化能力全面升级**

- ✅ **饼图**：扇区标签、百分比显示、悬停高亮
- ✅ **散点图**：单/多系列、坐标轴命名、悬停放大
- ✅ **CSV 文件**：直接从 CSV 生成图表，支持多列多系列
- ✅ **图表集成**：嵌入回合卡片，支持折叠/展开
- ✅ **视觉优化**：流畅过渡动画、响应式布局

```bash
# 饼图：市场份额分析
!chart pie --title "市场份额" --labels "A,B,C" --series "份额:35,25,40"

# 散点图：相关性分析
!chart scatter --title "身高体重" --data "170,65 175,70 180,80"

# CSV 图表：多系列折线图
!chart csv sales.csv --type line --x-col "月份" --y-col "销售额" --y-col "成本"
```

📖 **详细信息**：[CHANGELOG.md v1.45.0](CHANGELOG.md#1450---2025-01-22)

---

### v1.44.0 - 可视化 MVP：折线图和柱状图 📈

**终端数据可视化首次亮相**

- ✅ 基于 ECharts 5 的专业图表渲染
- ✅ 折线图和柱状图支持
- ✅ 主题自适应（深色/浅色）
- ✅ 交互式工具栏（缩放、保存图片）

📖 **详细信息**：[CHANGELOG.md v1.44.0](CHANGELOG.md#1440---2025-01-21)

---

### v1.40.0 - Web 会话持久化 💾

**无缝的浏览器体验**

- ✅ 自动保存/恢复会话（页面刷新后无缝恢复）
- ✅ 历史会话管理（浏览、加载、删除）
- ✅ 智能命名（基于首条输入自动生成）
- ✅ 定期备份（每 5 分钟自动保存）

📖 **详细信息**：[CHANGELOG.md v1.40.0](CHANGELOG.md#1400---2025-11-16)

---

**更多历史版本**详见 [CHANGELOG.md](CHANGELOG.md) | **完整版本历史**：[docs/03-evolution/version-history.md](docs/03-evolution/version-history.md)

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
- **社区**：Rust 社区，LLM 提供商（Ollama、Deepseek、Gemini、OpenAI）
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
