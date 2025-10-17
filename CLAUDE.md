# RealConsole 项目指南

**RealConsole v1.1.0** 是融合东方哲学智慧的智能 CLI Agent，使用 Rust 构建，集成 LLM 对话、工具调用、Shell 执行、意图识别和任务编排能力。项目遵循"一分为三"设计哲学，将易经变化智慧融入系统架构，在二元对立之外寻找演化路径。

**项目地址**：https://github.com/hongxin/RealConsole

## 本系统构建的核心理念

我希望在本系统的开发实现，在坚持“极简主义”开发理念的同时，要强调看问题能够一分为三。

如道德经所云，道生一，一生二，二生三，三生万物。

我对此段话的理解在于：
- 世间万物运行的背后存在着规律，所谓道；
- 从而构建一个整体，谓之一；
- 在整体中的千千万万，我们可以用阴阳演化来看待，现有两端，可以把握，谓之二；
- 然而目前很多的系统设计太过于刚性，往往采用一分为二的判断法则，非此即彼，所以需要不断的在二分法中艰难选择，需要不断打补丁，然后世间大道，在于把握两端的二之后，还有中间状态需要按照易经中演化的8卦到64卦在到384爻的变化，来观察，谓之三，这样看问题可以更全面；
- 那么有了这种柔性合理的三分观察，世间万物才能应运而生，包括我们系统中的变化，谓之三生万物。


## 快速理解

这是一个命令行智能助手，用户可以直接输入自然语言对话，系统通过 Intent DSL 理解意图，由 LLM（Deepseek/Ollama）提供智能，必要时调用内置工具（计算器、文件操作、时间等）完成任务。Shell 命令通过 `!` 前缀安全执行，所有操作都有黑名单保护和超时控制。v1.0.0 新增任务编排系统，支持 `/plan` 智能分解复杂目标为可执行子任务，自动分析依赖关系并优化并行执行，通过 `/execute` 一键完成整体任务。

## 核心技术栈

**语言**: Rust 2021 edition (1.70+)
**异步运行时**: tokio (全特性)
**LLM 集成**: Deepseek API、Ollama 本地模型，支持流式输出（SSE）
**配置管理**: YAML (serde_yaml) + 环境变量 (dotenvy)
**交互界面**: rustyline (REPL)、colored (彩色输出)
**工具系统**: 自研 Tool trait + OpenAI Function Calling Schema
**Intent DSL**: 正则+模板引擎，50+内置意图，LRU 缓存优化
**任务编排系统**: LLM驱动的任务分解 + Kahn拓扑排序 + 并行优化执行
**错误修复系统**: 12种错误模式 + 反馈学习 + 三层安全防护
**国际化系统**: 多语言支持（中文优先、英文同步），基于YAML的轻量级i18n架构
**测试覆盖**: 645个测试，100% 通过率

## 架构核心

项目围绕 `Agent` 核心展开，接收用户输入后分三路处理：自然语言对话（LLM 流式）、Shell 执行（! 前缀）、系统命令（/ 前缀）。Agent 持有 LLM Manager（primary/fallback 架构）、Tool Registry、Memory、Execution Logger 等组件。

**目录结构**：
- `src/agent.rs` - 核心调度逻辑
- `src/llm/` - Deepseek/Ollama 客户端，流式输出实现
- `src/dsl/intent/` - Intent DSL（匹配器、模板、实体提取）
- `src/dsl/type_system/` - 类型系统（预留，暂未启用）
- `src/tool*.rs` - 工具注册与执行引擎
- `src/task/` - 任务编排系统（分解、规划、执行）
- `src/commands/task_cmd.rs` - 任务命令接口（/plan、/execute等）
- `src/memory.rs` - 短期+长期记忆系统
- `src/shell_executor.rs` - Shell 命令安全执行
- `docs` - 项目的技术文档

**关键抽象**：
- `LlmClient` trait：统一 LLM 接口（chat、chat_stream、支持工具调用）
- `Tool` trait：工具标准接口（name、execute、to_openai_schema）
- `Intent` struct：意图描述（关键词、模板、参数、领域）
- `Agent::handle` 方法：统一入口，根据前缀分发

## 设计哲学

项目遵循"一分为三"思想，这不是简单地在二元对立间加入中间态，而是将状态视为向量空间中的演化路径。命令安全评估不是 Safe/Dangerous 二分，而是 (Safe, NeedsConfirmation, Dangerous) 三态，更进一步是多维向量（confidence, risk, user_level 等）。易经64卦的变化规律启发了 Intent 匹配的多维决策，错综互卦的多重视角体现在错误恢复的多路径尝试。

阅读 `docs/00-core/philosophy.md` 理解核心哲学，`docs/00-core/vision.md` 了解产品愿景。

同时，充分借鉴Claude Code这类LLM CLI软件工具中的核心成功经验：**Claude Code 成功的核心** 理解-分解-执行-反思-调整的闭环

## 开发约定

**配置**: 主配置 `realconsole.yaml`，环境变量 `.env`（不提交），最小配置示例 `config/minimal.yaml`
**测试**: 使用 `cargo test`，新功能需要测试覆盖
**格式**: `cargo fmt` + `cargo clippy` 无警告
**命名**: 使用 snake_case（文件、函数）、CamelCase（类型）
**错误**: 使用 `anyhow::Result` 和 `thiserror` 定义错误类型
**异步**: 统一使用 tokio，LLM 调用必须异步
**依赖**: 定期更新，避免引入未维护的 crate（如 meval 已替换为 evalexpr）

