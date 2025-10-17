# RealConsole 项目指南

**RealConsole** 是融合东方哲学智慧的智能 CLI Agent，使用 Rust 构建，集成 LLM 对话、工具调用、Shell 执行、意图识别和任务编排能力。项目遵循"一分为三"设计哲学，将易经变化智慧融入系统架构。

**项目地址**：https://github.com/hongxin/RealConsole

## 核心理念

**一分为三哲学**：超越二元对立，将状态视为向量空间中的演化路径。如命令安全不是 Safe/Dangerous 二分，而是 (Safe, NeedsConfirmation, Dangerous) 三态；Intent 匹配采用多维向量决策（confidence, risk, user_level 等）。

详见 `docs/00-core/philosophy.md`（哲学思想）、`docs/00-core/vision.md`（产品愿景）

**极简主义**：最小化依赖，核心功能优先，清晰分类，持续优化

**闭环开发**：借鉴 Claude Code 的理解-分解-执行-反思-调整循环

## 技术要点

**语言与核心库**：
- Rust 2021 (1.70+)，tokio 异步运行时
- rustyline (REPL)，serde_yaml (配置)，colored (输出)

**架构核心**：
- `Agent` 统一入口：自然语言 → LLM 流式，Shell（! 前缀），系统命令（/ 前缀）
- `LlmClient` trait：统一 LLM 接口（Deepseek/Ollama，流式输出）
- `Tool` trait：工具标准接口（14+ 内置工具，支持 Function Calling）
- `Intent DSL`：50+ 内置意图，正则+模板引擎，LRU 缓存

**关键目录**：
- `src/agent.rs` - 核心调度
- `src/llm/` - LLM 客户端
- `src/dsl/intent/` - Intent DSL
- `src/task/` - 任务编排系统
- `src/i18n.rs` - 国际化系统
- `docs/` - 五态架构文档（00-core/01-understanding/02-practice/03-evolution/04-reports）

## 开发规范

**代码风格**：
- 命名：snake_case（文件/函数）、CamelCase（类型）
- 错误：`anyhow::Result` + `thiserror`
- 异步：统一 tokio，LLM 调用必须异步
- 质量：`cargo fmt` + `cargo clippy` 零警告

**测试与配置**：
- 新功能需测试覆盖（`cargo test`）
- 主配置 `realconsole.yaml`，环境变量 `.env`（不提交）
- 测试覆盖报告放到 `coverage/` 目录

**文档组织**（五态架构）：
- 00-core：核心理念（philosophy/vision/roadmap）
- 01-understanding：设计分析
- 02-practice：用户/开发者指南
- 03-evolution：开发历程
- 04-reports：协同报告

**国际化**：
- 中文优先，英文同步
- 翻译资源：`locales/*.yaml`
- 使用：`i18n::t("key")` / `i18n::t_with_args("key", &[("param", "value")])`

## 快速操作

**构建运行**：
```bash
cargo build --release
cp .env.example .env  # 填入 DEEPSEEK_API_KEY
./target/release/realconsole
```

**测试**：
```bash
cargo test                    # 全部测试
cargo test --test test_intent_*  # Intent 测试
cargo llvm-cov --html         # 覆盖率报告
```

**扩展开发**：
- **工具**：实现 `Tool` trait，参考 `src/builtin_tools.rs`
- **Intent**：在 `src/dsl/intent/builtin.rs` 添加，遵循现有模式
- **LLM**：实现 `LlmClient` trait，参考 `src/llm/deepseek.rs`

## 文档导航

- **快速开始**：`docs/02-practice/user/quickstart.md`
- **用户手册**：`docs/02-practice/user/user-guide.md`
- **开发指南**：`docs/02-practice/developer/developer-guide.md`
- **完整索引**：`docs/README.md`

---

**最后更新**: 2025-10-17 | **许可**: MIT | **维护**: RealConsole Contributors
