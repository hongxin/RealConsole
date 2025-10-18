# Workflow Intent 系统迁移指南

**版本**: v0.10.5+workflow
**更新日期**: 2025-10-18
**目标读者**: 现有 RealConsole 用户

---

## 概述

Workflow Intent 系统是 RealConsole v0.10.5 引入的全新优化功能，可将常用的 LLM 任务流程固化为可复用的工作流模板，实现：

✅ **性能提升 40-50%** - 响应时间从 10-15秒 降至 5-8秒
✅ **成本降低 50-66%** - LLM 调用次数从 2-3次 减少到 1次
✅ **缓存命中 99.6%** - 相同查询耗时仅 0.05秒

本指南将帮助您在现有 RealConsole 安装上启用和配置 Workflow Intent 系统。

---

## 兼容性保证

**重要提示**：Workflow Intent 系统默认**关闭**，确保向后兼容。

- ✅ **零影响**: 不启用的情况下，对现有功能没有任何影响
- ✅ **可选开启**: 通过配置文件显式启用
- ✅ **渐进式**: 可以随时启用或禁用，无需重新安装
- ✅ **降级支持**: Workflow 匹配失败时自动回退到传统 Intent DSL

---

## 快速开始

### 方式一：使用配置向导（推荐）

如果您还没有配置文件，或想重新配置：

```bash
# 运行配置向导
realconsole wizard

# 在 "功能配置" 环节，选择启用 Workflow Intent 系统
# 向导会自动生成包含 workflow 配置的 realconsole.yaml
```

**Complete 模式提示**:
```
💡 Workflow Intent 系统可将常用任务模式固化为模板，提升 40-50% 性能
启用 Workflow Intent 系统？（实验性功能）[y/N]
```

选择 `y` 即可启用。

### 方式二：手动修改配置文件

如果您已有配置文件 `realconsole.yaml`，只需添加以下配置：

```yaml
# 功能配置
features:
  shell_enabled: true
  shell_timeout: 10
  tool_calling_enabled: true
  max_tool_iterations: 5
  max_tools_per_round: 3

  # ✨ 新增：启用 Workflow Intent 系统
  workflow_enabled: true
  workflow_cache_enabled: true
  workflow_cache_ttl_default: 300  # 缓存 5 分钟
```

保存文件后重启 RealConsole 即可生效。

---

## 配置详解

### 配置项说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `workflow_enabled` | boolean | `false` | 是否启用 Workflow Intent 系统 |
| `workflow_cache_enabled` | boolean | `true` | 是否启用缓存（启用 workflow 时生效） |
| `workflow_cache_ttl_default` | number | `300` | 默认缓存时间（秒） |

### 显示模式配置

Workflow 执行信息的显示受 `display.mode` 控制：

```yaml
# 显示模式配置
display:
  mode: standard  # minimal | standard | debug
```

**三种模式对比**：

| 模式 | 启动信息 | 匹配提示 | 执行统计 | 调试信息 |
|------|---------|---------|---------|---------|
| **Minimal** | ❌ | ❌ | ❌ | ❌ |
| **Standard** | ✅ 简化 | ✅ 名称 | ✅ 简化 | ❌ |
| **Debug** | ✅ 详细 | ✅ 置信度 | ✅ 详细 | ✅ |

**推荐**：初次使用时设为 `standard`，熟悉后可改为 `minimal`。

---

## 验证启用状态

### 1. 启动时查看提示

**Standard/Debug 模式**下启动 RealConsole，应该看到：

```
✓ Workflow Intent 系统已启用 4 个工作流模板
```

### 2. 测试内置工作流

试试这些内置工作流：

```bash
# 加密货币分析
> 分析 BNB 最近走势

# 输出示例（Standard 模式）：
# ⚡ crypto_analysis
# [执行工具调用和 LLM 分析...]
# ⓘ 6.8s
```

**对比传统方式**（禁用 workflow）：
- 传统方式：~14s，3 次 LLM 调用
- Workflow：~7s，1 次 LLM 调用
- 缓存命中：0.05s，0 次 LLM 调用

### 3. 查看 Debug 信息

**Debug 模式**下可看到详细统计：

```
⚡ Workflow: crypto_analysis (置信度: 0.92)
[执行...]
ⓘ 6.82s | LLM: 1 | 工具: 1 | 缓存: 未命中
```

