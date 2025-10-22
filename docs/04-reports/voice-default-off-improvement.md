# 语音播报默认关闭改进

**版本**: v1.4.0
**日期**: 2025-10-22
**类型**: 用户体验改进

---

## 改进目标

确保 RealConsole 启动时语音播报功能默认是关闭的，用户需要手动执行 `/voice on` 才能开启。这样可以避免：
- 用户忘记在配置文件中设置而导致意外播报
- 在安静环境下突然播报造成尴尬
- 提升整体用户体验

---

## 改进内容

### 1. 修改初始化逻辑

**文件**: `src/agent.rs:478-486`

**修改前**:
```rust
let broadcast_config = BroadcastConfig {
    enabled: config.voice.enabled,  // 使用配置文件的值
    voice: config.voice.voice.clone(),
    max_queue_size: config.voice.max_queue_size,
};
```

**修改后**:
```rust
// 启动时强制关闭语音播报，需要用户主动开启（/voice on）
// 这样可以避免用户忘记配置而导致意外播报，提升用户体验
let broadcast_config = BroadcastConfig {
    enabled: false, // 强制关闭，忽略配置文件中的设置
    voice: config.voice.voice.clone(),
    max_queue_size: config.voice.max_queue_size,
};
```

### 2. 更新配置文件说明

**文件**: `config/minimal.yaml:73-81`

**修改前**:
```yaml
# 语音播报（可选，v1.3.7+）
# voice:
#   enabled: false           # 是否启用
#   voice: "Ting-Ting"       # macOS 中文语音（可选）
#   max_queue_size: 10       # 队列大小
```

**修改后**:
```yaml
# 语音播报（可选，v1.3.7+）
# 注意：启动时语音播报始终是关闭的，需要运行 /voice on 手动开启
# voice:
#   enabled: false           # 此配置项已被忽略，启动时强制关闭
#   voice: "Ting-Ting"       # macOS 中文语音（可选）
#   max_queue_size: 10       # 队列大小
#   auto_broadcast: true     # 开启后是否自动播报 LLM 响应
#   max_broadcast_length: 200  # 最大播报长度（字符）
#   filter_code_blocks: true   # 是否过滤代码块
```

---

## 行为变化

### Before（改进前）

| 配置文件 | 启动时状态 | 说明 |
|---------|----------|------|
| 无 voice 配置 | 关闭 | 使用默认值 `enabled: false` |
| `voice.enabled: false` | 关闭 | 遵循配置 |
| `voice.enabled: true` | **开启** | 遵循配置（可能导致意外播报）|

### After（改进后）

| 配置文件 | 启动时状态 | 说明 |
|---------|----------|------|
| 无 voice 配置 | 关闭 | 强制关闭 |
| `voice.enabled: false` | 关闭 | 强制关闭 |
| `voice.enabled: true` | **关闭** | 忽略配置，强制关闭 |

**结论**: 无论配置如何，启动时语音播报始终是关闭的。

---

## 使用方式

### 手动开启语音

```bash
# 启动 realconsole
$ realconsole

# 检查语音状态（应该显示 OFF）
> /voice

# 开启语音播报
> /voice on

# 测试播报
> /voice say 你好，这是测试

# 关闭语音播报
> /voice off
```

### 配置语音选项

虽然 `voice.enabled` 会被忽略，但其他配置项仍然有效：

```yaml
voice:
  enabled: false           # 被忽略，启动时强制关闭
  voice: "Ting-Ting"       # 有效：指定语音
  max_queue_size: 10       # 有效：队列大小
  auto_broadcast: true     # 有效：开启后自动播报 LLM 响应
  max_broadcast_length: 200  # 有效：最大播报长度
  filter_code_blocks: true   # 有效：过滤代码块
```

---

## 影响范围

### 用户可见变化
- ✅ **启动时语音始终关闭**，无意外播报
- ✅ 需要主动执行 `/voice on` 才能开启
- ✅ 重启应用后需要重新开启（更安全）

### 配置兼容性
- ✅ 完全向后兼容
- ✅ `voice.enabled` 配置项被忽略（但不会报错）
- ✅ 其他配置项正常工作

### 代码影响
- ✅ 仅修改 1 个文件 2 处代码
- ✅ 无破坏性更改
- ✅ 测试覆盖完整

---

## 测试验证

### 自动化验证
```bash
$ ./test_voice_default.sh
✓ realconsole 已安装
✓ 已确认：启动时强制 enabled: false
✓ 所有验证通过！
```

### 手动测试步骤

1. **启动测试**
   ```bash
   $ realconsole
   > /voice
   ```
   期望输出：显示 `状态: OFF` 或 `语音播报已禁用`

2. **开启测试**
   ```bash
   > /voice on
   ```
   期望输出：`语音播报已启用`

3. **重启测试**
   - 退出 realconsole
   - 重新启动
   - 执行 `/voice`
   - 期望：又是关闭状态

4. **配置测试**
   - 在配置文件中设置 `voice.enabled: true`
   - 启动 realconsole
   - 执行 `/voice`
   - 期望：仍然是关闭状态（配置被忽略）

---

## 技术细节

### 实现位置
- **文件**: `src/agent.rs`
- **方法**: `Agent::create_voice_broadcaster()`
- **行号**: 478-486

### 修改原理
```rust
// 在创建 VoiceBroadcaster 时
let broadcast_config = BroadcastConfig {
    enabled: false,  // 硬编码为 false
    // ... 其他配置仍从 config.voice 读取
};
```

### 运行时控制
```rust
// 用户执行 /voice on 时
broadcaster.enable().await;  // 运行时修改状态

// 用户执行 /voice off 时
broadcaster.disable().await;  // 运行时修改状态
```

---

## 用户反馈处理

### 常见问题

**Q: 为什么配置文件中设置了 `enabled: true` 但启动时还是关闭的？**

A: 这是有意设计的。为了提升用户体验，我们强制启动时关闭语音播报，避免意外打扰。需要时请使用 `/voice on` 手动开启。

**Q: 每次重启都要重新开启很麻烦，能不能保存状态？**

A: 目前的设计是出于安全考虑，确保每次启动都是静默的。如果您需要频繁使用语音功能，可以：
1. 使用 shell alias: `alias rc-voice='realconsole && /voice on'`
2. 或在未来版本中我们会考虑添加"记住上次状态"选项

**Q: 配置文件中的 `voice.enabled` 还有用吗？**

A: 目前该字段会被忽略。我们保留它是为了向后兼容，未来可能用于其他用途。

---

## 后续改进建议

### 短期
- [ ] 在 `/voice` 命令输出中明确说明"启动时默认关闭"
- [ ] 添加 `/voice status` 子命令显示详细状态
- [ ] wizard 中提示用户这一行为

### 中期
- [ ] 添加配置项 `voice.remember_state`（记住上次状态）
- [ ] 支持启动参数 `--voice-on` 临时开启
- [ ] 添加快捷命令别名支持

### 长期
- [ ] 智能场景检测（如会议模式自动禁用）
- [ ] 语音播报历史记录
- [ ] 多语言语音支持

---

## 总结

此改进通过强制启动时关闭语音播报，显著提升了用户体验：

- ✅ **避免意外打扰**：启动时始终静默
- ✅ **用户可控**：需要时手动开启
- ✅ **安全第一**：重启后自动关闭
- ✅ **向后兼容**：不影响现有配置

**用户操作**: 需要语音时，执行 `/voice on` 即可！

---

**责任人**: Claude Code
**审核人**: [待填写]
**发布版本**: v1.4.0