**文档构建规则**（基于"一分为三"哲学的五态架构）：
1. **目录结构**: 00-core（道/核心理念）、01-understanding（理解态/设计分析）、02-practice（实践态/指南用例）、03-evolution（演化态/进展规划）、04-reports（报告/协同工作）、archive（归档/历史）
2. **命名规范**: 数字前缀体现演化路径，全小写连字符（如user-guide.md），语义化清晰
3. **索引导航**: 每层目录需有README索引，活跃文档精简（≤50个），过期文档归档到archive/
4. **协同报告**: 所有协同工作产生的分析、总结、决策报告统一存放到04-reports/，便于追溯
5. **维护原则**: 遵循极简主义，清晰分类、易于导航、持续优化

**多语言支持规则**（Phase 11）：
1. **语言优先级**: 中文优先，英文同步。所有文档默认中文，大版本更新时同步英文版本
2. **程序界面**:
   - 命令行通过 `--lang` 参数指定语言（支持 zh-CN, en-US）
   - 配置文件中通过 `display.language` 字段指定
   - 语言回退机制：命令行 > 配置文件 > 环境变量(REALCONSOLE_LANG) > 系统语言 > 默认中文
3. **翻译资源**:
   - 位于 `locales/` 目录，使用 YAML 格式
   - 键名采用点分层级（如 `welcome.app_name`）
   - 支持参数化翻译（如 `{version}`）
4. **添加新字符串**:
   - 使用 `i18n::t("key")` 获取翻译
   - 使用 `i18n::t_with_args("key", &[("param", "value")])` 进行参数替换
   - 同时更新 `locales/zh-CN.yaml` 和 `locales/en-US.yaml`
5. **设计哲学**: 遵循"一分为三"——明确态（已知语言）、演化态（可扩展）、容错态（回退机制）

**Git 工作流**：功能分支开发，PR 前确保测试通过，提交信息清晰说明变更原因

测试覆盖计量的相关内容、文档和结果，都统一放到 coverage 子目录进行规整管理。

## 重要路径

**快速上手**:
```bash
cargo build --release
cp .env.example .env  # 填入 DEEPSEEK_API_KEY
./target/release/realconsole
```

**运行测试**:
```bash
cargo test                              # 全部测试
cargo test --test test_intent_*         # Intent DSL 测试
cargo llvm-cov --html                   # 覆盖率报告
```

**工具开发**: 参考 `src/builtin_tools.rs`，实现 `Tool` trait，注册到 `Agent::new` 中的 tool_registry
**Intent 扩展**: 在 `src/dsl/intent/builtin.rs` 添加新意图，遵循现有模式
**LLM 集成**: 实现 `LlmClient` trait，参考 `src/llm/deepseek.rs`

## 文档导航

**文档中心**: `docs/README.md` - 基于三态架构的完整文档导航

**核心文档**（00-core）：
- `docs/00-core/philosophy.md` - 一分为三哲学思想
- `docs/00-core/vision.md` - 产品愿景和定位
- `docs/00-core/roadmap.md` - 技术路线图

**理解态**（01-understanding）：
- `docs/01-understanding/overview.md` - 架构总览
- `docs/01-understanding/design/` - 设计文档集
- `docs/01-understanding/analysis/` - 分析文档

**实践态**（02-practice）：
- `docs/02-practice/user/quickstart.md` - 5分钟快速开始
- `docs/02-practice/user/user-guide.md` - 完整用户手册
- `docs/02-practice/developer/developer-guide.md` - 开发者指南
- `docs/02-practice/developer/tool-development.md` - 创建自定义工具

**演化态**（03-evolution）：
- `docs/03-evolution/phases/` - 各阶段开发总结
- `docs/03-evolution/features/` - 功能实现文档
- `docs/CHANGELOG.md` - 完整开发历史

**协同报告**（04-reports）：
- `docs/04-reports/` - 协同工作产生的分析、总结、决策报告

**核心代码**: `src/agent.rs` (184行)、`src/tool_executor.rs` (297行)、`src/dsl/intent/matcher.rs` (1400+行)

## 当前状态

**版本**: 1.1.0 "International"
**阶段**: Phase 11 完成（国际化支持 + 多语言界面）
**测试**: 645个测试通过（新增i18n测试）
**最新改进**:
  - ✅ 国际化系统：中英文双语支持，遵循"一分为三"设计哲学
  - ✅ 灵活的语言选择：命令行、配置文件、环境变量三层回退
  - ✅ 轻量级架构：基于YAML的翻译资源，支持参数化字符串
  - ✅ 完整的主程序和REPL界面翻译
  - ✅ 语言自动推断：从系统环境智能选择默认语言
**里程碑**: v1.1.0 标志着 RealConsole 支持国际化，可服务全球用户
**已知问题**: Ollama 客户端存在兼容性问题（配置中暂时禁用）
**技术债**: 见 `docs/01-understanding/analysis/technical-debt.md`

---

**最后更新**: 2025-10-17
**许可**: MIT
**维护**: RealConsole Contributors
