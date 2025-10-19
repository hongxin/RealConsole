# 项目结构详解

> 完整的 RealConsole 代码组织和目录结构说明

## 📁 完整目录树

```
realconsole/
├── README.md                 # 项目主文档
├── Cargo.toml                # Rust 项目配置
├── realconsole.yaml          # 主配置文件
├── .env                      # 环境变量（不提交）
│
├── src/                      # 🦀 源代码
│   ├── main.rs               # 程序入口
│   ├── lib.rs                # 库入口
│   ├── agent.rs              # Agent 核心（Intent DSL 集成）
│   ├── repl.rs               # REPL 交互循环
│   ├── config.rs             # 配置系统
│   ├── display.rs            # 输出显示与格式化
│   ├── i18n.rs               # 国际化系统
│   │
│   ├── command/              # 命令系统
│   │   ├── mod.rs            # 命令模块定义
│   │   ├── command.rs        # 命令注册与分发
│   │   ├── commands_core.rs  # 核心命令（help, quit等）
│   │   ├── task_cmd.rs       # 任务编排命令 ⭐ NEW
│   │   ├── git_cmd.rs        # Git 智能助手命令
│   │   ├── project_cmd.rs    # 项目上下文命令
│   │   ├── log_cmd.rs        # 日志分析命令
│   │   ├── system_cmd.rs     # 系统监控命令
│   │   ├── memory_cmd.rs     # 记忆系统命令
│   │   └── tool_cmd.rs       # 工具管理命令
│   │
│   ├── command_router.rs     # 智能命令路由器 ⭐ NEW
│   ├── shell_executor.rs     # Shell 命令执行
│   │
│   ├── llm/                  # LLM 客户端
│   │   ├── mod.rs            # LLM 模块定义
│   │   ├── client.rs         # LlmClient trait 定义
│   │   ├── manager.rs        # LLM 管理器
│   │   ├── ollama.rs         # Ollama 客户端实现
│   │   ├── deepseek.rs       # Deepseek 客户端实现
│   │   └── openai.rs         # OpenAI 客户端实现
│   │
│   ├── task/                 # ⭐ NEW - 任务编排系统
│   │   ├── mod.rs            # 任务系统模块定义
│   │   ├── types.rs          # 任务数据结构（Task, TaskStatus等）
│   │   ├── decomposer.rs     # LLM任务智能分解器
│   │   ├── planner.rs        # 依赖分析与任务规划（Kahn算法）
│   │   └── executor.rs       # 并行执行引擎（Tokio并发）
│   │
│   ├── dsl/                  # DSL 系统
│   │   ├── mod.rs            # DSL 模块定义
│   │   ├── intent/           # Intent DSL 子系统
│   │   │   ├── mod.rs        # Intent 模块定义
│   │   │   ├── types.rs      # 核心数据结构（Intent, Pattern等）
│   │   │   ├── matcher.rs    # 意图匹配器（正则+模板）
│   │   │   ├── template.rs   # 模板引擎（参数替换）
│   │   │   ├── builtin.rs    # 50+ 内置意图定义
│   │   │   └── extractor.rs  # 实体提取引擎
│   │   └── type_system/      # 类型系统
│   │       ├── mod.rs
│   │       ├── types.rs      # 类型定义
│   │       └── validator.rs  # 类型验证
│   │
│   ├── conversation/         # 多轮对话系统
│   │   ├── mod.rs            # 对话模块定义
│   │   ├── manager.rs        # 对话管理器
│   │   ├── context.rs        # 对话上下文
│   │   ├── state.rs          # 对话状态机
│   │   └── analyzer.rs       # 参数分析器
│   │
│   ├── memory/               # 记忆系统
│   │   ├── mod.rs            # 记忆模块定义
│   │   ├── memory.rs         # 记忆管理器
│   │   ├── short_term.rs     # 短期记忆
│   │   └── long_term.rs      # 长期记忆（持久化）
│   │
│   ├── tool/                 # 工具调用系统
│   │   ├── mod.rs            # 工具模块定义
│   │   ├── tool.rs           # Tool trait 定义
│   │   ├── registry.rs       # 工具注册表
│   │   ├── executor.rs       # 工具执行引擎
│   │   └── builtin_tools.rs  # 14+ 内置工具实现
│   │
│   ├── wizard/               # 配置向导系统
│   │   ├── mod.rs            # 向导模块定义
│   │   ├── wizard.rs         # 向导主流程
│   │   ├── prompt.rs         # 交互式提示
│   │   ├── validator.rs      # 配置验证器
│   │   └── generator.rs      # 配置文件生成器
│   │
│   ├── git/                  # Git 集成
│   │   ├── mod.rs            # Git 模块定义
│   │   ├── repository.rs     # Git 仓库操作
│   │   ├── status.rs         # 状态查询
│   │   └── analyzer.rs       # 变更分析
│   │
│   ├── log/                  # 日志分析
│   │   ├── mod.rs            # 日志模块定义
│   │   ├── parser.rs         # 日志解析器
│   │   ├── analyzer.rs       # 日志分析器
│   │   └── health.rs         # 健康度评估
│   │
│   ├── system/               # 系统监控
│   │   ├── mod.rs            # 系统模块定义
│   │   ├── monitor.rs        # 系统监控器
│   │   ├── cpu.rs            # CPU 监控
│   │   ├── memory.rs         # 内存监控
│   │   └── disk.rs           # 磁盘监控
│   │
│   ├── project/              # 项目上下文
│   │   ├── mod.rs            # 项目模块定义
│   │   ├── detector.rs       # 项目类型检测
│   │   └── context.rs        # 项目上下文管理
│   │
│   ├── error.rs              # 错误类型定义
│   └── utils.rs              # 通用工具函数
│
├── tests/                    # 🧪 测试
│   ├── common/               # 测试公共模块
│   │   └── mod.rs
│   ├── test_agent.rs         # Agent 单元测试
│   ├── test_command_router.rs # 命令路由测试 ⭐ NEW
│   ├── test_config.rs        # 配置系统测试
│   ├── test_conversation_integration.rs  # 对话集成测试
│   ├── test_intent_integration.rs        # Intent DSL 集成测试
│   ├── test_shell.rs         # Shell 执行测试
│   ├── test_task_*.rs        # 任务系统测试 ⭐ NEW
│   └── test_tool_executor.rs # 工具执行器测试
│
├── benches/                  # 🏃 性能基准测试
│   ├── intent_matching.rs    # Intent 匹配性能测试
│   ├── task_execution.rs     # 任务执行性能测试
│   └── tool_calling.rs       # 工具调用性能测试
│
├── docs/                     # 📚 文档（五态架构）
│   ├── README.md             # 文档中心索引
│   ├── CHANGELOG.md          # 完整开发历史
│   │
│   ├── 00-core/              # 核心理念
│   │   ├── philosophy.md     # 一分为三哲学
│   │   ├── vision.md         # 产品愿景
│   │   └── roadmap.md        # 技术路线图
│   │
│   ├── 01-understanding/     # 理解态（设计、分析、思考）
│   │   ├── overview.md       # 架构总览
│   │   ├── design/           # 设计文档集
│   │   │   ├── config-wizard.md
│   │   │   ├── error-system.md
│   │   │   ├── intent-matching.md
│   │   │   └── task-orchestration.md
│   │   ├── analysis/         # 分析文档
│   │   │   ├── security.md
│   │   │   ├── technical-debt.md
│   │   │   └── python-rust-gap.md
│   │   └── thinking/         # 思考笔记
│   │       └── dsl-design.md
│   │
│   ├── 02-practice/          # 实践态（指南、用例、示例）
│   │   ├── user/             # 用户指南
│   │   │   ├── quickstart.md
│   │   │   ├── user-guide.md
│   │   │   ├── tool-calling-guide.md
│   │   │   ├── intent-dsl-guide.md
│   │   │   ├── llm-setup.md
│   │   │   └── env-config.md
│   │   ├── developer/        # 开发者指南
│   │   │   ├── developer-guide.md
│   │   │   ├── api-reference.md
│   │   │   ├── tool-development.md
│   │   │   ├── i18n-guide.md
│   │   │   └── project-structure.md  # ← 本文档
│   │   └── use-cases/        # 使用场景
│   │       ├── 10-cases.md
│   │       ├── 20-cases.md
│   │       └── 50-cases.md
│   │
│   ├── 03-evolution/         # 演化态（进展、特性）
│   │   ├── phases/           # 阶段总结
│   │   │   ├── phase2-summary.md
│   │   │   ├── phase5-plan.md
│   │   │   └── phase7-completion.md
│   │   ├── features/         # 功能实现文档
│   │   │   ├── git-assistant.md
│   │   │   ├── log-analyzer.md
│   │   │   ├── system-monitor.md
│   │   │   ├── config-wizard.md
│   │   │   ├── lazy-mode.md
│   │   │   ├── streaming.md
│   │   │   ├── shell-execution.md
│   │   │   └── summary.md
│   │   └── milestones/       # 里程碑
│   │       └── VERSION-MERGE-2025-10-19.md
│   │
│   ├── 04-reports/           # 协同报告（决策记录）
│   │   └── README.md
│   │
│   └── archive/              # 归档（226个历史文档）
│       ├── old-designs/
│       ├── old-progress/
│       └── release/
│
├── examples/                 # 💡 示例
│   ├── task_system_usage.md  # 任务系统使用示例
│   └── task_visualization.md # 任务可视化示例
│
├── config/                   # ⚙️ 配置样例
│   ├── minimal.yaml          # 最小配置示例
│   └── test-memory.yaml      # 测试记忆配置
│
├── scripts/                  # 🔧 脚本工具
│   ├── demo/                 # 演示脚本
│   │   ├── demo_shell.sh
│   │   ├── demo_lazy_mode.sh
│   │   └── demo-deepseek.sh
│   └── test/                 # 测试脚本
│       └── run_coverage.sh
│
├── locales/                  # 🌐 国际化资源
│   ├── zh-CN.yaml            # 简体中文
│   └── en-US.yaml            # 美式英语
│
├── memory/                   # 💾 记忆存储
│   ├── short_memory.jsonl    # 短期记忆（临时）
│   └── long_memory.jsonl     # 长期记忆（持久化）
│
└── target/                   # 📦 构建输出
    ├── debug/                # 调试构建
    └── release/              # 发布构建
```