缓存命中时：
```
⚡ crypto_analysis
ⓘ 0.05s (缓存)
```

---

## 内置工作流模板

启用后自动加载 **4 个内置工作流**：

### 1. 加密货币分析 (`crypto_analysis`)

**触发关键词**: 分析、加密货币、币、走势、投资

**示例输入**:
```
> 分析 BTC 最近走势
> BNB 投资策略
> 访问非小号分析 ETH
```

**优化效果**:
- LLM 调用: 3次 → 1次（减少 67%）
- 响应时间: 14.2s → 6.8s（提升 52%）
- 缓存 TTL: 300秒

### 2. 股票分析 (`stock_analysis`)

**触发关键词**: 股票、A股、港股、美股、行情

**示例输入**:
```
> 分析茅台股票
> 查询平安银行最新行情
```

**缓存 TTL**: 600秒（10分钟）

### 3. 天气分析 (`weather_analysis`)

**触发关键词**: 天气、气温、降雨、预报

**示例输入**:
```
> 北京今天天气
> 上海未来一周天气预报
```

**缓存 TTL**: 1800秒（30分钟）

### 4. 网站摘要 (`website_summary`)

**触发关键词**: 网站、访问、摘要、总结

**示例输入**:
```
> 访问 https://example.com 并总结
> 网站内容摘要 https://blog.example.com
```

**缓存策略**: 参数化缓存（基于 URL）

---

## 最佳实践

### 1. 何时启用 Workflow？

**推荐启用**：
- ✅ 经常执行重复性任务（如每天查询币价、天气）
- ✅ 对响应速度有要求
- ✅ 希望降低 LLM API 成本
- ✅ 任务模式相对固定

**可以不启用**：
- ❌ 任务高度个性化，很少重复
- ❌ 不在乎响应速度
- ❌ 使用本地 Ollama（成本不敏感）

### 2. 缓存策略调优

根据数据更新频率调整缓存 TTL：

```yaml
features:
  # 加密货币行情变化快，缓存 5 分钟
  workflow_cache_ttl_default: 300

  # 如果数据更新较慢（如新闻摘要），可以设置更长：
  # workflow_cache_ttl_default: 1800  # 30 分钟
```

**注意**：每个内置工作流有自己的 TTL 设置，此配置作为默认值。

### 3. 监控性能

**Standard 模式**显示每次执行耗时：
```
ⓘ 6.8s          # 首次执行
ⓘ 0.05s (缓存)  # 缓存命中
```

**Debug 模式**显示详细统计：
```
ⓘ 6.82s | LLM: 1 | 工具: 1 | 缓存: 未命中
ⓘ 0.05s | LLM: 0 | 工具: 0 | 缓存: 命中
```

通过这些信息可以评估 Workflow 的优化效果。

### 4. 渐进式迁移

1. **先体验**：启用 Workflow，使用内置模板
2. **再优化**：根据实际使用调整缓存 TTL
3. **后扩展**：自定义工作流模板（未来版本支持）

---

## 常见问题

### Q1: 启用 Workflow 后，原有功能会受影响吗？

**不会**。Workflow 系统作为优化层插入到 Agent 决策链中：

```
用户输入 → Conversation → Tool Calling → Workflow ✨ → Intent → Streaming
```

如果 Workflow 没有匹配，自动降级到传统 Intent DSL，保证 100% 兼容。

### Q2: 如何知道某个查询是否使用了 Workflow？

**Standard/Debug 模式**下会显示：

```
⚡ crypto_analysis    # 使用了 Workflow
✨ web_search         # 使用了传统 Intent
🤖 LLM 生成          # 使用了流式生成
```

不同的 emoji 标识不同的执行路径：
- ⚡ = Workflow Intent
- ✨ = Traditional Intent
- 🤖 = LLM Streaming

### Q3: Workflow 匹配失败会怎样？

Workflow 匹配失败时会自动降级：

```
1. 尝试 Workflow 匹配（置信度阈值：通常 > 0.7）
2. 失败 → 降级到 Intent DSL
3. 失败 → 降级到流式生成
```

用户不会感知到任何差异，只是性能优化未生效。

### Q4: 如何禁用 Workflow？

**方式一**：配置文件中设置为 `false`

