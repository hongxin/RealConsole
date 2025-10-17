# RealConsole 项目目录结构

> 版本：v0.6.0
> 最后更新：2025-10-16
> 文档架构：基于"一分为三"哲学的五态架构

---

## 📁 目录结构

```
realconsole/
├── src/                    # 源代码
│   ├── agent.rs           # 核心 Agent 逻辑
│   ├── llm/               # LLM 客户端实现
│   ├── dsl/               # Intent DSL & Type System
│   ├── tool*.rs           # 工具系统
│   ├── wizard/            # 配置向导
│   ├── commands/          # 命令系统（Git、项目、日志、系统监控）
│   ├── git_assistant.rs   # Git 智能助手
│   ├── project_context.rs # 项目上下文感知
│   ├── log_analyzer.rs    # 日志分析工具
│   ├── system_monitor.rs  # 系统监控工具
│   └── ...
│
├── tests/                  # 集成测试
│   ├── test_intent_*.rs   # Intent DSL 测试
│   ├── test_function_calling_e2e.rs  # Tool Calling E2E
│   └── ...
│
├── docs/                   # 文档中心（五态架构）✨ v0.6.0
│   ├── README.md          # 文档中心索引
│   ├── CHANGELOG.md       # 完整变更历史
│   │
│   ├── 00-core/           # 【道】核心理念
│   │   ├── philosophy.md      # 一分为三哲学
│   │   ├── vision.md          # 产品愿景
│   │   └── roadmap.md         # 技术路线图
│   │
│   ├── 01-understanding/  # 【理解态】认知和设计
│   │   ├── README.md
│   │   ├── overview.md        # 架构总览
│   │   ├── design/            # 设计文档（5个）
│   │   ├── analysis/          # 分析文档（4个）
│   │   └── thinking/          # 思考笔记（3个）
│   │
│   ├── 02-practice/       # 【实践态】使用和开发
│   │   ├── README.md
│   │   ├── user/              # 用户指南（6个）
│   │   │   ├── quickstart.md
│   │   │   ├── user-guide.md
│   │   │   ├── tool-calling-guide.md
│   │   │   ├── intent-dsl-guide.md
│   │   │   ├── llm-setup.md
│   │   │   └── env-config.md
│   │   ├── developer/         # 开发者指南（3个）
│   │   │   ├── developer-guide.md
│   │   │   ├── tool-development.md
│   │   │   └── api-reference.md
│   │   ├── use-cases/         # 使用场景（5个 + README）
│   │   └── examples/          # 示例代码
│   │
│   ├── 03-evolution/      # 【演化态】进展和历史
│   │   ├── README.md
│   │   ├── phases/            # 阶段总结（4个）
│   │   │   ├── phase3-intent-dsl.md
│   │   │   ├── phase5-pipeline.md
│   │   │   └── phase7-polish.md
│   │   ├── features/          # 功能实现（8个）
│   │   │   ├── shell-execution.md
│   │   │   ├── streaming.md
│   │   │   ├── git-assistant.md
│   │   │   ├── log-analyzer.md
│   │   │   ├── system-monitor.md
│   │   │   └── ...
│   │   └── milestones/        # 里程碑（2个）
│   │
│   ├── 04-reports/        # 【协同报告】工作成果
│   │   ├── README.md
│   │   └── reorganization-2025-10-16.md
│   │
│   └── archive/           # 【归档】历史文档
│       ├── README.md
│       ├── old-progress/      # 详细进展（48个）
│       │   ├── weekly/
│       │   ├── daily/
│       │   └── bugfixes/
│       ├── old-designs/       # 过时设计（32个）
│       └── _old-structure/    # 旧目录结构备份
│
├── scripts/                # 脚本工具
│   ├── test/              # 测试脚本
│   │   ├── test_cd.sh
│   │   ├── test_visual.sh
│   │   └── test_wizard_quick.sh
│   └── ...
│
├── config/                 # 配置示例
│   ├── minimal.yaml       # 最小配置
│   └── full.yaml          # 完整配置
│
├── examples/               # 示例代码
│   └── wizard_demo.rs     # 配置向导示例
│
├── benches/                # 性能基准测试
│   └── intent_matching.rs
│
├── coverage/               # 测试覆盖率报告（git ignore）
├── flamegraph/             # 性能分析（git ignore）
├── memory/                 # 运行时数据（git ignore）
├── sandbox/                # 开发沙箱（git ignore）
│
├── Cargo.toml             # Rust 项目配置
├── Cargo.lock             # 依赖锁定
├── .gitignore             # Git 忽略规则
├── .env                   # 环境变量（git ignore）
├── .env.example           # 环境变量示例
│
├── realconsole.yaml       # 运行时配置
├── README.md              # 项目主文档
├── CLAUDE.md              # Claude 开发指南
└── PROJECT_STRUCTURE.md   # 本文档
```

