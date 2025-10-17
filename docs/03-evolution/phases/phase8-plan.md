# Phase 8 开发计划

**版本**: v0.8.0
**主题**: 智能化 + 场景拓展 + 易用性提升
**时间**: 3-4 周
**状态**: 规划中

## 🎯 核心目标

从 v0.7.0 的"能用"到 v0.8.0 的"好用"，让 RealConsole 成为程序员和运维工程师的智能生产力工具。

### 三大方向（一分为三）

| 方向 | 核心价值 | 关键功能 |
|------|---------|---------|
| **🧠 智能进化** | 让 AI 更聪明 | 多轮对话、Pipeline 扩展、智能错误恢复 |
| **🛠️ 场景拓展** | 覆盖更多场景 | Docker 工具、远程服务器、更多项目类型 |
| **🎨 体验优化** | 提升易用性 | 命令历史、Tab 补全、快捷别名 |

## 📋 任务清单

### Week 1: 技术债务处理 + 基础功能

#### Day 1-2: 测试覆盖率提升（P0）
- [ ] 补充 LLM 客户端单元测试（目标：90%+ 覆盖）
- [ ] 补充 Agent 核心逻辑测试（目标：85%+ 覆盖）
- [ ] 补充工具执行器集成测试
- [ ] 补充边界测试和错误处理测试
- [ ] 生成覆盖率报告（目标：总体 80%+）

**验收标准**:
- ✅ 测试覆盖率 ≥ 80%
- ✅ 所有核心模块覆盖率 ≥ 85%
- ✅ CI 集成覆盖率检查

#### Day 3-4: LLM 客户端重构（P1）
- [ ] 提取公共 trait（LlmClient 增强）
- [ ] 实现通用 HTTP 客户端层（HttpClientBase）
- [ ] 重构 DeepseekClient（复用公共层）
- [ ] 重构 OllamaClient（复用公共层）
- [ ] 重构 OpenAIClient（复用公共层）
- [ ] 代码重复率降低到 < 30%

**验收标准**:
- ✅ 代码重复率从 70% 降低到 < 30%
- ✅ 所有 LLM 客户端测试通过
- ✅ 性能无退化

#### Day 5-7: 命令历史搜索（🔴 高优先级）
- [ ] 设计历史记录数据结构（History struct）
- [ ] 实现历史记录持久化（JSON/SQLite）
- [ ] 实现 Ctrl+R 交互式搜索
- [ ] 实现 `/history` 命令（查看历史）
- [ ] 实现 `/history search <keyword>` 命令
- [ ] 智能排序（频率 + 时间）
- [ ] 测试和文档

**验收标准**:
- ✅ Ctrl+R 搜索可用（类似 zsh）
- ✅ 历史记录持久化
- ✅ 智能排序算法实现
- ✅ 用户文档完整

**文件清单**:
- `src/history.rs` - 历史记录管理
- `src/repl.rs` - 集成 Ctrl+R 支持
- `src/commands/history_cmd.rs` - /history 命令
- `docs/02-practice/user/history-guide.md` - 用户指南

---

### Week 2: 智能进化

#### Day 8-12: 多轮对话支持（🔴 高优先级）
- [ ] 设计对话上下文结构（ConversationContext）
- [ ] 实现状态机（FSM for conversation state）
- [ ] 参数收集和验证机制
- [ ] 超时和退出机制
- [ ] 集成到 Agent（handle_multi_turn）
- [ ] 示例场景实现（日志分析、文件操作等）
- [ ] 测试和文档

**用户场景**:
```bash
» 帮我分析 nginx 日志
→ 我需要知道日志文件路径，请提供完整路径
» /var/log/nginx/access.log
→ [分析中...] 发现 23 个错误，最高频的是...
» 只看过去 1 小时的
→ [重新分析...] 过去 1 小时有 5 个错误...
```

**验收标准**:
- ✅ 多轮对话状态管理
- ✅ 参数收集和验证
- ✅ 3+ 场景测试通过
- ✅ 超时机制可用

**文件清单**:
- `src/conversation.rs` - 对话上下文管理
- `src/agent.rs` - 集成多轮对话
- `docs/02-practice/user/multi-turn-guide.md` - 用户指南

#### Day 13-15: LLM Pipeline 扩展（🔴 高优先级）
- [ ] 实现 count_files 操作
- [ ] 实现 search_content 操作（grep 增强）
- [ ] 实现 copy_files 操作
- [ ] 实现 move_files 操作
- [ ] 实现 archive_files 操作
- [ ] 更新 System Prompt
- [ ] 测试和文档