```yaml
features:
  workflow_enabled: false
```

**方式二**：注释掉或删除该行（默认为 `false`）

```yaml
features:
  # workflow_enabled: true
```

重启 RealConsole 即可生效。

### Q5: 缓存如何失效？

缓存基于以下策略失效：

1. **时间缓存** (`TimeBased`)：超过 TTL 后自动失效
   - 例如：`crypto_analysis` 缓存 300 秒

2. **参数缓存** (`ParameterBased`)：参数变化后失效
   - 例如：`website_summary` 不同 URL 不共享缓存

3. **手动清理**：重启 RealConsole 会清空缓存

### Q6: 如何自定义工作流模板？

**当前版本** (v0.10.5)：仅支持 4 个内置模板

**未来版本**：将支持 YAML 配置文件自定义工作流，例如：

```yaml
# ~/.realconsole/workflows/my-workflow.yaml
name: my_custom_workflow
description: 我的自定义工作流
cache_ttl: 600
steps:
  - tool: http_get
    args:
      url: "{base_url}/{param}"
  - llm: "分析数据：{http_response}"
```

敬请期待后续版本更新！

---

## 故障排查

### 问题 1: 启动时没有显示 Workflow 提示

**可能原因**:
1. `workflow_enabled` 未设置为 `true`
2. `display.mode` 设置为 `minimal`

**解决方案**:
```bash
# 检查配置
cat realconsole.yaml | grep -A 3 "features:"
cat realconsole.yaml | grep -A 2 "display:"

# 确保：
# features:
#   workflow_enabled: true
# display:
#   mode: standard  # 或 debug
```

### 问题 2: Workflow 没有匹配成功

**可能原因**:
1. 输入与内置模板关键词不匹配
2. 置信度低于阈值

**解决方案**:
```bash
# 使用 Debug 模式查看匹配详情
# realconsole.yaml:
display:
  mode: debug

# 然后查看输出，应该显示：
# ⚡ Workflow: xxx (置信度: 0.xx)
```

尝试使用内置模板的精确关键词：
```
# 加密货币分析
分析 BNB 走势  ✅
查询 BNB 信息  ❌（可能不匹配）

# 股票分析
分析茅台股票   ✅
查看茅台       ❌（可能不匹配）
```

### 问题 3: 缓存没有生效

**可能原因**:
1. `workflow_cache_enabled` 未启用
2. 参数不同（参数化缓存）
3. 超过 TTL

**解决方案**:
```yaml
# 确保缓存已启用
features:
  workflow_cache_enabled: true
  workflow_cache_ttl_default: 300
```

使用 Debug 模式查看缓存状态：
```
ⓘ 6.8s | ... | 缓存: 未命中
ⓘ 0.05s | ... | 缓存: 命中  ← 第二次应该命中
```

---

## 性能对比

### 真实案例：BNB 投资分析

**测试输入**: "访问非小号网站，分析 BNB 最近走势，给我投资建议"

| 指标 | 传统方式 | Workflow | Workflow (缓存) |
|------|---------|---------|----------------|
| **LLM 调用** | 3 次 | 1 次 | 0 次 |
| **工具调用** | 1 次 | 1 次 | 0 次 |
| **响应时间** | 14.2s | 6.8s | 0.05s |
| **API 成本** | 100% | 33% | 0% |

**结论**：
- 首次执行提升 **52%**，成本降低 **67%**
- 缓存命中提升 **99.6%**，成本降低 **100%**

---

## 下一步

### 了解更多

- [Workflow 使用指南](../../../docs/04-reports/workflow-system-usage.md) - 详细使用说明
- [实现总结](../../../docs/04-reports/workflow-implementation-summary.md) - 技术细节
- [LLM 调用流程分析](../../../docs/04-reports/llm-call-flow-analysis.md) - 优化原理

### 参与开发

Workflow Intent 系统是开源的，欢迎贡献：

- 提交新的工作流模板建议
- 报告 bug 或提出改进意见
- 分享使用案例和最佳实践

**GitHub**: https://github.com/hongxin/RealConsole

---

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.10.5 | 2025-10-18 | 初始发布，支持 4 个内置工作流模板 |

---

**反馈**: 如有问题或建议，欢迎提交 GitHub Issue

**最后更新**: 2025-10-18