---

## 📚 文档组织（五态架构）✨

### 设计理念

基于《道德经》"道生一，一生二，二生三，三生万物"的演化智慧：

```
道（核心理念）
  ↓
理解态 → 实践态 → 演化态 → 协同报告
  ↓       ↓       ↓         ↓
 认知    应用    升华      记录
  ↓       ↓       ↓         ↓
        归档（历史文档）
```

### `docs/` 五态结构

```
docs/
├── README.md              # 文档中心索引（7个推荐阅读路径）
├── CHANGELOG.md           # 完整开发历史
│
├── 00-core/               # 【道】核心理念层（3个文档）
│   ├── philosophy.md          # 一分为三哲学思想
│   ├── vision.md              # 产品愿景和定位
│   └── roadmap.md             # 技术路线图
│
├── 01-understanding/      # 【理解态】认知和设计（12个文档）
│   ├── README.md              # 理解态索引
│   ├── overview.md            # 架构总览
│   ├── design/                # 设计文档集
│   │   ├── architecture.md        # 完整系统架构
│   │   ├── intent-philosophy.md   # Intent系统设计哲学
│   │   ├── error-handling.md      # 错误处理系统
│   │   ├── security.md            # 安全性分析
│   │   └── config-wizard.md       # 配置向导设计
│   ├── analysis/              # 分析文档
│   │   ├── technical-debt.md      # 技术债务清单
│   │   ├── cases-coverage.md      # 50个用例覆盖分析
│   │   ├── python-rust-gap.md     # Python/Rust对比
│   │   └── dao.md                 # DAO模式分析
│   └── thinking/              # 思考笔记
│       ├── dsl-philosophy.md      # DSL设计哲学
│       ├── realconsole-dsl.md     # RealConsole DSL设计
│       └── fctoken-research.md    # FCToken框架研究
│
├── 02-practice/           # 【实践态】使用和开发（20个文档）
│   ├── README.md              # 实践态索引
│   ├── user/                  # 用户实践（6个）
│   │   ├── quickstart.md          # 5分钟快速开始
│   │   ├── user-guide.md          # 完整用户手册
│   │   ├── tool-calling-guide.md  # 工具调用指南
│   │   ├── intent-dsl-guide.md    # Intent DSL使用
│   │   ├── llm-setup.md           # LLM配置指南
│   │   └── env-config.md          # 环境变量配置
│   ├── developer/             # 开发者实践（3个）
│   │   ├── developer-guide.md     # 开发者指南
│   │   ├── tool-development.md    # 创建自定义工具
│   │   └── api-reference.md       # API参考
│   ├── use-cases/             # 使用场景（5个 + README）
│   │   ├── README.md
│   │   ├── basic-10-cases.md      # 10个基础场景
│   │   ├── basic-20-cases.md      # 20个基础场景
│   │   ├── advanced-30-cases.md   # 30个进阶场景
│   │   ├── expert-50-cases.md     # 50个专家场景
│   │   └── selected-cases.md      # 精选案例
│   └── examples/              # 示例代码
│
├── 03-evolution/          # 【演化态】进展和历史（14个文档）
│   ├── README.md              # 演化态索引
│   ├── phases/                # 阶段总结（4个）
│   │   ├── phase3-intent-dsl.md   # Phase 3完整记录
│   │   ├── phase3-summary.md      # Phase 3精简总结
│   │   ├── phase5-pipeline.md     # Phase 5 Pipeline DSL
│   │   └── phase7-polish.md       # Phase 7最终打磨
│   ├── features/              # 功能实现（8个）
│   │   ├── summary.md             # 功能总览
│   │   ├── shell-execution.md     # Shell执行系统
│   │   ├── streaming.md           # LLM流式输出
│   │   ├── git-assistant.md       # Git智能助手
│   │   ├── log-analyzer.md        # 日志分析器
│   │   ├── system-monitor.md      # 系统监控
│   │   ├── lazy-mode.md           # 懒人模式
│   │   └── config-wizard.md       # 配置向导
│   └── milestones/            # 里程碑（2个）
│       ├── 2026-q1-plan.md        # 2026 Q1行动计划
│       └── next-phase.md          # 下一阶段规划
│
├── 04-reports/            # 【协同报告】工作成果
│   ├── README.md              # 报告索引和规范
│   └── reorganization-2025-10-16.md  # 文档重组报告
│
└── archive/               # 【归档】历史文档（226个）
    ├── README.md              # 归档说明
    ├── old-progress/          # 详细开发记录（48个）
    │   ├── weekly/                # 周总结
    │   ├── daily/                 # 日总结
    │   └── bugfixes/              # Bug修复记录
    ├── old-designs/           # 过时设计文档（32个）
    └── _old-structure/        # 旧目录结构完整备份
        ├── archived/
        ├── design/
        ├── features/
        ├── guides/
        └── ...
```

