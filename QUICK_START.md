# RealConsole v1.8.3 - 5分钟快速开始

**版本**: v1.8.3 (自主学习闭环版)
**理念**: 简洁 × 强大 = 极致体验

---

## ⚡ 30秒启动

### 1. 安装（10秒）
```bash
make install
```

### 2. 设置 API Key（10秒）
```bash
export DEEPSEEK_API_KEY="your-api-key"
```

### 3. 开始使用（10秒）
```bash
realconsole
```

✅ **完成**！开始智能 CLI 之旅

---

## 🎯 核心功能体验

### 自然语言交互
```bash
# 说人话，系统懂你
> 帮我查看文件
> 创建一个 Rust 项目
> 这个错误怎么解决？
```

### 自主学习（🌊🔥 离坎炼化炉）
```bash
# 查看学习状态
/likan status

# 输出示例：
━━━━━━━━━━━━━━━━━━━━━━━━
🌊🔥 离坎炼化炉状态
━━━━━━━━━━━━━━━━━━━━━━━━
上次循环: 2分钟前
模式数量: 8
高质量: 3 ⭐
下次循环: 3分钟后
循环间隔: 5分钟
━━━━━━━━━━━━━━━━━━━━━━━━
```

**特点**:
- ✅ 自动运行，无需干预
- ✅ 持续学习你的习惯
- ✅ 建议越用越准

### 实时状态显示（可选）
```bash
# 编辑配置文件
vim ~/.realconsole/realconsole.yaml

# 启用提示符显示
likan:
  show_in_prompt: true
```

**效果**:
```bash
# 默认
(RealConsole v1) user %

# 启用后（8个模式，3个高质量）
🌊🔥 8 (3 ⭐) | (RealConsole v1) user %
```

---

## 📋 常用命令

### 系统命令（/ 前缀）
```bash
/help          # 帮助信息
/status        # 系统状态
/config        # 查看配置
/exit          # 退出
```

### 离坎炼化炉命令
```bash
/likan status   # 查看状态
/likan history  # 查看历史
/likan cycle    # 手动触发循环
```

### Shell 命令（! 前缀）
```bash
!ls -la        # 执行 shell 命令
!git status    # 直接运行 git
```

### 自然语言（无前缀）
```bash
> 帮我编译项目
> 查看最近的 git 提交
> 分析这个错误
```

---

## ⚙️ 配置选项

### 零配置启动（v1.8.4+ ✨ 新特性）

**只需环境变量，无需配置文件**:
```bash
export DEEPSEEK_API_KEY="your-api-key"
realconsole
```

系统自动检测并配置：
- ✅ LLM provider（deepseek/openai/claude）
- ✅ 工具调用
- ✅ 智能建议
- ✅ 离坎炼化炉（自主学习）

### 最小配置文件（可选）
```yaml
# ~/.realconsole/realconsole.yaml
llm:
  primary:
    api_key: ${DEEPSEEK_API_KEY}
```

✅ **仅此一行**！其他全部智能默认

**配置模板**: 参考 `config/minimal.yaml`

### 标准配置（可选）
```yaml
llm:
  primary:
    api_key: ${DEEPSEEK_API_KEY}

# 启用提示符显示
likan:
  show_in_prompt: true

# 启用语音播报（macOS）
voice:
  enabled: true
```

**配置模板**: 参考 `config/standard.yaml`

### 完整配置（高级用户）
查看 `realconsole.yaml` 了解所有选项

---

## 📚 配置层级

| 层级 | 文件 | 适用场景 |
|------|------|---------|
| **Minimal** | `config/minimal.yaml` | 新手快速上手 |
| **Standard** | `config/standard.yaml` | 日常开发使用 |
| **Full** | `realconsole.yaml` | 高级深度定制 |

**详细说明**: 参考 `config/README.md`

---

## 🎨 核心特性

### 1. 自主学习循环 ✨
```
你的使用习惯
    ↓
【离坎炼化炉】每5分钟自动运行
    ↓
Kan (☵) 提取深层模式
    ↓
Li (☲) 生成优化建议
    ↓
建议质量持续提升
```

### 2. 八维记忆空间 ✨
```
【八卦记忆宫】
  乾 ☰ - 意图目标
  坤 ☷ - 对话历史
  震 ☳ - 命令执行
  巽 ☴ - 使用趋势
  坎 ☵ - 深层模式 ⭐
  离 ☲ - 显性知识 ⭐
  艮 ☶ - 系统快照
  兑 ☱ - 用户反馈
```

### 3. 实时状态可见 ✨
```bash
🌊🔥 8 (3 ⭐) | (RealConsole v1) user %
     ↑        ↑
   总模式  高质量模式
```

---

## 💡 使用技巧

### 1. 拼写纠错
```bash
$ cargo biuld
🤖 检测到拼写错误，建议：
   cargo build
```

### 2. 智能建议
```bash
$ npm install
Error: ...

🤖 建议：
   1. rm -rf node_modules && npm install
   2. npm ci
```

### 3. 重复操作优化
```bash
# 重复 3 次后
🤖 检测到重复操作，建议创建别名
```

---

## 🚀 高级功能

### 多轮对话
```bash
> 创建一个 Rust 项目
[系统创建项目]

> 添加 tokio 依赖
[系统理解上下文，自动添加]

> 写个异步示例
[系统生成相关代码]
```

### 工具调用
```bash
> 帮我分析日志中的错误
[自动调用 log_analyzer 工具]

> 检查 git 状态并创建提交
[自动调用 git 工具链]
```

---

## 📊 性能指标

| 指标 | 数值 | 说明 |
|------|------|------|
| 启动时间 | < 100ms | 瞬间启动 |
| 内存占用 | < 50MB | 轻量级 |
| 响应延迟 | < 200ms | 实时响应 |
| 学习周期 | 5分钟 | 自动优化 |

---

## 🔧 故障排除

### 问题1: 无法启动
```bash
# 检查配置
realconsole --check-config

# 运行向导
realconsole wizard
```

### 问题2: LLM 不响应
```bash
# 检查 API key
echo $DEEPSEEK_API_KEY

# 检查网络
curl https://api.deepseek.com/v1
```

### 问题3: 炼化炉未运行
```bash
# 检查配置
/likan status

# 编辑配置
vim ~/.realconsole/realconsole.yaml

# 确保启用
likan:
  enabled: true
```

---

## 📚 更多资源

- **完整文档**: `docs/02-practice/user/user-guide.md`
- **功能展示**: `docs/SHOWCASE.md`
- **开发文档**: `docs/02-practice/developer/developer-guide.md`
- **版本说明**: `CHANGELOG-v1.8.3.md`

---

## 🎯 下一步

### 探索功能
- [ ] 尝试自然语言交互
- [ ] 查看炼化炉学习状态
- [ ] 启用提示符显示
- [ ] 体验多轮对话

### 定制配置
- [ ] 调整通知模式
- [ ] 配置语音播报
- [ ] 自定义快捷命令

### 深入学习
- [ ] 阅读完整文档
- [ ] 了解八卦哲学
- [ ] 参与社区讨论

---

**理念**: 极简交互 × 强大智能 × 自主进化

**愿景**: 最好的 CLI Agent

**状态**: Production Ready ✅

🚀 **开始你的智能 CLI 之旅！**
