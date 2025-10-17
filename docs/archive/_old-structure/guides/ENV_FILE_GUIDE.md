# .env 文件使用指南

## 简介

RealConsole 支持通过 `.env` 文件管理敏感配置（如 API keys），无需每次手动设置环境变量。

**优势：**
- ✅ 配置持久化 - 无需每次 `export`
- ✅ 安全存储 - `.env` 已在 `.gitignore` 中
- ✅ 方便管理 - 所有密钥集中管理
- ✅ 团队协作 - 通过 `.env.example` 分享配置模板

## 快速开始

### 方法 1：从示例文件创建（推荐）

```bash
# 1. 复制示例文件
cp .env.example .env

# 2. 编辑 .env 文件，填入真实的 API key
vim .env
# 或
nano .env
```

**`.env` 内容示例：**
```bash
# Deepseek API Key
DEEPSEEK_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 可选：自定义端点
DEEPSEEK_ENDPOINT=https://api.deepseek.com/v1
```

### 方法 2：手动创建

```bash
# 创建 .env 文件
cat > .env << EOF
DEEPSEEK_API_KEY=sk-your-actual-api-key-here
EOF

# 确保文件权限安全
chmod 600 .env
```

### 方法 3：使用 echo（一行命令）

```bash
echo "DEEPSEEK_API_KEY=sk-your-key" > .env
```

## 配置文件中使用

创建 `.env` 后，在 `realconsole.yaml` 中引用：

```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}  # 自动从 .env 读取
```

## 工作原理

1. RealConsole 启动时，自动查找配置文件所在目录的 `.env`
2. 解析 `.env` 文件，加载环境变量
3. **不覆盖已有环境变量**（优先级：命令行 > .env）
4. 配置文件中的 `${VAR}` 会被替换为环境变量的值

**加载顺序：**
```
启动 RealConsole
  ↓
加载 .env 文件（如果存在）
  ↓
加载配置文件（realconsole.yaml）
  ↓
扩展环境变量（${VAR}）
  ↓
初始化 LLM 客户端
```

## 示例

### 完整的 Deepseek 配置

**`.env` 文件：**
```bash
# Deepseek API Key
DEEPSEEK_API_KEY=sk-abc123def456ghi789jkl

# 可选配置
DEEPSEEK_ENDPOINT=https://api.deepseek.com/v1
HTTPS_PROXY=http://127.0.0.1:7890
```

**`realconsole.yaml` 文件：**
```yaml
prefix: "/"

llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: ${DEEPSEEK_ENDPOINT:-https://api.deepseek.com/v1}
    api_key: ${DEEPSEEK_API_KEY}

features:
  shell_enabled: true
  shell_timeout: 10
```

**运行：**
```bash
# 无需手动 export，直接运行
./target/debug/realconsole --config realconsole.yaml
```

**输出：**
```
✓ 已加载 .env: ./.env
已加载配置: realconsole.yaml
✓ Primary LLM: deepseek-chat (deepseek)
```

### 多个 API keys

**`.env` 文件：**
```bash
# 多个 LLM 提供商
DEEPSEEK_API_KEY=sk-deepseek-key
OPENAI_API_KEY=sk-openai-key
OLLAMA_ENDPOINT=http://192.168.1.100:11434
```

**`realconsole.yaml` 文件：**
```yaml
llm:
  primary:
    provider: deepseek
    api_key: ${DEEPSEEK_API_KEY}
  fallback:
    provider: ollama
    endpoint: ${OLLAMA_ENDPOINT}
```

### 代理设置

**`.env` 文件：**
```bash
DEEPSEEK_API_KEY=sk-your-key

# 代理配置
HTTPS_PROXY=http://127.0.0.1:7890
HTTP_PROXY=http://127.0.0.1:7890
```

## 支持的格式

`.env` 文件支持以下格式：

```bash
# 1. 基本格式
KEY=value

# 2. 带引号（单引号或双引号）
KEY="value with spaces"
KEY='another value'

# 3. 空值
KEY=

# 4. 注释（以 # 开头）
# This is a comment
KEY=value  # inline comment 也支持

# 5. 空行
KEY1=value1

KEY2=value2

# 6. export 格式（会自动忽略 export 关键字）
export KEY=value
```

**示例：**
```bash
# RealConsole API Keys
DEEPSEEK_API_KEY=sk-abc123

# Optional settings
DEEPSEEK_ENDPOINT="https://api.deepseek.com/v1"

# Proxy (uncomment if needed)
# HTTPS_PROXY=http://127.0.0.1:7890

export RUST_LOG=info
```

## 安全最佳实践

### ✅ 应该做的

1. **将 .env 添加到 .gitignore**
   ```bash
   # 检查是否已忽略
   git check-ignore .env
   # 输出：.env （说明已忽略）
   ```

2. **设置文件权限**
   ```bash
   chmod 600 .env  # 只有所有者可读写
   ```

3. **使用 .env.example 分享配置模板**
   ```bash
   # 不包含真实密钥
   cat > .env.example << EOF
   DEEPSEEK_API_KEY=sk-your-api-key-here
   OPENAI_API_KEY=sk-your-openai-key-here
   EOF

   git add .env.example
   ```

