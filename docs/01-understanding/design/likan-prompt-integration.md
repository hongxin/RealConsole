# 离坎炼化炉 - 提示符集成设计

**版本**: v1.8.3+
**日期**: 2025-10-27
**状态**: ✅ 已完成

---

## 🎯 目标

实现 `notification_mode: prompt` 模式，在命令行提示符中显示离坎炼化炉状态。

## 📊 效果示例

```bash
# 默认提示符
(RealConsole v1) user %

# 集成炼化炉状态后
🌊🔥 8 | (RealConsole v1) user %
```

**状态说明**：
- `🌊🔥` - 离坎符号
- `8` - 当前模式数量
- 可选：`(3⭐)` - 高质量模式数

## 🏗️ 技术方案

### 1. 扩展 REPL 提示符生成

目前 `src/repl.rs` 中的提示符是固定的。需要：

**Before**:
```rust
let prompt = format!("({}) {} % ", ...);
```

**After**:
```rust
let likan_prefix = self.get_likan_prefix().await;
let prompt = format!("{} ({}) {} % ", likan_prefix, ...);
```

### 2. 状态获取接口

在 Agent 中添加：
```rust
impl Agent {
    pub async fn get_likan_prompt_prefix(&self) -> Option<String> {
        if let Some(ref statusbar) = self.likan_statusbar {
            let status = statusbar.status();
            let s = status.read().await;

            if s.pattern_count == 0 {
                return None; // 无模式时不显示
            }

            Some(if s.high_confidence_count > 0 {
                format!("🌊🔥 {} ({} ⭐)", s.pattern_count, s.high_confidence_count)
            } else {
                format!("🌊🔥 {}", s.pattern_count)
            })
        } else {
            None
        }
    }
}
```

### 3. 配置控制

在 `realconsole.yaml`:
```yaml
likan:
  notification_mode: prompt  # 启用提示符模式
  show_in_prompt: true       # 也可以单独控制
```

## 🎨 设计原则

**极简主义**：
- 只显示最关键信息（模式数）
- 符号简洁（🌊🔥）
- 不干扰主提示符

**性能考虑**：
- 提示符生成频率高，需要缓存
- 使用 RwLock 读取，避免阻塞
- 状态更新异步，提示符同步读取

**用户体验**：
- 信息一目了然
- 不影响命令输入
- 可通过配置完全禁用

## 🔧 实施步骤

1. ✅ 在 Agent 中添加 `get_likan_prompt_prefix()` 方法（src/agent.rs:897-944）
2. ✅ 修改 REPL 提示符生成逻辑（src/repl.rs:226-230）
3. ✅ 添加配置选项（realconsole.yaml:120）
4. ✅ 测试不同模式切换（22/22 likan 测试通过）
5. ✅ 文档更新（完成）
   - 设计文档：本文档
   - 实施报告：docs/04-reports/likan-prompt-integration-completion.md
   - 用户指南：docs/02-practice/user/likan-config-guide.md

## 📝 注意事项

**兼容性**：
- 不影响现有 minimal 和 none 模式
- 默认不启用，用户需要主动配置

**性能**：
- 提示符生成是热路径，避免复杂计算
- 状态读取使用 try_read()，失败时返回默认值

**可扩展性**：
- 未来可以显示更多信息（如倒计时）
- 可以支持自定义格式

---

**设计者**: RealConsole Team
**审核者**: 待定
**下一步**: 开始实施 🚀