**示例**:
```bash
» 统计 src 目录下有多少个 rs 文件
🤖 LLM 生成
→ 执行: find src -name '*.rs' -type f | wc -l
结果: 56 个文件
```

**验收标准**:
- ✅ 5+ 新操作类型
- ✅ System Prompt 更新
- ✅ 安全验证完整
- ✅ 测试覆盖

**文件清单**:
- `src/dsl/intent/llm_bridge.rs` - 扩展操作类型
- `docs/03-evolution/features/pipeline-operations.md` - 操作文档

---

### Week 3: 场景拓展

#### Day 16-21: 容器工具集成（Docker）（🔴 高优先级）

##### Day 16-17: Docker 基础命令
- [ ] 实现 `/docker ps` 命令（容器列表）
- [ ] 实现 `/docker stats` 命令（资源监控）
- [ ] 实现 `/docker logs <container>` 命令
- [ ] 实现 `/docker exec <container> <cmd>` 命令
- [ ] Docker 可用性检测

**验收标准**:
- ✅ 4 个基础命令可用
- ✅ 彩色输出和状态显示
- ✅ Docker 未安装时优雅降级

##### Day 18-19: Docker 智能助手
- [ ] LLM 理解 Docker 意图
- [ ] 生成 Docker 命令建议
- [ ] 命令预览和确认
- [ ] 常用场景模板（启动 MySQL、Redis、Postgres 等）

**示例**:
```bash
» 启动一个 MySQL 容器，密码是 root123
🤖 建议命令:
docker run -d --name mysql-db \
  -e MYSQL_ROOT_PASSWORD=root123 \
  -p 3306:3306 \
  mysql:latest

执行此命令？[Y/n]
```

**验收标准**:
- ✅ 智能命令生成
- ✅ 预览和确认机制
- ✅ 5+ 常用场景模板

##### Day 20-21: Docker 监控和日志
- [ ] 实时日志流（类似 docker logs -f）
- [ ] 容器资源监控（CPU、内存）
- [ ] 容器健康检查
- [ ] 批量操作支持

**验收标准**:
- ✅ 实时日志查看
- ✅ 资源监控可视化
- ✅ 批量操作可用

**文件清单**:
- `src/commands/docker_cmd.rs` - Docker 命令实现
- `src/docker_manager.rs` - Docker 管理器
- `docs/02-practice/user/docker-guide.md` - 用户指南
- `docs/03-evolution/features/docker-integration.md` - 功能文档

---

### Week 4: 体验优化 + 远程支持

#### Day 22-26: Tab 自动补全（🔴 高优先级）

##### Day 22-23: 基础补全
- [ ] 实现 Completer trait（rustyline）
- [ ] 系统命令补全（/help, /docker, etc.）
- [ ] Shell 命令补全（基础）
- [ ] 文件路径补全

**验收标准**:
- ✅ 系统命令补全可用
- ✅ 文件路径补全可用
- ✅ 响应速度 < 100ms

##### Day 24-26: 上下文感知补全
- [ ] Git 分支补全
- [ ] Docker 容器名补全
- [ ] 项目文件补全
- [ ] 历史命令补全
- [ ] 缓存机制优化

**示例**:
```bash
» /he[Tab] → /help
» !ls ~/Doc[Tab] → !ls ~/Documents/
» /docker logs ngi[Tab] → /docker logs nginx-web
» git checkout fea[Tab] → git checkout feature/new-api
```

**验收标准**:
- ✅ 上下文感知补全
- ✅ 补全候选排序优化
- ✅ 缓存命中率 > 80%

**文件清单**:
- `src/completer.rs` - 补全器实现
- `src/repl.rs` - 集成补全
- `docs/02-practice/user/completion-guide.md` - 用户指南

#### Day 27-30: 远程服务器基础功能（🔴 高优先级）

##### Day 27-28: SSH 连接管理
- [ ] 实现 SSH 客户端（使用 ssh2 crate）
- [ ] 实现 `/ssh connect <server>` 命令
- [ ] 实现 `/ssh list` 命令
- [ ] 连接配置管理（~/.ssh/config 集成）
- [ ] 密钥认证支持

**验收标准**:
- ✅ SSH 连接可用
- ✅ 密钥认证支持
- ✅ 连接状态管理