## 核心模块说明

### 1. Agent 核心 (`src/agent.rs`)

**职责**：智能代理的核心调度器

- 统一入口处理用户输入
- 集成 Intent DSL 进行意图识别
- 协调 LLM、Shell、系统命令的路由
- 管理对话上下文和记忆

**关键方法**：
- `Agent::new()` - 创建 Agent 实例
- `Agent::handle()` - 处理用户输入（核心方法）
- `Agent::chat()` - LLM 对话处理
- `Agent::execute_shell()` - Shell 命令执行

### 2. 命令路由器 (`src/command_router.rs`)

**职责**：智能识别和路由命令类型

**路由优先级**：
1. 强制 Shell (`!` 前缀) - 最高优先级
2. 系统命令 (`/` 前缀) - 次优先级
3. 常见 Shell 命令 - 智能识别（80+ 常用命令）
4. 自然语言 - 兜底处理

**特点**：
- 无感过渡：常见命令零延迟识别
- 启发式规则：排除明显的自然语言（包含"我"、"你"、"吗"等）
- 可配置：支持禁用智能路由

### 3. LLM 集成 (`src/llm/`)

**支持的提供商**：
- **Ollama** - 本地部署，隐私优先
- **Deepseek** - 高性价比云服务
- **OpenAI** - 兼容接口

