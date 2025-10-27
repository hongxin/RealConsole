# RealConsole 配置模板

**理念**: Less Config, More Magic ✨

---

## 📚 三层配置体系

### 1. Minimal（最小配置）- 推荐新手

**文件**: `config/minimal.yaml`

**内容**: 只需 API key
```yaml
llm:
  primary:
    api_key: ${DEEPSEEK_API_KEY}
```

**适用场景**:
- ✅ 第一次使用 RealConsole
- ✅ 快速上手，不想复杂配置
- ✅ 信任智能默认值

**优势**:
- 30秒启动
- 零学习成本
- 97% 配置项智能默认

---

### 2. Standard（标准配置）- 推荐日常使用

**文件**: `config/standard.yaml`

**内容**: 常用功能配置
```yaml
llm: ...
features: ...
memory: ...
display: ...
```

**适用场景**:
- ✅ 日常开发工作
- ✅ 需要调整部分功能
- ✅ 团队协作配置统一

**优势**:
- 清晰的注释说明
- 常用功能都覆盖
- 易于维护和分享

---

### 3. Full（完整配置）- 推荐高级用户

**文件**: `../realconsole.yaml`（根目录）

**内容**: 所有配置选项
```yaml
llm: ...
features: ...
memory: ...
display: ...
likan: ...
voice: ...
# ... 所有配置项
```

**适用场景**:
- ✅ 高级用户深度定制
- ✅ 需要精细控制所有参数
- ✅ 特殊场景和企业部署

**优势**:
- 完全控制
- 所有功能可配置
- 适合复杂场景

---

## 🚀 快速开始

### 方案1: 最小配置（推荐）

```bash
# 1. 复制最小配置模板
cp config/minimal.yaml ~/.realconsole/realconsole.yaml

# 2. 设置 API key
export DEEPSEEK_API_KEY="your-api-key"

# 3. 启动
realconsole
```

### 方案2: 使用配置向导

```bash
realconsole wizard
```

### 方案3: 手动配置

```bash
# 根据需求选择模板
cp config/minimal.yaml ~/.realconsole/realconsole.yaml     # 最小
cp config/standard.yaml ~/.realconsole/realconsole.yaml    # 标准
cp realconsole.yaml ~/.realconsole/realconsole.yaml        # 完整

# 编辑配置
vim ~/.realconsole/realconsole.yaml
```

---

## 💡 配置选择建议

| 用户类型 | 推荐配置 | 理由 |
|---------|---------|------|
| 新手 | minimal.yaml | 快速上手 |
| 开发者 | standard.yaml | 功能完整 |
| 团队 | standard.yaml | 统一规范 |
| 高级用户 | realconsole.yaml | 深度定制 |

---

## 🎯 智能默认值（minimal 配置时自动生效）

### LLM 配置
- **自动检测**: 根据环境变量自动选择 provider
  - `DEEPSEEK_API_KEY` → deepseek
  - `OPENAI_API_KEY` → openai
  - `ANTHROPIC_API_KEY` → claude
- **默认模型**: 各 provider 的推荐模型
- **默认端点**: 官方 API 端点

### 功能配置
- **工具调用**: 启用（tool_calling_enabled: true）
- **智能建议**: 启用（auto_suggest: true）
- **Shell 执行**: 启用，10秒超时
- **工具迭代**: 最多 30 轮，每轮 5 个工具

### 离坎炼化炉（自主学习）
- **状态**: 启用（enabled: true）
- **循环间隔**: 5 分钟（300 秒）
- **通知模式**: 最小化（minimal）
- **提示符显示**: 关闭（show_in_prompt: false）
- **置信度阈值**: 0.6
- **最小频率**: 3 次

### 记忆系统
- **容量**: 30 条
- **持久化**: 自动保存到 memory/session.jsonl
- **自动保存**: 启用

### 显示配置
- **模式**: minimal（极简）
- **语言**: zh-CN（中文）
- **颜色**: 启用
- **Emoji**: 关闭（避免兼容性问题）

---

## 🔧 配置验证

RealConsole 会自动验证并修复配置问题：

### 自动修复
- ✅ LLM 配置缺失 → 环境变量检测
- ✅ 炼化炉间隔过短（< 60秒）→ 调整为 60 秒
- ✅ 内存容量过小（< 10）→ 调整为 10
- ✅ notification_mode=prompt 但 show_in_prompt=false → 自动启用

### 友好提示
- ⚠️ API key 未配置 → 提示设置环境变量
- ⚠️ 网络连接失败 → 提示检查端点
- ⚠️ 存储不可写 → 提示检查权限

---

## 📖 更多资源

- **快速开始**: `../QUICK_START.md`
- **用户指南**: `../docs/02-practice/user/user-guide.md`
- **配置说明**: `../docs/02-practice/user/likan-config-guide.md`
- **功能展示**: `../docs/SHOWCASE.md`

---

**更新日期**: 2025-10-27
**版本**: v1.8.4+
**理念**: 简洁 × 强大 = 极致体验