##### Day 29-30: 远程命令执行
- [ ] 实现 `/ssh exec <command>` 命令
- [ ] 命令输出实时显示
- [ ] 错误处理和超时
- [ ] 批量操作基础（多服务器）

**示例**:
```bash
» /ssh connect prod-server
→ 连接到 prod-server (192.168.1.100)
✅ 已连接

» /ssh exec "df -h"
→ [prod-server] 执行: df -h
文件系统         容量  已用  可用  已用% 挂载点
/dev/sda1        100G   65G   35G   65% /
```

**验收标准**:
- ✅ 远程命令执行
- ✅ 实时输出显示
- ✅ 批量操作基础功能

**文件清单**:
- `src/ssh_client.rs` - SSH 客户端
- `src/commands/ssh_cmd.rs` - SSH 命令
- `docs/02-practice/user/remote-guide.md` - 用户指南
- `docs/03-evolution/features/remote-server.md` - 功能文档

---

## 🎯 里程碑

### M1: 技术债务清零（Week 1 结束）
- ✅ 测试覆盖率 ≥ 80%
- ✅ LLM 客户端重构完成（代码重复 < 30%）
- ✅ 命令历史搜索可用
- ✅ 错误消息统一化

### M2: 智能化提升（Week 2 结束）
- ✅ 多轮对话支持（3+ 场景）
- ✅ LLM Pipeline 支持 5+ 新操作类型
- ✅ 智能错误恢复（基础版）

### M3: 场景拓展（Week 3 结束）
- ✅ Docker 工具集成（完整功能）
- ✅ 容器监控和日志查看
- ✅ Docker 命令智能生成

### M4: 生产就绪（Week 4 结束）
- ✅ Tab 自动补全（系统命令 + 上下文感知）
- ✅ 远程服务器基础功能（SSH 连接、命令执行）
- ✅ 性能优化（启动时间、内存占用）
- ✅ 文档完善（用户指南 + 功能文档）

## 📊 成功指标

### 功能指标
- ✅ 15+ 新功能/增强
- ✅ 3 个核心场景完整覆盖（容器、远程、智能对话）
- ✅ 命令历史 + Tab 补全可用

### 质量指标
- ✅ 测试覆盖率 ≥ 80%
- ✅ 0 Clippy 警告
- ✅ 300+ 测试通过
- ✅ 性能无退化（启动时间、内存占用）

### 用户体验指标
- ✅ 配置向导一次成功率 > 90%
- ✅ 常用命令输入减少 30%+（历史 + 补全）
- ✅ 错误提示清晰度提升 50%+

### 文档指标
- ✅ 新功能文档完整（02-practice/user/）
- ✅ API 文档更新（02-practice/developer/）
- ✅ Phase 8 完成总结（03-evolution/phases/）

## 🚫 不做清单（保持极简）

### 短期不做
- ❌ Web 界面 - 专注 CLI 体验
- ❌ GUI 应用 - 不符合定位
- ❌ K8s 集成 - 留待 Phase 9
- ❌ CI/CD 集成 - 留待 Phase 9
- ❌ 数据库操作 - 留待 Phase 9
- ❌ 插件系统 - 过早优化

## 📝 开发规范

### 代码规范
- **格式**: 使用 `cargo fmt`
- **Lint**: 通过 `cargo clippy`（0 警告）
- **测试**: 新功能必须有测试覆盖
- **文档**: 公共 API 必须有文档注释

### 提交规范
- 遵循 Conventional Commits 规范
- 提交信息格式：`<type>(<scope>): <subject>`
- 类型：feat, fix, docs, refactor, test, chore

### PR 规范
- PR 标题清晰说明变更内容
- PR 描述包含：变更原因、主要改动、测试情况
- 代码审查通过后合并
- 确保 CI 通过

## 📚 相关文档

- **规划分析**: [docs/04-reports/phase8-planning-analysis.md](../04-reports/phase8-planning-analysis.md) - 详细分析报告
- **技术债务**: [docs/01-understanding/analysis/technical-debt.md](../../01-understanding/analysis/technical-debt.md)
- **路线图**: [docs/00-core/roadmap.md](../../00-core/roadmap.md)

## 🚀 下一步行动

1. ✅ 审查和确认本开发计划
2. ⏭️ 设置开发环境（依赖更新）
3. ⏭️ 创建 Week 1 任务分支
4. ⏭️ 开始 Day 1-2 任务：测试覆盖率提升

---

**文档版本**: v1.0
**创建日期**: 2025-10-16
**状态**: 待审批
**负责人**: RealConsole Team