**核心特性**：
- 流式输出（SSE）- 实时响应
- 工具调用（Function Calling）- 自动调用内置工具
- 多 LLM 切换 - Primary + Fallback 机制
- 统一接口（`LlmClient` trait）

### 4. 任务编排系统 (`src/task/`)

**架构分层**：
1. **分解器** (`decomposer.rs`) - LLM 智能分解目标为任务
2. **规划器** (`planner.rs`) - Kahn 拓扑排序分析依赖
3. **执行器** (`executor.rs`) - Tokio 并发并行执行

**核心算法**：
- Kahn 拓扑排序 - 依赖关系检测
- 并行度控制 - 最大 4 并发执行
- 超时与取消 - 优雅中断机制

### 5. Intent DSL (`src/dsl/intent/`)

**设计理念**：自然语言到结构化命令的桥梁

**核心组件**：
- **匹配器** (`matcher.rs`) - 正则匹配 + 优先级排序
- **模板引擎** (`template.rs`) - 参数替换和命令生成
- **实体提取** (`extractor.rs`) - 从输入中提取关键信息
- **内置意图** (`builtin.rs`) - 50+ 预定义意图模板

**匹配流程**：
```
用户输入 → 正则匹配 → 参数提取 → 模板替换 → Shell 命令
```

### 6. 工具调用系统 (`src/tool/`)

**内置工具**：
- `calculator` - 数学计算
- `read_file` / `write_file` - 文件操作
- `list_dir` - 目录列表
- `get_datetime` - 时间查询
- `http_get` - HTTP 请求
- ... 14+ 工具

**执行模式**：
- 串行执行 - 按顺序依次调用
- 并行执行 - Tokio 并发优化

