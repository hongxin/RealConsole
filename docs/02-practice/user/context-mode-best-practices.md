# 对话上下文模式 - 最佳实践

本指南帮助你充分利用 RealConsole 的对话上下文功能，优化使用体验和性能。

---

## 目录

- [模式选择指南](#模式选择指南)
- [使用场景](#使用场景)
- [性能优化](#性能优化)
- [配置建议](#配置建议)
- [常见问题](#常见问题)
- [高级技巧](#高级技巧)

---

## 模式选择指南

### Disabled（默认）- 何时使用

✅ **推荐场景**:
- 快速查询单个问题
- 脚本自动化调用
- 追求最快响应速度
- 最低 Token 消耗

❌ **不推荐场景**:
- 需要连续分析同一主题
- 多步骤任务（如数据分析）
- 需要 AI 记住前文信息

**示例**:
```bash
% 查看 CPU 使用率
% 统计目录大小
% 解释 Docker 命令
```

**优势**:
- ⚡ 最快响应（无需加载历史）
- 💰 Token 消耗最低
- 🎯 适合单一明确的查询

---

### Manual - 何时使用

✅ **推荐场景**:
- 专注的长期对话（如代码审查）
- 需要明确控制上下文生命周期
- 多人协作（启动后共享会话）
- 教学演示（清晰的开始/结束）

❌ **不推荐场景**:
- 临时快速查询
- 不确定是否需要上下文
- 经常忘记关闭上下文

**示例**:
```bash
% /context start
✓ 上下文已启动

% 分析 app.py 中的性能瓶颈
🤖 发现 3 个瓶颈...

% 第一个瓶颈的具体代码在哪？
🤖 在第 45 行的数据库查询...

% 给出优化建议
🤖 基于刚才分析的瓶颈，建议...

% /context stop
✓ 上下文已停止
```

**优势**:
- 🎮 完全掌控
- 📊 清晰的会话边界
- 🔍 可随时查看状态（`/context status`）

---

### Auto - 何时使用

✅ **推荐场景**:
- 日常交互式使用
- 不确定是否需要上下文
- 希望无缝的多轮对话体验
- 探索性分析任务

❌ **不推荐场景**:
- 脚本自动化
- 追求极致性能
- 需要严格控制 Token 消耗

**触发词** (20+ 个自动检测):
- 代词: 它、这个、那个、这些、那些、it、that、these...
- 追问: 为什么、怎么、继续、详细、why、how、more...
- 承接: 现在、那么、所以、因此、now、then、so...
- 回顾: 刚才、之前、上面、earlier、previous...

**示例**:
```bash
% 列出 Python 文件
🤖 当前目录有 12 个 Python 文件

% 显示它们的大小          # ✨ 自动检测到"它们"
🤖 这些文件的大小分别是...  # 自动使用上下文

% 找出最大的 3 个
🤖 根据刚才的列表...      # 继续使用上下文
```

**优势**:
- 🤖 智能自动化
- 🔄 无缝衔接
- 💡 减少认知负担

---

## 使用场景

### 场景 1: 日志分析（Manual 模式）

**任务**: 分析生产环境日志

```bash
% /context start
✓ 上下文已启动

% 分析 production.log 最近 1 小时的错误
🤖 发现 127 个错误，主要类型：
- ConnectionTimeout: 89 次
- AuthenticationError: 32 次
- DataValidationError: 6 次

% 统计每种错误的时间分布
🤖 基于刚才的分析：
ConnectionTimeout 错误主要集中在...

% 找出 ConnectionTimeout 的根本原因
🤖 通过分析刚才提到的 89 次超时...

% 给出修复建议
🤖 综合前面的分析，建议：...

% /context show   # 查看分析历史
当前上下文 (4 轮)

[轮次 1] 14:32:15
  👤 分析 production.log 最近 1 小时的错误
  🤖 发现 127 个错误，主要类型...

% /context stop
✓ 上下文已停止
统计: 已清除 4 轮对话（2341 字符）
```

**为什么使用 Manual**:
- 明确的分析会话
- 需要保持专注
- 可随时查看分析历史

---

### 场景 2: 代码探索（Auto 模式）

**任务**: 探索新项目代码库

```bash
% 这个项目的主要功能是什么
🤖 这是一个 Web 服务，主要提供...

% 入口文件在哪里          # ✨ 自动检测到"入口"（引用）
🤖 根据刚才的分析，入口在 main.py...

% 看一下它的结构         # ✨ 自动检测到"它"（代词）
🤖 main.py 的结构如下...

% API 路由定义在哪       # 继续使用上下文
🤖 基于前面的代码结构...
```

**为什么使用 Auto**:
- 探索性质，不确定需要多少轮
- 自然的对话流程
- 无需手动管理

---

### 场景 3: 快速查询（Disabled 模式）

**任务**: 快速解决单一问题

```bash
% Docker 如何挂载主机目录
🤖 使用 -v 或 --mount 参数...

% Rust 如何读取环境变量
🤖 使用 std::env::var()...

% 解释这个错误：ECONNREFUSED
🤖 ECONNREFUSED 表示连接被拒绝...
```

**为什么使用 Disabled**:
- 每个问题独立
- 最快响应
- 最低成本

---

## 性能优化

### Token 消耗

**上下文对 Token 的影响**:

| 模式 | 每次调用 Token | 适用场景 |
|------|---------------|---------|
| Disabled | 仅当前输入（~50-200） | 单次查询 |
| Manual/Auto（1轮） | 当前 + 1 轮历史（~200-500） | 简单对话 |
| Manual/Auto（5轮） | 当前 + 5 轮历史（~1000-3000） | 深入分析 |
| Manual/Auto（20轮） | 当前 + 20 轮历史（~4000-10000） | 长期对话 |

**优化建议**:

1. **合理设置 `max_turns`**:
   ```yaml
   conversation:
     max_turns: 10  # 大多数场景 5-10 轮足够
   ```

2. **使用 `max_context_length` 限制**:
   ```yaml
   conversation:
     max_context_length: 5000  # 控制总字符数
   ```

3. **及时清除不需要的上下文**:
   ```bash
   % /context clear  # 清除但保持激活
   % /context stop   # 完全停止
   ```

4. **启用自动清理**:
   ```yaml
   conversation:
     auto_clear:
       enabled: true
       idle_timeout: 300  # 5 分钟未活动自动清除
   ```

---

### 响应速度

**影响因素**:

1. **上下文大小**: 更多轮次 = 更多处理时间
2. **历史长度**: 长文本回答会增加上下文大小
3. **网络延迟**: 更多 Token 需要更多传输时间

**优化建议**:

1. **不包含不必要的内容**:
   ```yaml
   conversation:
     include:
       tool_calls: true       # 工具调用通常需要
       shell_output: false    # Shell 输出通常不需要
       errors: true           # 错误信息通常需要
   ```

2. **使用简洁的提问**:
   ```bash
   # ❌ 不好：包含大量背景
   % 我刚才运行了很多命令，包括 ls、cat、grep 等等，现在我想问...

   # ✅ 好：简洁明了
   % 统计刚才找到的错误数量
   ```

---

## 配置建议

### 日常使用（推荐）

```yaml
conversation:
  mode: auto                 # 智能检测，无缝体验
  max_turns: 10             # 10 轮足够大多数对话
  max_context_length: 5000  # 控制在 5K 字符
  auto_clear:
    enabled: true
    idle_timeout: 300       # 5 分钟自动清除
    on_task_complete: false
  include:
    tool_calls: true
    shell_output: false
    errors: true
```

**适合**: 日常交互使用，平衡功能和性能

---

### 深度分析（专家用户）

```yaml
conversation:
  mode: manual              # 手动控制，更精确
  max_turns: 20            # 允许更长对话
  max_context_length: 10000 # 更大的上下文
  auto_clear:
    enabled: true
    idle_timeout: 600       # 10 分钟
    on_task_complete: false
  include:
    tool_calls: true
    shell_output: true      # 包含 Shell 输出
    errors: true
```

**适合**: 深入的代码审查、日志分析、调试任务

---

### 性能优先（脚本/自动化）

```yaml
conversation:
  mode: disabled            # 关闭上下文
```

**适合**: 脚本调用、CI/CD 集成、自动化任务

---

### 学习模式（初学者）

```yaml
conversation:
  mode: manual              # 明确的开始/结束
  max_turns: 5             # 较小的上下文
  max_context_length: 3000
  auto_clear:
    enabled: true
    idle_timeout: 180       # 3 分钟（较短）
    on_task_complete: true  # 任务完成自动清除
  include:
    tool_calls: true
    shell_output: false
    errors: true
```

**适合**: 学习阶段，养成良好的上下文管理习惯

---

## 常见问题

### Q1: 如何知道上下文是否激活？

**A**: 查看 REPL 提示符：

```bash
# 未激活
(RealConsole v1) user RealConsole %

# 已激活
(RealConsole v1) user RealConsole [上下文: 3轮] %
```

或使用命令：
```bash
% /context status
```

---

### Q2: Auto 模式会自动清除吗？

**A**: 是的，如果启用了 `auto_clear`：

```yaml
conversation:
  auto_clear:
    enabled: true
    idle_timeout: 300  # 5 分钟未活动后清除
```

会在空闲时自动清除，提示符会显示警告：
```bash
(RealConsole v1) user RealConsole [上下文: 5轮 | 4分钟前] %
                                                  ↑ 黄色警告
```

---

### Q3: Manual 模式忘记 `stop` 会怎样？

**A**: 上下文会一直保留，直到：
1. 手动执行 `/context stop`
2. 达到 `idle_timeout` 自动清除
3. 退出 RealConsole

**建议**: 设置合理的 `idle_timeout` 作为保险。

---

### Q4: 如何查看当前上下文内容？

**A**: 使用 `/context show`：

```bash
% /context show
当前上下文 (3 轮)

[轮次 1] 14:32:15
  👤 分析 error.log
  🤖 发现 3 种错误类型...

[轮次 2] 14:33:02
  👤 统计每种错误的数量
  🤖 TypeError: 15次，ValueError: 8次...

[轮次 3] 14:34:56
  👤 找出 TypeError 的根本原因
  🤖 通过分析代码...
```

---

### Q5: 上下文会持久化吗？

**A**: 不会。上下文仅在当前会话中保存，退出 RealConsole 后丢失。

**原因**:
- 保持简洁和高效
- 避免积累过期上下文
- 每次启动都是新的开始

**未来**: 可能支持可选的上下文持久化功能。

---

### Q6: 如何在 Manual 模式和 Auto 模式之间切换？

**A**: 需要修改配置文件并重启：

```bash
# 1. 编辑配置
vim ~/.realconsole/realconsole.yaml

# 2. 修改 mode
conversation:
  mode: auto  # 或 manual

# 3. 重启 RealConsole
```

**提示**: 可以准备多个配置文件，使用环境变量切换：
```bash
export REALCONSOLE_CONFIG=~/.realconsole/config-auto.yaml
realconsole
```

---

### Q7: Token 消耗太大怎么办？

**A**: 优化建议：

1. 减少 `max_turns`:
   ```yaml
   max_turns: 5  # 从 20 减到 5
   ```

2. 减少 `max_context_length`:
   ```yaml
   max_context_length: 3000  # 从 8000 减到 3000
   ```

3. 及时清除：
   ```bash
   % /context clear  # 完成一个主题后清除
   ```

4. 切换到 Disabled 模式（单次查询）

---

## 高级技巧

### 技巧 1: 分主题管理上下文

对于多个不相关的任务，分别管理上下文：

```bash
# 主题 1: 日志分析
% /context start
% 分析 error.log
% 统计错误类型
% /context stop

# 主题 2: 代码审查（新上下文）
% /context start
% 审查 app.py
% 检查安全问题
% /context stop
```

---

### 技巧 2: 使用 `clear` 而不是 `stop`

如果想继续相同主题但清除历史：

```bash
% /context clear  # 清除但保持激活
✓ 上下文已清除
提示: 上下文仍处于激活状态

% 继续新的子任务  # 从零开始，但不需要重新 start
```

---

### 技巧 3: 监控空闲时间

利用提示符的空闲警告：

```bash
(RealConsole v1) user RealConsole [上下文: 8轮 | 4分钟前] %
                                                  ↑ 黄色警告
```

看到黄色警告时，考虑：
- 执行任意命令刷新活动时间
- 或主动 `clear`/`stop` 如果不再需要

---

### 技巧 4: 快捷检查状态

绑定快捷命令（在 shell rc 文件）：

```bash
alias rcs='realconsole --once "/context status"'
```

随时在另一个终端查看状态：
```bash
$ rcs
上下文状态
模式: Manual
状态: 🟢 激活
轮次: 5 / 20
```

---

### 技巧 5: 配置模板

创建多个配置模板：

```bash
~/.realconsole/
├── config-disabled.yaml  # 性能优先
├── config-manual.yaml    # 手动控制
├── config-auto.yaml      # 日常使用
└── config-analysis.yaml  # 深度分析
```

使用环境变量切换：
```bash
export REALCONSOLE_CONFIG=~/.realconsole/config-auto.yaml
realconsole
```

---

## 总结

### 模式选择快速决策

```
需要连续多轮对话？
├─ 否 → Disabled（默认）
└─ 是
   ├─ 需要精确控制？
   │  └─ 是 → Manual
   └─ 否 → Auto（推荐）
```

### 关键要点

1. **Disabled 是默认** - 不影响现有使用
2. **Auto 最方便** - 适合日常交互
3. **Manual 最精确** - 适合专业任务
4. **合理配置限制** - 平衡功能和性能
5. **利用 REPL 提示** - 实时了解状态
6. **及时清理上下文** - 控制 Token 消耗

---

## 相关文档

- [快速开始](quickstart.md) - 基础使用指南
- [用户手册](user-guide.md) - 完整功能说明
- [配置参考](../../03-evolution/context-mode-design.md) - 详细配置说明
- [设计理念](../../00-core/philosophy.md) - 哲学思想

---

**最后更新**: 2025-10-20
**适用版本**: RealConsole v1.3.0+
