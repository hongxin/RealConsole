# 离坎炼化炉 - 配置指南

**版本**: v1.8.2+
**功能**: 自主学习循环系统

---

## 🌊🔥 什么是离坎炼化炉？

离坎炼化炉是 RealConsole 的自主学习系统，基于易经离坎二卦：
- **坎 ☵（水）**: 向下汇聚，从交互历史中提取深层模式
- **离 ☲（火）**: 向上挥发，输出优化的智能建议

系统自动运行，持续优化建议质量，无需人工干预。

---

## ⚙️ 配置选项

在 `realconsole.yaml` 中配置：

```yaml
likan:
  # 基础配置
  enabled: true                    # 是否启用炼化炉（默认 true）
  cycle_interval_secs: 300         # 循环间隔/秒（默认 300 = 5分钟）

  # 通知模式
  notification_mode: minimal       # 通知方式（见下文）
  show_in_prompt: false            # 是否在提示符显示（默认 false）

  # 模式提取阈值
  min_confidence: 0.6              # 最小置信度（0.0-1.0，默认 0.6）
  min_frequency: 3                 # 最小频率/次（默认 3）
  max_patterns: 50                 # 最大模式数量（默认 50）
```

---

## 📢 通知模式

### 1. minimal（默认，推荐）
简洁一行通知，不干扰使用：
```
🌊🔥 炼化完成: 8 模式 (3 ⭐)
```

**适用场景**: 日常使用，想知道系统状态但不希望被打扰

### 2. none（静默）
完全静默，只能通过命令查询：
```yaml
notification_mode: none
```

**适用场景**: 完全后台运行，通过 `/likan status` 主动查询

### 3. prompt（✨ v1.8.3+ 已实现）
在提示符中显示状态，持续可见：
```bash
# 无模式时（默认提示符）
(RealConsole v1) user RealConsole %

# 有模式时（8个模式，3个高质量）
🌊🔥 8 (3 ⭐) | (RealConsole v1) user RealConsole %
```

**配置方式**：
```yaml
notification_mode: prompt
# 或单独控制
show_in_prompt: true
```

**适用场景**: 需要实时看到状态，不想主动查询

**注意**: 无模式时不显示前缀，避免干扰

---

## 🔧 常用命令

### 查看状态
```bash
/likan status
```
显示：
- 上次循环时间
- 发现的模式数量
- 高质量模式数量
- 下次循环倒计时
- 循环间隔

### 查看历史
```bash
/likan history
```
显示最近 10 次循环记录

### 手动触发
```bash
/likan cycle
```
立即执行一次炼化循环（不等待定时触发）

---

## 💡 推荐配置

### 日常使用（默认）
```yaml
likan:
  enabled: true
  cycle_interval_secs: 300      # 5分钟
  notification_mode: minimal
```

### 安静工作
```yaml
likan:
  enabled: true
  cycle_interval_secs: 600      # 10分钟
  notification_mode: none
```

### 频繁使用
```yaml
likan:
  enabled: true
  cycle_interval_secs: 180      # 3分钟
  notification_mode: minimal
  min_frequency: 2              # 降低阈值
```

### 持续可见（✨ v1.8.3+）
```yaml
likan:
  enabled: true
  cycle_interval_secs: 300      # 5分钟
  notification_mode: prompt     # 在提示符显示
  # 或单独控制
  show_in_prompt: true
```
**适用**: 需要实时看到炼化炉状态，不想主动查询

### 完全禁用
```yaml
likan:
  enabled: false
```

---

## 🎯 参数调优

### cycle_interval_secs（循环间隔）
- **太短**（< 60s）: 资源浪费，模式变化不大
- **太长**（> 3600s）: 学习滞后，建议质量提升慢
- **推荐**: 300s（5分钟）平衡性能与效果

### min_confidence（最小置信度）
- **低阈值**（0.4-0.5）: 更多模式，可能包含噪音
- **高阈值**（0.7-0.9）: 更少模式，质量更高
- **推荐**: 0.6 平衡覆盖与质量

### min_frequency（最小频率）
- **低阈值**（1-2）: 捕获偶发模式
- **高阈值**（5+）: 只保留高频模式
- **推荐**: 3 过滤随机噪音

---

## 🔍 监控与调试

### 检查炼化炉是否运行
```bash
/likan status
```
看到 "上次循环" 不是 "等待首次触发" 即表示运行中

### 查看学习效果
```bash
/likan history
```
观察 "模式" 数量和 "⭐" 数量变化

### 强制更新
```bash
/likan cycle
```
立即触发，无需等待定时

---

## 🐛 故障排查

### 问题：没有发现任何模式
**原因**: 系统刚启动，交互历史不足
**解决**: 正常使用一段时间，积累足够数据

### 问题：通知太频繁
**解决**:
1. 增加 `cycle_interval_secs`
2. 或改为 `notification_mode: none`

### 问题：学习效果不明显
**解决**:
1. 降低 `min_confidence` 和 `min_frequency`
2. 减少 `cycle_interval_secs` 提高更新频率
3. 增加使用频率，积累更多数据

---

## 📚 相关文档

- [离坎炼化炉设计文档](../../01-understanding/design/likan-statusbar-issue.md)
- [用户指南](./user-guide.md)
- [配置文件说明](./config-guide.md)

---

**最后更新**: 2025-10-27
**作者**: RealConsole Team 🌊🔥
