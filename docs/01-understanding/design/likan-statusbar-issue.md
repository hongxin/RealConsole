# 离坎炼化炉状态栏 - 问题分析与解决方案

**日期**: 2025-10-27
**状态**: 🔧 问题确认，临时方案已实施

---

## 🐛 问题确认

用户反馈：状态栏**没有固定在底部**，而是插入到输出流中间。

### 实际表现

```
...LLM 回复文本...
🌊🔥 [waiting] 0 patterns | next: 5m  ← 应该在底部，但在这里！
```

### 问题原因

**REPL (rustyline) 与终端控制冲突**：

```
时间线：
1. 用户输入命令
2. Agent 处理
3. LLM 流式输出到 stdout
4. [后台线程] 状态栏渲染到 stderr（移动光标到底部）
5. LLM 继续输出
6. REPL 重新渲染提示符
7. 状态栏被"埋"在输出中间
```

**核心问题**：
- rustyline 控制终端输入/输出
- crossterm 的光标控制被 rustyline 覆盖
- 两者不是为协作设计的

---

## 🔍 技术分析

### 冲突点

1. **输出流竞争**
   - rustyline 控制 stdout（用户输入+输出）
   - crossterm 控制 stderr（状态栏）
   - 但 rustyline 的重绘会影响整个终端

2. **光标位置**
   - crossterm 保存/恢复光标位置
   - 但 rustyline 也会移动光标
   - 导致光标位置不同步

3. **渲染时机**
   - 状态栏每分钟渲染
   - LLM 流式输出是异步的
   - 无法保证渲染时机

### 类似问题

其他 CLI 工具的解决方案：

1. **VS Code Terminal**
   - 完全控制终端，不使用 rustyline
   - 自己实现输入处理

2. **tmux/screen**
   - 使用虚拟终端
   - 完全控制布局

3. **htop/vim**
   - 进入全屏模式（alternate screen）
   - 完全接管终端

---

## 💡 临时解决方案（已实施）

### 当前策略

**禁用实时状态栏，改为通知模式**：

```rust
// FurnaceStatus::default()
enabled: false  // 禁用实时状态栏
```

**循环完成时输出简洁通知**：

```rust
// 仅一行，使用 stderr
eprintln!("🌊🔥 炼化完成: {} 模式 (3 ⭐)", report.patterns_found);
```

### 效果

```
(RealConsole v1) user % hello
Hello! 👋
...
(RealConsole v1) user % 今日新闻
...
🌊🔥 炼化完成: 3 模式  ← 循环完成时出现一次
(RealConsole v1) user %
```

### 优点

- ✅ 不干扰用户输入
- ✅ 不与 REPL 冲突
- ✅ 简洁明了（一行）
- ✅ 只在必要时出现

### 缺点

- ❌ 无法实时显示状态
- ❌ 无法看到倒计时
- ❌ 没有固定底部栏的专业感

---

## 🚀 未来解决方案（待实施）

### 方案 1：集成到 REPL 提示符

**原理**：将状态集成到 rustyline 的提示符中

```rust
// 自定义提示符
fn prompt(&self) -> String {
    let status = self.likan_status.read();
    format!(
        "🌊🔥 {} patterns | (RealConsole v1) user % ",
        status.pattern_count
    )
}
```

**效果**：

```
🌊🔥 8 patterns | (RealConsole v1) user % hello
```

**优点**：
- ✅ 始终可见
- ✅ 不冲突
- ✅ 简洁

**缺点**：
- ❌ 提示符太长
- ❌ 信息有限

---

### 方案 2：使用 Alternate Screen

**原理**：进入全屏模式，自己控制布局

```rust
// 进入 alternate screen
execute!(stdout, EnterAlternateScreen)?;

// 布局：
// ┌────────────────────────────┐
// │ 输出区域                   │
// │                            │
// ├────────────────────────────┤
// │ 🌊🔥 状态栏                │
// ├────────────────────────────┤
// │ > 输入区域                 │
// └────────────────────────────┘
```

**优点**：
- ✅ 完全控制
- ✅ 真正固定底部
- ✅ 专业外观

**缺点**：
- ❌ 实现复杂
- ❌ 需要重写 REPL
- ❌ 丧失 rustyline 的功能（历史、补全等）

---

### 方案 3：独立状态查询命令

**原理**：提供命令查询状态，不实时显示