### 文档分类原则

| 态 | 目录 | 定位 | 典型文档 | 更新频率 |
|---|------|------|---------|---------|
| **道** | `00-core/` | 永恒的指导思想 | philosophy.md, vision.md | 低（季度级） |
| **理解态** | `01-understanding/` | 深入理解系统本质 | overview.md, design/* | 中（月度级） |
| **实践态** | `02-practice/` | 实际操作指南 | user/*, developer/* | 高（周度级） |
| **演化态** | `03-evolution/` | 发展历程和未来 | phases/*, features/* | 持续（每次发布） |
| **协同报告** | `04-reports/` | 决策和分析记录 | reorganization-*.md | 不定期 |
| **归档** | `archive/` | 历史文档保留 | old-progress/* | 只增不减 |

### 命名规范

1. **数字前缀**: `00-core`, `01-understanding`, `02-practice`, `03-evolution`, `04-reports`
2. **全小写连字符**: `tool-calling-guide.md`, `user-guide.md`
3. **语义化**: 文件名清晰表达内容
4. **报告格式**: `{类型}-{日期}.md`（如 `reorganization-2025-10-16.md`）

---

## 🧪 测试组织

### `tests/` 集成测试

```
tests/
├── test_intent_*.rs       # Intent DSL 集成测试
├── test_function_calling_e2e.rs  # Tool Calling E2E
├── test_git_*.rs          # Git 智能助手测试
├── test_project_*.rs      # 项目上下文测试
├── test_log_*.rs          # 日志分析测试
├── test_system_*.rs       # 系统监控测试
└── ...
```

### `scripts/test/` 测试脚本

```
scripts/test/
├── test_cd.sh             # CD 命令测试
├── test_visual.sh         # 视觉效果测试
├── test_wizard_quick.sh   # 配置向导测试
├── test_log_perf.sh       # 日志性能测试
└── ...
```

**使用**：
```bash
# 运行集成测试
cargo test

# 运行测试脚本
./scripts/test/test_cd.sh
./scripts/test/test_visual.sh
./scripts/test/test_wizard_quick.sh
```

---

## 🔧 开发工具

### 性能分析

**Flamegraph**（生成火焰图）：
```bash
cargo install flamegraph
cargo flamegraph --bench intent_matching

# 输出：flamegraph/flamegraph.svg
```

**Benchmark**（性能基准测试）：
```bash
cargo bench

# 输出：target/criterion/
```

### 测试覆盖率

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html

# 输出：coverage/html/index.html
```

### 内存数据

**记忆系统数据**：
```
memory/
├── session.jsonl          # 短期记忆（环形缓冲区）
└── long_memory.jsonl      # 长期记忆（持久化）
```

---

## 📦 构建产物

### 编译输出

```
target/
├── debug/                 # Debug 构建
│   └── realconsole
├── release/               # Release 构建
│   └── realconsole
└── criterion/             # Benchmark 结果
```

### 开发沙箱

```
sandbox/
└── ...                    # 临时测试文件（git ignore）
```

---

## 🚀 快速开始

### 开发流程

```bash
# 1. 克隆代码
git clone https://github.com/hongxin/realconsole
cd realconsole

# 2. 配置向导（快速模式）
cargo run --release -- wizard --quick

# 3. 开发模式运行
cargo run

# 4. 运行测试
cargo test

# 5. 构建 Release
cargo build --release
```

### 项目维护

```bash
# 清理构建产物
cargo clean

# 更新依赖
cargo update

# 检查代码质量
cargo clippy

# 格式化代码
cargo fmt

# 生成文档
cargo doc --open

# 测试覆盖率
cargo llvm-cov --html
```

---

## 📝 Git 工作流

### 提交规范

```bash
# 功能开发
git checkout -b feature/your-feature
git commit -m "feat: Add your feature"
git push origin feature/your-feature

# Bug 修复
git checkout -b fix/your-fix
git commit -m "fix: Fix your bug"
git push origin fix/your-fix

# 文档更新
git commit -m "docs: Update documentation"

# 重构
git commit -m "refactor: Refactor code"
```

### .gitignore 规则

**忽略的目录/文件**：
- `/target/` - 编译产物
- `/coverage/` - 覆盖率报告
- `/flamegraph/` - 性能分析
- `/memory/` - 运行时数据
- `/sandbox/` - 开发沙箱
- `.env` - 敏感配置
- `*.local.yaml` - 本地配置
- `.DS_Store` - macOS 系统文件

---

## 📖 文档索引

### 核心文档

- [README.md](README.md) - 项目介绍和快速开始
- [CLAUDE.md](CLAUDE.md) - Claude 开发指南（含文档构建规则）
- [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - 本文档

### 文档中心

- [docs/README.md](docs/README.md) - 完整文档导航和推荐路径

### 核心理念（00-core）

- [一分为三哲学](docs/00-core/philosophy.md)
- [产品愿景](docs/00-core/vision.md)
- [技术路线图](docs/00-core/roadmap.md)

### 用户文档（02-practice/user）

- [快速开始](docs/02-practice/user/quickstart.md) - 5分钟上手
- [用户手册](docs/02-practice/user/user-guide.md) - 完整功能说明
- [工具调用指南](docs/02-practice/user/tool-calling-guide.md)
- [Intent DSL 指南](docs/02-practice/user/intent-dsl-guide.md)
- [LLM 配置](docs/02-practice/user/llm-setup.md)

### 开发文档（01-understanding & 02-practice/developer）

- [架构总览](docs/01-understanding/overview.md)
- [开发者指南](docs/02-practice/developer/developer-guide.md)
- [工具开发](docs/02-practice/developer/tool-development.md)
- [API 参考](docs/02-practice/developer/api-reference.md)

### 功能文档（03-evolution/features）

- [Git 智能助手](docs/03-evolution/features/git-assistant.md)
- [日志分析器](docs/03-evolution/features/log-analyzer.md)
- [系统监控](docs/03-evolution/features/system-monitor.md)
- [功能总览](docs/03-evolution/features/summary.md)

### 进度与报告

- [开发历史](docs/CHANGELOG.md) - 完整变更日志
- [阶段总结](docs/03-evolution/phases/) - Phase 3-7完整记录
- [协同报告](docs/04-reports/) - 重要决策和分析

---

## 🎯 目录整理历史

### v0.6.0 文档架构升级（2025-10-16）

**重大变更**：实施基于"一分为三"哲学的五态文档架构

**变更详情**：
1. ✅ 创建五态目录结构
   - `00-core/` - 核心理念（3个文档）
   - `01-understanding/` - 理解态（12个文档）
   - `02-practice/` - 实践态（20个文档）
   - `03-evolution/` - 演化态（14个文档）
   - `04-reports/` - 协同报告（新增）
   - `archive/` - 历史归档（226个文档）

2. ✅ 文档迁移与重组
   - 迁移50个核心文档到新结构
   - 归档226个历史文档
   - 创建8个README导航索引

3. ✅ 命名规范统一
   - 数字前缀（00-04）
   - 全小写连字符
   - 语义化清晰

4. ✅ 文档规则固化
   - 写入 CLAUDE.md（5条规则）
   - 协同报告归置规范
   - 极简主义维护原则

**成果统计**：
- 主目录：14个 → 5个（减少64%）
- 活跃文档：120+ → 50个（减少58%）
- README索引：3个 → 8个（增加167%）
- 归档文档：226个（100%保留）

**详细报告**：[docs/04-reports/reorganization-2025-10-16.md](docs/04-reports/reorganization-2025-10-16.md)

### v0.5.2 → v0.6.0 整理（2025-10-16 早期）

**日期**：2025-10-16

**变更**：
1. ✅ 创建 `docs/planning/` 目录
2. ✅ 创建 `docs/features/` 目录
3. ✅ 创建 `scripts/test/` 目录
4. ✅ 移动测试脚本：`test_*.sh` → `scripts/test/`
5. ✅ 移动规划文档到 `docs/planning/`
6. ✅ 移动功能文档到 `docs/features/`
7. ✅ 更新 `.gitignore` 添加开发产物目录

**结果**：
- 根目录文件数：从 ~24 减少到 ~16
- 为后续五态架构升级奠定基础

---

## 📊 文档统计

**活跃文档**（50个）：
- 核心文档（00-core）: 3个
- 理解态（01-understanding）: 12个
- 实践态（02-practice）: 20个
- 演化态（03-evolution）: 14个
- 协同报告（04-reports）: 持续增长
- 变更日志: 1个

**归档文档**（226个）：
- 详细进展记录: 48个
- 过时设计文档: 32个
- 旧目录结构备份: 完整保留

**导航索引**（8个README）：
- 文档中心: docs/README.md
- 五态目录: 各层README
- 专题索引: use-cases/README.md, archive/README.md, 04-reports/README.md

---

## 💡 设计原则

1. **五态分离** - 理解、实践、演化、报告各有其位
2. **数字编排** - 00-04前缀体现演化路径
3. **统一命名** - 全小写连字符，简洁语义化
4. **清晰导航** - 每层README索引
5. **精简高效** - 活跃文档≤50个，历史归档

---

**维护者**：RealConsole Team
**项目地址**：https://github.com/hongxin/realconsole
**问题反馈**：https://github.com/hongxin/realconsole/issues

---

**道生一，一生二，二生三，三生万物** ✨
