# RealConsole - 简洁强大的智能 CLI Agent

**理念**: 极简交互 × 强大智能 × 自主进化

---

## ⚡ 一分钟体验

### 安装（10秒）
```bash
make install
```

### 配置（10秒）
```bash
realconsole wizard
```

### 开始使用（40秒）
```bash
realconsole

# 自然语言交互
> 帮我查看当前目录的文件大小
> 创建一个 Rust 项目
> 这个错误怎么解决？
```

✅ **零学习成本** - 说人话，系统懂你

---

## 🎯 核心亮点

### 1. 极简提示符，状态一目了然

```bash
# 普通提示符
(RealConsole v1) user RealConsole %

# 智能状态显示（可选）
🌊🔥 8 (3 ⭐) | (RealConsole v1) user RealConsole %
     ↑        ↑
   总模式  高质量模式
```

**特点**:
- 不占空间
- 实时更新
- 可完全禁用

---

### 2. 自主学习，越用越懂你

```text
你的使用习惯
    ↓
【离坎炼化炉】自动运行
    ↓
提取行为模式
    ↓
优化建议质量
    ↓
越用越准确
```

**示例**:
```bash
# 第1次：普通建议
$ cargo b
建议：cargo build

# 第10次（系统学习后）：
$ cargo b
建议：cargo build --release  ← 你最常用的
```

**关键**:
- ✅ 完全自动
- ✅ 无需配置
- ✅ 后台运行
- ✅ 用户无感

---

### 3. 八维记忆，深度理解上下文

```text
【八卦记忆宫】
  乾 ☰ - 记住你的意图
  坤 ☷ - 记住对话历史
  震 ☳ - 记住执行命令
  巽 ☴ - 记住使用趋势
  坎 ☵ - 提取深层模式  ⭐
  离 ☲ - 生成显性知识 ⭐
  艮 ☶ - 保存系统快照
  兑 ☱ - 学习用户反馈
```

**效果**:
```bash
# 系统理解项目上下文
$ 测试一下
🤖 基于当前 Rust 项目，建议：
   cargo test --lib

# 系统理解历史习惯
$ 部署
🤖 基于你的使用习惯，建议：
   ./deploy.sh --production  ← 你上次用的
```

---

### 4. 智能建议，主动帮助

```bash
# 拼写错误？自动修正
$ cargo biuld
🤖 检测到拼写错误，建议：
   cargo build

# 命令失败？智能分析
$ npm install
Error: Cannot find module...

🤖 检测到依赖问题，建议：
   1. rm -rf node_modules && npm install
   2. npm ci
   3. 检查 package.json

# 重复操作？智能优化
$ git add . && git commit -m "fix" && git push
（重复3次后）

🤖 检测到重复操作，建议创建别名：
   alias gcp='git add . && git commit -m "$1" && git push'
```

---

## 🔥 实战场景

### 场景1：新手友好

```bash
# 新手不懂命令？说人话
> 我想部署这个项目

🤖 理解了，这是一个 Rust 项目，建议：
   1. 先编译：cargo build --release
   2. 运行测试：cargo test
   3. 部署脚本：./deploy.sh

   选择哪一个？[1/2/3]
```

### 场景2：老手高效

```bash
# 老手配置后，一键执行
$ rc deploy
→ cargo build --release
→ cargo test
→ docker build -t app:latest .
→ docker push registry/app:latest
→ kubectl apply -f k8s/
✅ 部署完成！
```

### 场景3：团队协作

```bash
# 新成员加入，系统自动学习团队习惯
$ 提交代码

🤖 基于团队规范，建议：
   1. cargo fmt          ← 团队要求格式化
   2. cargo clippy       ← 团队要求检查
   3. git commit -s      ← 团队要求签名

   一键执行全部？[Y/n]
```

---

## 📊 性能数据

| 指标 | 数值 | 说明 |
|------|------|------|
| 启动时间 | < 100ms | 瞬间启动 |
| 内存占用 | < 50MB | 轻量级 |
| 二进制大小 | 12MB | 精简高效 |
| 建议延迟 | < 200ms | 实时响应 |
| 学习周期 | 5分钟 | 自动优化 |
| 测试覆盖 | 100% | 质量保证 |

---

## 🎨 配置示例

### 极简配置（5行）
```yaml
llm:
  primary:
    provider: deepseek
    api_key: ${DEEPSEEK_API_KEY}
```

✅ 其他全部智能默认

### 完整配置（可选）
```yaml
likan:
  enabled: true              # 启用自主学习
  cycle_interval_secs: 300   # 5分钟学习一次
  notification_mode: prompt  # 在提示符显示状态
  show_in_prompt: true       # 实时状态可见
  min_confidence: 0.6        # 模式置信度阈值
  min_frequency: 3           # 最小使用次数
```

---

## 🚀 进阶功能

### 1. 多模态输入（规划中）
```bash
# 语音 + 文本
$ realconsole --voice
🎤 说出你的命令...

[你]："编译这个项目"
[系统]："收到，正在执行 cargo build..."
```

### 2. 多 Agent 协作（规划中）
```text
你的问题
  ↓
【Plan Agent】分解任务
  ↓
【Debug Agent】诊断错误
  ↓
【Code Agent】生成修复
  ↓
【Shell Agent】执行验证
  ↓
【Learn Agent】记录经验
```

### 3. 知识图谱（规划中）
```text
Error → Solution → Context → Pattern → Knowledge
   ↓         ↓          ↓         ↓          ↓
  自动关联 → 智能推理 → 深度理解 → 主动建议
```

---

## 💡 设计哲学

### 简洁 (Simplicity)
> "大道至简" - 老子

- 极简界面
- 零配置启动
- 自然语言交互

### 强大 (Power)
> "上善若水" - 老子

- 自主学习
- 深度理解
- 主动进化

### 平衡 (Balance)
> "离坎相济，水火既济" - 易经

- 显性知识（离☲）
- 隐性模式（坎☵）
- 动态平衡

---

## 📚 更多资源

- **快速开始**: `docs/02-practice/user/quickstart.md`
- **用户指南**: `docs/02-practice/user/user-guide.md`
- **开发文档**: `docs/02-practice/developer/developer-guide.md`
- **哲学理念**: `docs/00-core/philosophy.md`

---

## 🎯 立即体验

```bash
# 1. 安装
make install

# 2. 配置
realconsole wizard

# 3. 开始使用
realconsole

# 4. 查看学习状态
/likan status
```

---

**理念**: Less is More, Simple but Powerful
**愿景**: 最好的 CLI Agent
**状态**: Production Ready ✅

🚀 **让我们一起，让 CLI 更智能！**