### 7. 配置向导 (`src/wizard/`)

**功能**：
- 交互式配置生成
- LLM 提供商选择（Ollama / Deepseek）
- API Key 验证
- 自动生成 `realconsole.yaml` 和 `.env`

**模式**：
- Quick Mode（5 分钟）- 快速完成基础配置
- Full Mode（完整）- 所有高级选项

## 测试架构

### 单元测试

- **覆盖率**: 78%+
- **测试数量**: 654+ 测试
- **运行**: `cargo test`

### 集成测试

- Intent DSL 集成测试
- 对话系统集成测试
- 任务编排集成测试

### 性能基准测试

- Intent 匹配性能
- 任务执行性能
- 工具调用性能

**运行**: `cargo bench`

## 文档架构（五态）

### 五态哲学

基于"一分为三"哲学的文档组织：

1. **核心态** (00-core) - 理念、愿景、路线图
2. **理解态** (01-understanding) - 设计、分析、思考
3. **实践态** (02-practice) - 指南、用例、示例
4. **演化态** (03-evolution) - 进展、特性、里程碑
5. **报告态** (04-reports) - 决策、协同报告

### 文档导航

- 📚 **文档中心**: [docs/README.md](../README.md)
- 🚀 **快速开始**: [docs/02-practice/user/quickstart.md](../user/quickstart.md)
- 👨‍💻 **开发指南**: [docs/02-practice/developer/developer-guide.md](developer-guide.md)

## 构建与部署

### 开发构建

```bash
cargo build
```

### 发布构建

```bash
cargo build --release
```

### 安装到用户目录

```bash
make install
# 或
./scripts/install.sh
```

**安装位置**：
- 可执行文件：`~/.local/bin/realconsole`
- 配置目录：`~/.realconsole/`
- 记忆数据：`~/.realconsole/memory/`

## 依赖管理

### 核心依赖

- **tokio** - 异步运行时
- **rustyline** - REPL 交互
- **serde** / **serde_yaml** - 序列化
- **reqwest** - HTTP 客户端
- **colored** - 彩色输出

### 开发依赖

- **mockito** - HTTP Mock
- **criterion** - 性能基准测试

### 依赖原则

- 最小化依赖
- 优先 pure Rust 实现
- 避免过度抽象

## 代码风格

### 命名规范

- 文件/函数：`snake_case`
- 类型/Trait：`CamelCase`
- 常量：`SCREAMING_SNAKE_CASE`

### 模块组织

- 每个模块一个目录
- `mod.rs` 作为模块入口
- 公开接口放在模块根部

### 错误处理

- 库代码：使用 `Result<T, E>`
- 应用代码：使用 `anyhow::Result`
- 自定义错误：使用 `thiserror`

## 性能考量

### 启动优化

- 延迟加载非核心模块
- 配置文件缓存
- 避免启动时的网络请求

### 运行时优化

- Intent 匹配 LRU 缓存
- 异步 I/O（Tokio）
- 并行任务执行

### 内存优化

- 流式处理大文件
- 定期清理短期记忆
- 限制输出缓冲区大小

## 安全考虑

### Shell 执行安全

- 黑名单检查
- 超时控制（30秒）
- 输出大小限制（100KB）

### API Key 安全

- 环境变量存储
- `.env` 文件不提交
- 配置文件中使用 `${VAR}` 引用

### 文件访问安全

- 路径验证
- 禁止访问系统关键目录
- 文件大小限制

## 扩展开发

### 添加新工具

1. 实现 `Tool` trait
2. 在 `builtin_tools.rs` 注册
3. 添加测试用例

详见：[工具开发指南](tool-development.md)

### 添加新 Intent

1. 在 `builtin.rs` 定义 Intent
2. 设置匹配模式和模板
3. 添加测试用例

详见：[Intent DSL 指南](../user/intent-dsl-guide.md)

### 添加新 LLM 提供商

1. 实现 `LlmClient` trait
2. 在 `llm_manager.rs` 注册
3. 更新配置示例

详见：[LLM 集成指南](../user/llm-setup.md)

## 参考资料

- **API 参考**: [api-reference.md](api-reference.md)
- **开发指南**: [developer-guide.md](developer-guide.md)
- **架构总览**: [../../01-understanding/overview.md](../../01-understanding/overview.md)
- **项目路线图**: [../../00-core/roadmap.md](../../00-core/roadmap.md)

---

**最后更新**: 2025-10-19 | **版本**: v1.1.0