4. **定期轮换 API keys**
   ```bash
   # 更新 .env
   vim .env
   # 修改 DEEPSEEK_API_KEY
   ```

### ❌ 不应该做的

1. **不要提交 .env 到 Git**
   ```bash
   # 检查是否意外提交
   git status | grep .env
   # 应该没有输出
   ```

2. **不要在公开场所分享 .env**
   - 不要截图包含 .env 内容
   - 不要复制粘贴到聊天/论坛
   - 不要发送到云笔记（如果不加密）

3. **不要使用弱权限**
   ```bash
   # 不要这样做
   chmod 777 .env  # ❌ 所有人可读
   ```

## 验证配置

### 检查 .env 是否生效

```bash
# 方法 1：查看启动输出
./target/debug/realconsole --config realconsole.yaml
# 应该看到：✓ 已加载 .env: ./.env

# 方法 2：测试 LLM 连接
./target/debug/realconsole --config realconsole.yaml --once "/llm diag primary"

# 方法 3：调试模式查看环境变量
RUST_LOG=debug ./target/debug/realconsole --config realconsole.yaml
```

### 故障排查

**问题 1：.env 未加载**
```bash
# 检查文件是否存在
ls -la .env

# 检查文件位置（应该和 realconsole.yaml 在同一目录）
ls -la realconsole.yaml .env

# 检查文件内容
cat .env
```

**问题 2：API key 无效**
```bash
# 检查环境变量是否正确设置
echo $DEEPSEEK_API_KEY

# 手动测试 API
curl https://api.deepseek.com/v1/models \
  -H "Authorization: Bearer $(grep DEEPSEEK_API_KEY .env | cut -d= -f2)"
```

**问题 3：优先级问题**
```bash
# .env 不会覆盖已有环境变量
export DEEPSEEK_API_KEY=old-key  # 已设置
# .env 中的 DEEPSEEK_API_KEY=new-key 会被忽略

# 解决：取消已设置的环境变量
unset DEEPSEEK_API_KEY
```

## 与 Python 版本的差异

| 特性 | Python 版本 | Rust 版本 |
|------|------------|-----------|
| .env 支持 | ✅ | ✅ |
| 自动加载 | ✅ | ✅ |
| 不覆盖已有 | ✅ | ✅ |
| 位置 | 配置文件同目录 | 配置文件同目录 |
| 解析错误处理 | 忽略，不影响主流程 | 忽略，不影响主流程 |
| 引号支持 | ✅ | ✅ |
| export 支持 | ✅ | ✅ |

**完全兼容！** 🎉

## 团队协作

### 配置分享流程

1. **创建 .env.example**
   ```bash
   # 包含所有必需的配置项，但不包含真实值
   cat > .env.example << EOF
   # Required
   DEEPSEEK_API_KEY=sk-your-api-key-here

   # Optional
   DEEPSEEK_ENDPOINT=https://api.deepseek.com/v1
   HTTPS_PROXY=http://127.0.0.1:7890
   EOF
   ```

2. **提交到 Git**
   ```bash
   git add .env.example
   git commit -m "docs: add .env.example for configuration"
   git push
   ```

3. **团队成员使用**
   ```bash
   # 克隆仓库
   git clone ...
   cd realconsole

   # 复制并配置
   cp .env.example .env
   vim .env  # 填入真实 API key
   ```

### README 提示

在 README.md 中添加：

```markdown
## 配置

1. 复制 .env 示例文件：
   ```bash
   cp .env.example .env
   ```

2. 编辑 .env，填入你的 API keys：
   ```bash
   vim .env
   ```

3. 运行：
   ```bash
   cargo run
   ```

**重要：** 不要提交 .env 到 Git！
```

## 高级用法

### 多环境配置

```bash
# 开发环境
cp .env.example .env.dev
# 编辑 .env.dev

# 生产环境
cp .env.example .env.prod
# 编辑 .env.prod

# 使用时指定
cp .env.dev .env  # 开发
cp .env.prod .env # 生产
```

### 条件加载

```bash
# 只在特定条件下加载 .env
if [ ! -f .env ]; then
  echo "DEEPSEEK_API_KEY=sk-default" > .env
fi
```

### 与 Docker 结合

```dockerfile
# Dockerfile
FROM rust:latest

WORKDIR /app
COPY . .

# 从构建参数注入环境变量
ARG DEEPSEEK_API_KEY
RUN echo "DEEPSEEK_API_KEY=${DEEPSEEK_API_KEY}" > .env

RUN cargo build --release
CMD ["./target/release/realconsole"]
```

## 总结

**.env 文件的优势：**
- ✅ 无需每次 `export`
- ✅ 配置持久化
- ✅ 安全存储（已在 .gitignore）
- ✅ 团队协作友好（通过 .env.example）
- ✅ 与 Python 版本完全兼容

**快速开始：**
```bash
# 1. 复制示例
cp .env.example .env

# 2. 填入 API key
vim .env

# 3. 运行
cargo run -- --config realconsole.yaml
```

**文档链接：**
- [LLM 配置指南](LLM_SETUP_GUIDE.md)
- [Deepseek 验证](DEEPSEEK_VERIFICATION.md)
- [快速开始](QUICKSTART.md)

祝使用愉快！🚀
