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
- Rust 2021 (1.90+)，tokio 异步运行时
- rustyline (REPL)，serde_yml (配置)，colored (输出)

**架构核心**：
- `Agent` 统一入口：自然语言 → LLM 流式，Shell（! 前缀），系统命令（/ 前缀）
- `LlmClient` trait：统一 LLM 接口（Deepseek/Ollama，流式输出）
- `Tool` trait：工具标准接口（14+ 内置工具，支持 Function Calling）
- `Intent DSL`：50+ 内置意图，正则+模板引擎，LRU 缓存
- `Web Terminal` (v1.23.0)：浏览器访问，axum + WebSocket + xterm.js

**关键目录**：
- `src/agent.rs` - 核心调度
- `src/llm/` - LLM 客户端
- `src/dsl/intent/` - Intent DSL
- `src/task/` - 任务编排系统
- `src/tracer/` - 统一追踪系统（v1.5.0新增，四维观测体系）
- `src/web/` - Web 终端（v1.23.0新增，浏览器访问）
- `src/i18n.rs` - 国际化系统
- `scripts/` - Shell 脚本集合（test/ 按功能分类的测试脚本，utils/ 发布等工具）
- `docs/` - 五态架构文档（00-core/01-understanding/02-practice/03-evolution/04-reports）
- `scripts/` - 测试脚本目录

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

**文档组织**（五态体系 + 极简README）：
- **README.md / README.cn.md**：极简主页（~170行），学习 mlx 风格
  - 英文版（README.md）：面向国际用户
  - 中文版（README.cn.md）：日常开发优先更新，大版本时同步英文版
- **docs/**：详细文档体系
  - 00-core：核心理念（philosophy/vision/roadmap）
  - 01-understanding：设计分析
  - 02-practice：用户/开发者指南（详细功能说明、配置、API等）
  - 03-evolution：开发历程
  - 04-reports：开发或测试的报告
  - documentation-restructure.md：文档重组说明

**文档更新流程**：
1. 日常开发：只更新 README.cn.md（中文版）
2. 功能迭代：同步更新 docs/ 详细文档
3. 版本发布：发布前同步 README.md（英文版）
4. 大版本：审视并优化文档结构

**国际化**：
- 中文优先，英文同步
- 翻译资源：`locales/*.yaml`
- 使用：`i18n::t("key")` / `i18n::t_with_args("key", &[("param", "value")])`

**根目录管理**（极简原则）：
- **保留**：README（.md/.cn.md）、CLAUDE.md、CHANGELOG.md、LICENSE、Cargo.toml、配置文件（.env.example, realconsole.yaml）、构建脚本（Makefile, install.sh, uninstall.sh）
- **归档**：发布说明 → `docs/03-evolution/archives/vX.Y.Z-release-notes.md`，开发报告 → `docs/04-reports/`
- **清理**：临时辅助文件（COMMIT_MESSAGE*.md, GIT_COMMIT_GUIDE*.md 等）提交后立即删除
- **时机**：每次 Git 提交前检查根目录，执行清理和归档操作

## 快速操作

**安装使用**（推荐）：
```bash
# 1. 编译并安装到用户目录（~/.local/bin）
make install

# 2. 运行配置向导
realconsole wizard

# 3. 开始使用
realconsole
```

**手动构建运行**：
```bash
cargo build --release
cp .env.example .env  # 填入 DEEPSEEK_API_KEY
./target/release/realconsole
```

**Web 终端**（v1.23.0 新增）：
```bash
# 1. 配置 API Key（用于 LLM 对话）
export DEEPSEEK_API_KEY="your-api-key-here"

# 2. 启动 Web 服务（默认 http://127.0.0.1:7788）
realconsole web

# 自定义端口
realconsole web --port 9000

# 局域网访问（谨慎使用）
realconsole web --bind 0.0.0.0
```
**注意**：需要配置 LLM 才能使用对话功能，否则只能使用系统命令和 Shell 命令。

详见：`docs/02-practice/user/web-terminal.md`

**测试**：
```bash
make test                     # 全部测试（或 cargo test）
make test-intent              # Intent 测试
make coverage                 # 覆盖率报告
```

**安装位置**：
- 可执行文件：`~/.local/bin/realconsole`
- 配置目录：`~/.realconsole/`
- Memory 数据：`~/.realconsole/memory/`

**卸载**：
```bash
make uninstall                # 或 ./uninstall.sh
```

**扩展开发**：
- **工具**：实现 `Tool` trait，参考 `src/builtin_tools.rs`
- **Intent**：在 `src/dsl/intent/builtin.rs` 添加，遵循现有模式
- **LLM**：实现 `LlmClient` trait，参考 `src/llm/deepseek.rs`
- **Web**：扩展 Web 功能参考 `src/web/`（v1.23.0，axum + WebSocket）

## 文档导航

- **快速开始**：`docs/02-practice/user/quickstart.md`
- **用户手册**：`docs/02-practice/user/user-guide.md`
- **开发指南**：`docs/02-practice/developer/developer-guide.md`
- **完整索引**：`docs/README.md`

**v1.5.0 新增文档**：
- **trace 命令设计**：`docs/04-reports/trace-command-design.md`
- **四维哲学理论**：`docs/04-reports/four-dimensions-philosophy.md`
- **实施计划**：`docs/04-reports/trace-implementation-plan.md`
- **测试完成报告**：`docs/04-reports/phase-5-testing-completion.md`

---

**最后更新**: 2025-10-23 | **许可**: MIT | **维护**: RealConsole Contributors