```bash
(RealConsole v1) user % /likan status

🌊🔥 离坎炼化炉状态
━━━━━━━━━━━━━━━━━━━━━━━━━━━
上次循环: 2分钟前
模式总数: 8 个
高质量: 3 个 (⭐)
下次循环: 3分钟后
━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**优点**：
- ✅ 实现简单
- ✅ 不干扰
- ✅ 详细信息

**缺点**：
- ❌ 需要主动查询
- ❌ 无法一眼看到

---

### 方案 4：组合方案（推荐）

**结合多种策略**：

1. **日常使用** - 通知模式（当前）
   - 循环完成时简洁通知

2. **主动查询** - `/likan status` 命令
   - 详细状态和历史

3. **提示符集成** - 可选显示
   - 配置文件控制是否在提示符显示

```yaml
likan:
  notification_mode: minimal  # minimal, prompt, none
  show_in_prompt: false       # 是否在提示符显示
```

---

## 🔧 实施计划

### Phase 1（已完成）

- ✅ 禁用实时状态栏
- ✅ 改为通知模式
- ✅ 编译通过

### Phase 2（建议优先）✅ 已完成

- [x] 实现 `/likan status` 命令
- [x] 显示详细状态和历史
- [x] 支持 `/likan cycle` 手动触发

### Phase 3（可选）✅ 已完成

- [x] 配置文件控制
- [x] 多种通知模式（minimal/prompt/none）
- [ ] 提示符集成选项（保留未来实现）

### Phase 4（长期）

- [ ] 研究 alternate screen 方案
- [ ] 评估重写 REPL 的可行性
- [ ] 考虑 GUI 版本（如 Web UI）

---

## 📊 技术挑战总结

### 核心困境

**rustyline 是为传统 CLI 设计的，不支持复杂 UI**：

| 需求 | rustyline | 解决方案 |
|------|-----------|----------|
| 固定底部栏 | ❌ 不支持 | Alternate Screen |
| 实时更新 | ❌ 冲突 | 独立线程 + 锁 |
| 光标控制 | ❌ 冲突 | 完全接管终端 |
| 历史/补全 | ✅ 内置 | 自己实现 |

### 权衡

**简单 CLI vs 复杂 UI**：

```
简单 CLI (rustyline)
  ✅ 快速实现
  ✅ 功能完善（历史、补全、编辑）
  ❌ UI 受限

复杂 UI (自己实现)
  ✅ 完全控制
  ✅ 丰富界面
  ❌ 工作量大
  ❌ 需要重新实现基础功能
```

---

## 🎯 推荐方案

### 短期（当前）

**通知模式 + 状态查询命令**：

```bash
# 自动通知
🌊🔥 炼化完成: 8 模式 (3 ⭐)

# 手动查询
> /likan status
详细状态...
```

**理由**：
- 最小改动
- 不影响用户体验
- 提供完整功能

### 长期（可选）

如果用户需求强烈，可以考虑：

1. **TUI 模式** - 类似 htop，进入全屏
2. **Web UI** - 浏览器查看状态
3. **独立守护进程** - 在系统托盘显示

---

## 💡 用户建议

### 当前版本使用

1. **自动通知**
   - 每5分钟炼化完成时，会输出简洁通知
   - 不干扰正常使用

2. **查询状态**（待实现）
   ```bash
   /likan status  # 查看详细状态
   /likan cycle   # 手动触发循环
   /likan history # 查看循环历史
   ```

3. **调整间隔**
   - 如果通知太频繁，可以延长间隔
   - 修改 `FurnaceConfig::default().cycle_interval_secs`

---

## 🙏 总结

**实时固定底部状态栏在 rustyline 环境下技术上不可行**。

但我们提供了更实用的替代方案：
- ✅ 自动通知（不干扰）
- ✅ 状态查询命令（详细信息）
- ✅ 保持系统简洁

这符合"极简主义"和"顺势而为"的哲学。

---

**完成者**: Claude & RealConsole Team
**最后更新**: 2025-10-27

---

## 🎉 Phase 2 & 3 完成总结

### Phase 2 - 命令系统 ✅
- `/likan status` - 查看详细状态
- `/likan history` - 查看循环历史
- `/likan cycle` - 手动触发循环

### Phase 3 - 配置增强 ✅
**配置文件支持**：
```yaml
likan:
  enabled: true
  cycle_interval_secs: 300
  notification_mode: minimal  # minimal / prompt / none
  show_in_prompt: false
  min_confidence: 0.6
  min_frequency: 3
  max_patterns: 50
```

**三种通知模式**：
1. **minimal**（默认）：简洁一行通知
2. **prompt**：更新状态栏（预留）
3. **none**：静默模式，仅通过命令查询

**技术实现**：
- `NotificationMode` 枚举
- `FurnaceConfig` 扩展支持 serde
- Agent 后台循环集成配置
- 从配置文件动态加载

---

> "顺势而为，不强求完美"
> "形式服从功能，体验优先"
> "少则得，多则惑"
>
> 🌊🔥🎯
