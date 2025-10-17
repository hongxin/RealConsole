# RealConsole (Rust)

> **中文 | [English](README.en.md)**

程序员和运维工程师都非常喜欢用的智能 CLI Agent - 基于 Rust 的高性能实现

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-645%2B%20passed-green.svg)](tests/)
[![Coverage](https://img.shields.io/badge/coverage-78%2B%25-yellow.svg)](docs/test_reports/)
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](docs/CHANGELOG.md)

## ⚠️ 免责声明

**重要提示**：本程序主要采用 [Claude Code](https://claude.com/claude-code) 的氛围编程（Vibe Coding）方法实现，这是一种探索性的开发模式，因此我们无法保证程序的安全性和稳定性。

**使用目的**：
- 本程序仅供**教育**、**科研**和**技术探索**目的使用
- 不建议在生产环境中使用

**责任声明**：
使用、编译或运行本程序即表示您已充分知晓其实验性质和潜在风险。因使用本程序而导致的任何问题、损失或损害，均由用户自行承担全部责任。本程序的开发者、贡献者和维护者对此概不负责。

**建议**：
- 在测试环境中谨慎使用
- 定期备份重要数据
- 理解每个命令的作用后再执行

---

## ✨ 核心特性

### 🧠 智能 AI 能力
- **LLM 驱动的智能对话** - 支持 Ollama/Deepseek，实时流式输出，自然语言交互
- **任务编排系统** ⭐ NEW - LLM智能分解复杂目标，自动依赖分析和并行优化执行（`/plan`, `/execute`）
- **智能 Pipeline 生成** - 自动理解用户意图，将自然语言转换为文件操作命令
- **工具自动调用** - 14+ 内置工具（计算器、文件操作、时间查询等），智能并行执行
- **意图识别** - 50+ 内置意图模板，自动理解用户需求并执行
- **多层 Fallback 机制** - 4层保障（LLM生成→规则匹配→模板匹配→对话），确保系统永不失败

**使用示例**:
```bash
» 显示最大的3个rs文件
🤖 LLM 生成
→ 执行: find . -name '*.rs' -type f -exec ls -lh {} + | sort -k5 -hr | head -n 3

» 帮我计算 2 的 10 次方
[LLM 自动调用 calculator 工具]
根据计算结果，2^10 = 1024

» /plan 创建一个Rust项目，包含src目录和测试目录，然后创建main.rs
🤖 智能任务分解
▸ 3 阶段 · 4 任务 · ⚡ 15秒 (节省 5秒)
├─ → Stage 1 (5s)
│  └─ 创建项目根目录 $ mkdir -p myproject
├─ ⇉ Stage 2 (5s)  [并行执行]
│  ├─ 创建src目录 $ mkdir -p myproject/src
│  └─ 创建tests目录 $ mkdir -p myproject/tests
└─ → Stage 3 (5s)
   └─ 创建main.rs文件 $ touch myproject/src/main.rs

» /execute
✓ 4/4 · 100% · 12秒
```

### 🛠️ DevOps 工具集
- **项目上下文感知** - 自动识别项目类型（Rust/Python/Node/Go/Java），智能推荐构建/测试/运行命令（`/project`）
- **Git 智能助手** - 状态查看、变更分析、自动生成提交消息（遵循 Conventional Commits）（`/gs`, `/gd`, `/ga`, `/gb`）
- **日志分析工具** - 多格式解析、错误聚合、健康度评估（`/la`, `/le`, `/lt`）
- **Shell 安全执行** - 通过 `!` 前缀执行命令，黑名单保护，超时控制

### 💻 系统监控
- **系统资源监控** - CPU/内存/磁盘实时监控，进程 TOP 列表（`/sys`, `/cpu`, `/disk`, `/top`）
- **跨平台支持** - macOS + Linux 完整支持，零额外依赖
- **执行日志** - 完整的操作记录与审计

### 🎨 友好体验
- **配置向导** - 5 分钟快速完成初始化（`realconsole wizard --quick`）
- **多层次帮助** - Quick/All/Topic 帮助系统，示例库，快速参考卡片（`/help`, `/examples`, `/quickref`）
- **智能错误提示** - 30+ 错误代码，详细修复建议，源错误追踪
- **记忆系统** - 短期+长期记忆，支持搜索和导出
- **懒人模式** - 直接输入即对话，无需命令前缀

## 🚀 快速开始

### 1. 安装 Rust

```bash
# 安装 Rust（如果未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. 构建项目

```bash
# 克隆仓库
git clone https://github.com/your-repo/realconsole.git
cd realconsole

# 构建 release 版本
cargo build --release
```

### 3. 配置向导（推荐新用户）🧙

**快速模式**（5 分钟完成）：

```bash
./target/release/realconsole wizard --quick
```

向导会引导你完成：
- ✅ LLM 提供商选择（Deepseek API / Ollama 本地）
- ✅ API Key 配置（如使用 Deepseek）
- ✅ 基础功能设置（Shell 执行、记忆系统等）
- ✅ 自动生成 `realconsole.yaml` 和 `.env` 文件

**完整模式**（更多选项）：

```bash
./target/release/realconsole wizard
```

### 4. 运行 RealConsole

```bash
# 使用默认配置
./target/release/realconsole

# 使用指定配置文件
./target/release/realconsole --config realconsole.yaml

# 单次执行模式
./target/release/realconsole --once "你好"
```

### 手动配置（高级用户）

如果你不想使用配置向导，可以手动创建配置：

1. **复制环境变量示例**：
```bash
cp .env.example .env
```

2. **编辑 `.env` 填入 API Key**：
```bash
DEEPSEEK_API_KEY=sk-your-key-here
```

3. **编辑 `realconsole.yaml` 配置 LLM**：
```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}

features:
  shell_enabled: true
  memory_enabled: true
  tool_calling_enabled: true
```

**详细配置指南**：
- [配置向导设计](docs/01-understanding/design/config-wizard.md)
- [LLM 配置指南](docs/02-practice/user/llm-setup.md)
- [完整用户手册](docs/02-practice/user/user-guide.md)

## 💬 使用示例

### 1. 智能对话（懒人模式）

直接输入问题，无需命令前缀：

```bash
» 你好
你好！我是 AI 助手，有什么可以帮助你的吗？

» 用 Rust 写一个 hello world
好的，这是一个简单的 Rust Hello World 程序：

fn main() {
    println!("Hello, World!");
}

要运行它：
1. 保存为 main.rs
2. 运行: rustc main.rs && ./main
```

### 2. Shell 命令执行

使用 `!` 前缀执行系统命令：

```bash
» !pwd
/Users/user/project/realconsole

» !ls -la
total 96
drwxr-xr-x  10 user  staff   320 Oct 14 10:30 .
...

» !echo "Hello from shell"
Hello from shell

» !date
2025年10月14日 星期二 00时41分12秒 CST
```

### 3. 工具调用（自动执行）✨

启用工具调用后，LLM 会自动调用工具完成任务：

```bash
# 启用工具调用（编辑 realconsole.yaml）
features:
  tool_calling_enabled: true

# 示例 1: 自动计算
» 帮我计算 2 的 10 次方

[LLM 自动调用 calculator 工具]
根据计算结果，2^10 = 1024

# 示例 2: 文件操作
» 读取 README.md 的前 5 行

[LLM 自动调用 read_file 工具]
文件内容：
# RealConsole (Rust)
极简版智能 CLI Agent...

# 示例 3: 获取时间
» 现在几点了？

[LLM 自动调用 get_datetime 工具]
当前时间是 2025-01-15 14:30:45
```

**内置工具**:
- `calculator` - 数学计算（支持 +、-、*、/、^、sin、cos、sqrt 等）
- `read_file` - 读取文件内容
- `write_file` - 写入文件内容
- `list_dir` - 列出目录内容
- `get_datetime` - 获取当前日期时间

**查看所有工具**:
```bash
» /tools
可用工具 5 个工具:
  • calculator - 执行数学计算表达式
  • read_file - 读取文件内容
  • write_file - 写入文件内容
  • list_dir - 列出目录内容
  • get_datetime - 获取当前日期时间
```

详细文档：
- **用户指南**：[docs/02-practice/user/tool-calling-guide.md](docs/02-practice/user/tool-calling-guide.md)
- **开发者指南**：[docs/02-practice/developer/tool-development.md](docs/02-practice/developer/tool-development.md)

---

### 4. 多层次帮助系统 📚

使用 `/` 前缀访问系统命令：

```bash
» /help
💬 RealConsole v0.7.0

智能对话:
  直接输入问题即可，无需命令前缀
  示例: 计算 2 的 10 次方

⚡ 快速命令:
  /help       显示此帮助
  /help all   显示所有命令（详细）
  /examples   查看使用示例
  /quickref   快速参考卡片
  /quit       退出程序
...

» /help all           # 完整帮助
» /help tools         # 工具管理帮助
» /help memory        # 记忆系统帮助
» /help shell         # Shell 执行帮助

» /examples           # 使用示例库
💡 RealConsole 使用示例

━━━ 智能对话 ━━━
  计算 2 的 10 次方
  用 Rust 写一个 hello world
  ...

» /quickref           # 快速参考卡片
╭─────────────── RealConsole 快速参考 ───────────────╮
│                                                   │
│  智能对话        直接输入问题                        │
│  执行 Shell      !<命令>                           │
│  系统命令        /<命令>                            │
│  ...                                              │
╰───────────────────────────────────────────────────╯

» /quit
Bye 👋
```

---

### 5. 任务编排系统 ⭐ NEW

v1.0.0 新增的任务编排功能，让 AI 自动分解复杂目标为可执行任务：

```bash
# 步骤 1: 用自然语言描述目标
» /plan 创建一个Rust项目，包含src目录和tests目录，然后创建main.rs和lib.rs

🤖 LLM 智能分解任务...
✓ 已分解为 6 个子任务

📊 执行计划
▸ 4 阶段 · 6 任务 · ⚡ 20秒 (节省 10秒)
├─ → Stage 1 (5s)
│  └─ 创建项目根目录 $ mkdir -p myproject
├─ ⇉ Stage 2 (5s)  [并行执行]
│  ├─ 创建src目录 $ mkdir -p myproject/src
│  └─ 创建tests目录 $ mkdir -p myproject/tests
├─ ⇉ Stage 3 (5s)  [并行执行]
│  ├─ 创建main.rs $ touch myproject/src/main.rs
│  └─ 创建lib.rs $ touch myproject/src/lib.rs
└─ → Stage 4 (5s)
   └─ 创建测试文件 $ touch myproject/tests/integration_test.rs

使用 /execute 执行

# 步骤 2: 执行计划
» /execute
⚡ 开始执行: 创建Rust项目...

→ Stage 1: 创建项目根目录 ✓ (2s)
⇉ Stage 2: 并行执行 src 和 tests 目录创建 ✓ (3s)
⇉ Stage 3: 并行执行 main.rs 和 lib.rs 创建 ✓ (2s)
→ Stage 4: 创建测试文件 ✓ (3s)

✓ 6/6 · 100% · 10秒

# 查看任务列表
» /tasks
创建Rust项目 6 任务 · 4 阶段 · 20秒
├─ → Stage 1
│  └─ 创建项目根目录
├─ ⇉ Stage 2
│  ├─ 创建src目录
│  └─ 创建tests目录
└─ ...

# 查看执行状态
» /task_status
✓ 6/6 · 10秒 · 100%
✓ 创建项目根目录 (2s)
✓ 创建src目录 (3s)
✓ 创建tests目录 (3s)
✓ 创建main.rs (2s)
✓ 创建lib.rs (2s)
✓ 创建测试文件 (3s)
```

**核心特性**:
- ✅ **LLM智能分解** - 自然语言描述目标，AI自动拆解为可执行步骤
- ✅ **依赖分析** - Kahn拓扑排序自动检测任务依赖，保证执行顺序
- ✅ **并行优化** - 自动识别可并行任务，显著提升执行效率（最大4并发）
- ✅ **极简可视化** - 树状结构清晰展示任务层次，输出行数减少75%+
- ✅ **安全防护** - 继承Shell黑名单和超时控制机制

**典型场景**:
- 项目脚手架创建（目录、文件、配置初始化）
- 批量文件操作（重命名、转换、清理）
- 数据处理流水线（提取、转换、加载）
- 开发工作流（构建、测试、部署）

详细文档：
- **使用指南**：[examples/task_system_usage.md](examples/task_system_usage.md)
- **可视化设计**：[examples/task_visualization.md](examples/task_visualization.md)

---

### 6. DevOps 工作流 ✨

#### 项目上下文感知

快速了解项目信息和推荐命令：

```bash
» /project
📦 项目上下文

  项目名称: realconsole
  项目类型: Rust 项目
  根目录: /Users/user/realconsole

🔨 推荐命令:
  构建: cargo build
  测试: cargo test
  运行: cargo run

📊 项目信息:
  ✓ 发现 Cargo.toml
  ✓ 发现 src/ 目录
  ✓ 发现测试目录

🔄 Git 信息:
  分支: main
  状态: 2 个文件已修改
```

#### Git 智能助手

加速 Git 工作流：

```bash
# 1. 查看 Git 状态（彩色分类显示）
» /gs
📊 Git 仓库状态

📁 已修改文件 (2):
  • src/main.rs
  • Cargo.toml

# 2. 查看差异分析
» /gd
📊 代码变更分析

📈 统计信息:
  • 新增: 120 行
  • 删除: 45 行
  • 修改文件: 2 个

🔍 变更模式:
  ✓ 发现新函数定义
  ✓ 发现新测试用例

# 3. 自动生成提交消息（遵循 Conventional Commits）
» /ga
📝 变更分析与提交建议

🎯 变更类型: feat (新功能)
📁 影响范围: core

💬 建议的提交消息:
feat(core): add DevOps features

- Add project context detection
- Add Git smart assistant
- Add log analyzer
- Add system monitor

详细变更:
- 新增 3,431 行代码
- 新增 37+ 测试
- 4 个新模块
```

#### 日志分析工具

快速诊断日志问题：

```bash
# 分析日志文件
» /la /var/log/app.log
📊 日志分析报告

📈 统计信息:
  • 总行数: 10,234
  • 时间范围: 2025-01-15 10:00:00 - 14:30:45

📊 日志级别分布:
  • ERROR: 23 (0.2%)
  • WARN: 156 (1.5%)
  • INFO: 8,945 (87.4%)
  • DEBUG: 1,110 (10.9%)

⚠️ Top 5 错误模式:
  1. "Connection timeout after Nms" - 出现 12 次
  2. "Failed to load config from /PATH" - 出现 5 次
  3. "Database query timeout" - 出现 3 次

🏥 健康度: 良好 (ERROR < 1%)

# 只查看错误
» /le /var/log/app.log
[显示所有 ERROR 级别日志...]

# 实时监控日志尾部（类似 tail -f）
» /lt /var/log/app.log
[实时显示新增日志...]
```

#### 系统监控工具

快速查看系统资源：

```bash
# 系统概览（一键查看所有资源）
» /sys
💻 系统监控

━━━ CPU ━━━
  使用率: 15.3%
  • 用户: 8.2%
  • 系统: 7.1%
  • 空闲: 84.7%

━━━ 内存 ━━━
  总量: 16.0 GB
  已用: 8.5 GB (53%)
  可用: 7.5 GB
  缓存: 2.3 GB

━━━ 磁盘 ━━━
  / (根分区):
    总量: 500 GB
    已用: 320 GB (64%)
    可用: 180 GB

# CPU 详情
» /cpu
[显示 CPU 详细信息...]

# 内存详情
» /memory-info
[显示内存详细信息...]

# 磁盘使用
» /disk
[显示所有磁盘分区...]

# 进程 TOP 列表
» /top
🔝 进程资源使用 TOP 5

按 CPU 排序:
  1. chrome - 45.2% CPU, 1.2 GB 内存
  2. node - 12.3% CPU, 850 MB 内存
  ...
```

---

### 6. 友好错误提示 ⚠️

RealConsole 提供 30+ 错误代码和详细的修复建议：

```bash
» !rm -rf /
[E302] 命令包含危险操作，已被安全策略阻止

💡 修复建议:
1. 此命令可能造成系统损坏，建议使用更安全的替代方案
2. 查看允许的命令列表和安全策略
   📖 https://docs.realconsole.com/shell-safety

» !sleep 20
[E303] 命令执行超时（超过 10 秒）

💡 修复建议:
1. 命令执行时间过长，请检查命令或增加超时时间
2. 在配置文件中调整 features.shell_timeout
   💻 vi realconsole.yaml

» realconsole --config nonexistent.yaml
[E001] 配置文件不存在: nonexistent.yaml

💡 修复建议:
1. 运行配置向导创建配置文件
   💻 realconsole wizard --quick
2. 查看配置指南
   📖 https://docs.realconsole.com/config
```

## 📁 项目结构

```
realconsole/
├── README.md                 # 项目主文档
├── Cargo.toml                # Rust 项目配置
├── realconsole.yaml        # 主配置文件
├── .env                      # 环境变量（不提交）
│
├── src/                      # 🦀 源代码
│   ├── main.rs               # 程序入口
│   ├── agent.rs              # Agent 核心（Intent DSL 集成）
│   ├── repl.rs               # REPL 交互循环
│   ├── config.rs             # 配置系统
│   ├── command/              # 命令系统
│   │   ├── command.rs        # 命令注册与分发
│   │   ├── commands_core.rs  # 核心命令
│   │   ├── task_cmd.rs       # 任务编排命令 ⭐ NEW
│   │   └── commands_*.rs     # 其他命令模块
│   ├── shell_executor.rs     # Shell 命令执行
│   ├── llm_manager.rs        # LLM 管理器
│   ├── llm/                  # LLM 客户端
│   │   ├── ollama.rs
│   │   ├── deepseek.rs
│   │   └── openai.rs
│   ├── task/                 # ⭐ NEW - 任务编排系统
│   │   ├── mod.rs            # 模块定义
│   │   ├── types.rs          # 任务数据结构
│   │   ├── decomposer.rs     # LLM任务分解器
│   │   ├── planner.rs        # 依赖分析与规划
│   │   └── executor.rs       # 并行执行引擎
│   ├── dsl/                  # DSL 系统
│   │   ├── intent/           # Intent DSL
│   │   │   ├── types.rs      # 核心数据结构
│   │   │   ├── matcher.rs    # 意图匹配器
│   │   │   ├── template.rs   # 模板引擎
│   │   │   ├── builtin.rs    # 50+ 内置意图
│   │   │   └── extractor.rs  # 实体提取引擎
│   │   └── type_system/      # 类型系统
│   ├── memory.rs             # 记忆系统
│   ├── tool.rs               # 工具注册
│   ├── tool_executor.rs      # 工具执行引擎
│   └── builtin_tools.rs      # 14+ 内置工具
│
├── tests/                    # 🧪 测试
│   ├── test_*.rs             # 单元测试
│   └── test_intent_integration.rs  # Intent DSL 集成测试
│
├── docs/                     # 📚 文档（五态架构）
│   ├── README.md             # 文档中心索引
│   ├── CHANGELOG.md          # 完整开发历史
│   ├── 00-core/              # 核心理念（哲学、愿景、路线图）
│   ├── 01-understanding/     # 理解态（设计、分析、思考）
│   │   ├── design/           # 设计文档集
│   │   ├── analysis/         # 分析文档
│   │   └── thinking/         # 思考笔记
│   ├── 02-practice/          # 实践态（指南、用例、示例）
│   │   ├── user/             # 用户指南
│   │   ├── developer/        # 开发者指南
│   │   └── use-cases/        # 使用场景
│   ├── 03-evolution/         # 演化态（进展、特性）
│   │   ├── phases/           # 阶段总结
│   │   ├── features/         # 功能实现文档
│   │   └── milestones/       # 里程碑
│   ├── 04-reports/           # 协同报告（决策记录）
│   └── archive/              # 归档（226个历史文档）
│
├── config/                   # ⚙️ 配置样例
│   ├── minimal.yaml          # 最小配置示例
│   └── test-memory.yaml      # 测试记忆配置
│
├── scripts/                  # 🔧 脚本工具
│   ├── demo/                 # 演示脚本
│   └── test/                 # 测试脚本
│
└── memory/                   # 💾 记忆存储
    └── long_memory.jsonl     # 长期记忆
```

## 🏗️ 架构设计

### 核心组件

```
┌─────────────────────────────────────────┐
│              用户输入                    │
└────────────────┬────────────────────────┘
                 │
         ┌───────▼────────┐
         │  Agent::handle │
         └───────┬────────┘
                 │
      ┏━━━━━━━━━┻━━━━━━━━━┓
      ┃                    ┃
┌─────▼─────┐      ┌──────▼───────┐
│ 文本输入   │      │ Shell (!前缀) │
│           │      │               │
│ LLM 流式  │      │ Shell 执行器  │
│  输出     │      │               │
└───────────┘      └──────────────┘
```

### LLM 流式输出

- **SSE 解析**：Server-Sent Events 格式
- **实时回调**：每个 token 立即显示
- **优雅降级**：非流式客户端自动降级

详细设计：[docs/03-evolution/features/streaming.md](docs/03-evolution/features/streaming.md)

### Shell 执行

- **黑名单检查**：禁止危险命令（rm -rf /、sudo 等）
- **超时控制**：30 秒自动终止
- **输出限制**：最大 100KB
- **跨平台**：Unix/Windows 支持

详细设计：[docs/03-evolution/features/shell-execution.md](docs/03-evolution/features/shell-execution.md)

## 🔐 安全特性

### Shell 命令黑名单

以下危险命令被禁止执行：
- `rm -rf /` - 删除根目录
- `sudo` - 权限提升
- `dd if=/dev/zero` - 磁盘写入
- `mkfs` - 格式化
- Fork 炸弹、shutdown、reboot 等

示例：
```bash
» !rm -rf /
Shell 执行失败: 禁止执行危险命令: 匹配模式 'rm\s+-rf\s+/'

» !sudo apt-get update
Shell 执行失败: 禁止执行危险命令: 匹配模式 'sudo\s+'
```

## 📊 性能指标

| 指标 | 数值 |
|------|------|
| 启动时间 | < 50ms |
| 内存占用 | ~5MB |
| LLM 首 token 延迟 | < 500ms |
| Shell 命令超时 | 30s（可配置） |
| 最大输出 | 100KB（可配置） |

## 🎯 项目统计

### 代码质量
- **测试覆盖**: 645+ 测试通过（100% 通过率）
- **代码行数**: 13,000+ 行 Rust 代码
- **测试覆盖率**: 78%+
- **Clippy 警告**: 0

### 主要命令
- **系统命令**: `/help`, `/quit`, `/clear`, `/examples`, `/quickref`
- **任务编排** ⭐ NEW: `/plan`, `/execute`, `/tasks`, `/task_status`
- **项目工具**: `/project`, `/proj`
- **Git 助手**: `/gs`, `/gd`, `/ga`, `/gb`
- **日志分析**: `/la`, `/le`, `/lt`
- **系统监控**: `/sys`, `/cpu`, `/disk`, `/top`, `/memory-info`
- **工具管理**: `/tools`, `/mem`, `/log`, `/shell`

### 架构特点
- **REPL 交互循环** - 基于 rustyline 的命令行界面
- **LLM 集成** - 支持 Ollama/Deepseek/OpenAI，流式输出（SSE）
- **任务编排系统** ⭐ NEW - LLM智能分解 + Kahn拓扑排序 + 并行优化执行
- **工具调用系统** - 14+ 内置工具，支持并行执行
- **Intent DSL** - 50+ 内置意图模板，智能匹配
- **记忆系统** - 短期+长期记忆，支持搜索和导出
- **安全防护** - 黑名单检查、超时控制、输出限制
- **跨平台支持** - macOS + Linux，零额外依赖

详细开发历史和技术文档请参考 [CHANGELOG](docs/CHANGELOG.md)

## 🚧 计划功能

- [ ] 命令历史搜索
- [ ] Tab 自动补全
- [ ] 向量检索优化
- [ ] Web 界面
- [ ] 更多内置意图（当前 10 个）
- [ ] Intent DSL 性能优化（LRU 缓存、模糊匹配）

## 📚 文档

> **文档系统已升级**: 基于"一分为三"哲学的五态架构，清晰易导航 ✨

### 文档中心
- **📚 文档主索引**：[docs/README.md](docs/README.md) - 完整文档导航和推荐阅读路径

### 核心文档（00-core）
- **💭 一分为三哲学**：[docs/00-core/philosophy.md](docs/00-core/philosophy.md) - 设计理念
- **🎯 产品愿景**：[docs/00-core/vision.md](docs/00-core/vision.md) - 产品定位
- **🗺️ 技术路线图**：[docs/00-core/roadmap.md](docs/00-core/roadmap.md) - 发展规划

### 用户指南（02-practice/user）
- **🚀 快速开始**：[docs/02-practice/user/quickstart.md](docs/02-practice/user/quickstart.md) - 5分钟上手
- **📖 用户手册**：[docs/02-practice/user/user-guide.md](docs/02-practice/user/user-guide.md) - 完整功能说明
- **🛠️ 工具调用指南**：[docs/02-practice/user/tool-calling-guide.md](docs/02-practice/user/tool-calling-guide.md)
- **🧠 Intent DSL**：[docs/02-practice/user/intent-dsl-guide.md](docs/02-practice/user/intent-dsl-guide.md)
- **🔧 LLM 配置**：[docs/02-practice/user/llm-setup.md](docs/02-practice/user/llm-setup.md)
- **🌐 环境变量**：[docs/02-practice/user/env-config.md](docs/02-practice/user/env-config.md)

### 开发者文档（01-understanding & 02-practice/developer）
- **🏗️ 架构总览**：[docs/01-understanding/overview.md](docs/01-understanding/overview.md)
- **👨‍💻 开发者指南**：[docs/02-practice/developer/developer-guide.md](docs/02-practice/developer/developer-guide.md)
- **🔨 工具开发**：[docs/02-practice/developer/tool-development.md](docs/02-practice/developer/tool-development.md)
- **📘 API 参考**：[docs/02-practice/developer/api-reference.md](docs/02-practice/developer/api-reference.md)

### 功能文档（03-evolution/features）
- **🔄 Git 智能助手**：[docs/03-evolution/features/git-assistant.md](docs/03-evolution/features/git-assistant.md)
- **📊 日志分析器**：[docs/03-evolution/features/log-analyzer.md](docs/03-evolution/features/log-analyzer.md)
- **💻 系统监控**：[docs/03-evolution/features/system-monitor.md](docs/03-evolution/features/system-monitor.md)
- **🧙 配置向导**：[docs/03-evolution/features/config-wizard.md](docs/03-evolution/features/config-wizard.md)
- **⚡ 懒人模式**：[docs/03-evolution/features/lazy-mode.md](docs/03-evolution/features/lazy-mode.md)
- **🌊 流式输出**：[docs/03-evolution/features/streaming.md](docs/03-evolution/features/streaming.md)
- **🔧 Shell 执行**：[docs/03-evolution/features/shell-execution.md](docs/03-evolution/features/shell-execution.md)
- **📚 功能总览**：[docs/03-evolution/features/summary.md](docs/03-evolution/features/summary.md)

### 进度与报告
- **📊 开发历史**：[docs/CHANGELOG.md](docs/CHANGELOG.md) - 完整变更日志
- **📈 阶段总结**：[docs/03-evolution/phases/](docs/03-evolution/phases/) - 开发历程记录
- **📝 协同报告**：[docs/04-reports/](docs/04-reports/) - 重要决策和分析报告

## 🔧 开发

### 运行测试

```bash
cargo test
```

### 运行演示

```bash
# Shell 执行演示
./scripts/demo/demo_shell.sh

# 懒人模式演示
./scripts/demo/demo_lazy_mode.sh

# Deepseek 演示
./scripts/demo/demo-deepseek.sh
```

### 代码格式化

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## 🤝 贡献

欢迎贡献！请确保：
1. 代码通过 `cargo test`
2. 代码格式化 `cargo fmt`
3. 无 clippy 警告 `cargo clippy`

## 📄 License

MIT License - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- **Python 版本**：[SmartConsole](https://github.com/example/smartconsole) - 设计灵感来源
- **Rust 社区**：优秀的工具和库
- **LLM 提供商**：Ollama、Deepseek、OpenAI

---

**RealConsole** - 极简而强大的智能 CLI Agent 🚀
