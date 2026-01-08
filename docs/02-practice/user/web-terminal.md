# Web 终端

中文 | **[English](web-terminal.en.md)**

> v1.23.0 新增功能，持续优化至 v1.52.0

RealConsole 支持通过浏览器访问的 Web 终端，允许在局域网中远程使用 RealConsole。

## 快速开始

### 环境准备

1. **配置 API Key**（用于 LLM 对话）

```bash
# 设置 Deepseek API Key
export DEEPSEEK_API_KEY="your-api-key-here"
```

2. **创建配置文件** `realconsole.yaml`（可选，使用默认配置也可以）

```yaml
llm:
  primary:
    provider: "deepseek"
    model: "deepseek-chat"
    api_key: "${DEEPSEEK_API_KEY}"
```

### 启动 Web 服务

#### 本地访问（默认，推荐）

```bash
# 使用默认配置启动（127.0.0.1:7788）
realconsole web

# 自定义端口
realconsole web --port 9000
```

**访问地址**: http://127.0.0.1:7788

**特点**:
- ✅ 仅本机可访问，最安全
- ✅ 适合个人开发测试
- ✅ 默认配置，无需额外参数

#### 局域网访问（谨慎使用）

```bash
# 绑定所有网络接口（允许局域网访问）
realconsole web --bind 0.0.0.0

# 绑定到特定 IP（推荐，更安全）
realconsole web --bind 192.168.1.100

# 自定义端口
realconsole web --bind 0.0.0.0 --port 9000
```

**访问地址**:
- 本机: http://127.0.0.1:7788
- 局域网: http://\<你的IP\>:7788

**获取本机 IP**:
```bash
# macOS / Linux
ifconfig | grep "inet " | grep -v 127.0.0.1

# 或使用测试脚本（会自动显示）
./scripts/test/test_web_lan.sh
```

**安全提醒** ⚠️:
- 0.0.0.0 会暴露在整个局域网
- 当前版本无身份验证
- 仅在受信任的网络中使用
- 具有完整的 Shell 执行权限

### 访问终端

#### 本地访问

启动服务后，在本机浏览器中访问：

```
http://127.0.0.1:7788
```

#### 局域网访问

如果使用 `--bind 0.0.0.0` 或 `--bind <特定IP>` 启动，可以从局域网内其他设备访问：

**步骤 1: 获取服务器 IP**
```bash
# macOS / Linux
ifconfig | grep "inet " | grep -v 127.0.0.1 | awk '{print $2}'
# 例如输出: 192.168.1.100

# 或使用测试脚本
./scripts/test/test_web_lan.sh
```

**步骤 2: 其他设备访问**

在同一局域网内的手机、平板或其他电脑上，打开浏览器访问：
```
http://<服务器IP>:7788
# 例如: http://192.168.1.100:7788
```

**步骤 3: 故障排查**

如果无法访问，检查：
1. 确认两台设备在同一局域网
2. 检查防火墙设置（见下方"防火墙配置"）
3. 尝试 ping 服务器 IP: `ping 192.168.1.100`
4. 检查服务是否正常运行

## 配置文件

在 `realconsole.yaml` 中添加 Web 配置：

```yaml
# LLM 配置（必需，用于对话功能）
llm:
  primary:
    provider: "deepseek"
    model: "deepseek-chat"
    endpoint: "https://api.deepseek.com/v1"
    api_key: "${DEEPSEEK_API_KEY}"  # 从环境变量读取

# Web 终端配置
web:
  enabled: true
  bind: "127.0.0.1"  # 默认仅本地访问
  port: 7788
  allowed_origins: ["*"]  # CORS 配置
```

**重要**：如果需要使用 LLM 对话功能，必须配置 `llm` 部分。否则只能使用系统命令和 Shell 命令。

### 配置说明

- **enabled**: 是否启用 Web 服务（默认 false）
- **bind**: 绑定地址
  - `127.0.0.1` - 仅本地访问（安全）
  - `0.0.0.0` - 允许局域网访问（谨慎使用）
  - 特定 IP - 绑定到指定网卡
- **port**: 端口号（默认 7788）
- **allowed_origins**: CORS 允许的源

## 功能特性

### 支持的操作

✅ **系统命令** - 所有以 `/` 开头的 RealConsole 命令
```
/help
/memory add "记忆内容"
/stats
```

✅ **Shell 命令** - 以 `!` 开头的系统命令
```
!ls -la
!pwd
!git status
```

✅ **LLM 对话** - 直接输入文本与 AI 交互
```
介绍一下 Rust 语言
翻译这段代码
```

✅ **数据可视化** (v1.44.0+) - 以 `!chart` 开头的图表命令
```
!chart line --title "月度销售" --x-axis "1月,2月,3月" --series "销售额:120,132,145"
!chart pie --title "市场份额" --labels "A,B,C" --series "份额:35,25,40"
!chart csv data.csv --type line --x-col "月份" --y-col "销售额"
```

> 📖 **完整可视化指南**: [visualization-guide.md](visualization-guide.md)

### 终端快捷键

- **Ctrl+C** - 中断当前操作
- **Ctrl+L** - 清屏
- **Enter** - 执行命令
- **Backspace** - 删除字符

### 界面特点

- 🎨 **现代 UI** - 渐变背景，圆角卡片，美观大方
- 📱 **响应式设计** - 支持手机、平板、桌面浏览器
- 🌙 **深色主题** - 护眼的深色终端界面
- ⚡ **实时响应** - WebSocket 实时通信，低延迟

## 架构设计

### 技术栈

- **后端**: axum + WebSocket
- **前端**: xterm.js + 原生 JavaScript
- **协议**: JSON 消息格式

### 消息协议

#### 客户端 → 服务器

```json
{
  "type": "input",
  "content": "user command"
}
```

#### 服务器 → 客户端

```json
// 普通输出
{
  "type": "output",
  "content": "response text"
}

// 错误信息
{
  "type": "error",
  "content": "error message"
}

// 清屏
{
  "type": "clear"
}
```

## 安全建议

### ⚠️ 重要提示

1. **默认仅本地访问** - `bind: 127.0.0.1` 仅允许本机访问
2. **局域网访问需谨慎** - 使用 `0.0.0.0` 会暴露在局域网
3. **无身份验证** - 当前版本不支持密码保护
4. **Shell 命令执行** - 具有系统命令执行权限

### 推荐实践

- ✅ 仅在受信任的网络中使用
- ✅ 使用防火墙限制访问
- ✅ 定期检查连接日志
- ❌ 不要在公网暴露服务
- ❌ 不要在生产环境使用

## 故障排查

### 无法访问

**本地访问失败**:
1. 检查服务是否启动
2. 检查端口是否被占用：`lsof -i :7788`
3. 尝试其他端口：`realconsole web --port 8080`
4. 检查浏览器控制台错误

**局域网访问失败**:
1. 确认使用 `--bind 0.0.0.0` 或特定 IP 启动
2. 确认两台设备在同一局域网（ping 测试）
3. 检查防火墙配置（见下方）
4. 尝试在服务器本机访问 `http://127.0.0.1:7788`
5. 检查路由器是否隔离设备（AP 隔离）

### 防火墙配置

#### macOS

**检查防火墙状态**:
```bash
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
```

**配置步骤**:
1. 系统偏好设置 → 安全性与隐私 → 防火墙
2. 点击"防火墙选项"
3. 点击 ➕ 添加应用程序
4. 选择 `realconsole` 可执行文件
5. 确保设置为"允许传入连接"

#### Linux (iptables)

```bash
# 允许端口 7788
sudo iptables -A INPUT -p tcp --dport 7788 -j ACCEPT

# 仅允许特定网段访问（更安全）
sudo iptables -A INPUT -p tcp --dport 7788 -s 192.168.1.0/24 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 7788 -j DROP

# 保存规则
sudo iptables-save > /etc/iptables/rules.v4
```

#### Linux (firewalld)

```bash
# 永久允许端口
sudo firewall-cmd --permanent --add-port=7788/tcp
sudo firewall-cmd --reload

# 仅允许特定 IP
sudo firewall-cmd --permanent --add-rich-rule='rule family="ipv4" source address="192.168.1.0/24" port port="7788" protocol="tcp" accept'
sudo firewall-cmd --reload
```

### WebSocket 连接失败

1. 浏览器控制台查看错误（F12 → Console）
2. 检查 CORS 配置（默认已允许所有源）
3. 尝试不同的浏览器（Chrome、Firefox、Safari）
4. 检查代理设置（可能阻止 WebSocket）

### 命令执行失败

1. **LLM 对话失败**: 检查 `realconsole.yaml` 中的 API Key 配置
2. **Shell 命令失败**: 检查文件权限和执行权限
3. **系统命令失败**: 查看服务器终端的错误输出

## 使用示例

### 示例 1：本地开发（推荐）

```bash
# 启动 Web 终端（默认配置）
realconsole web

# 浏览器访问
open http://localhost:7788
```

**适用场景**: 个人开发、测试、学习

### 示例 2：局域网协作

#### 快速测试（交互式）

```bash
# 使用测试脚本（推荐）
./scripts/test/test_web_lan.sh

# 选择 "2" 进入局域网模式
# 脚本会自动显示本机 IP 和访问地址
```

#### 手动启动

```bash
# 步骤 1: 获取本机 IP
ifconfig | grep "inet " | grep -v 127.0.0.1
# 假设输出: 192.168.1.100

# 步骤 2: 启动服务（绑定到所有接口）
realconsole web --bind 0.0.0.0

# 步骤 3: 其他设备访问
# 浏览器打开: http://192.168.1.100:7788
```

**适用场景**:
- 团队演示
- 移动设备访问
- 跨设备调试

### 示例 3：指定 IP 绑定（更安全）

```bash
# 仅绑定到特定 IP（推荐）
realconsole web --bind 192.168.1.100 --port 7788

# 访问地址
# 本机: http://127.0.0.1:7788
# 局域网: http://192.168.1.100:7788
```

**优势**: 比 0.0.0.0 更安全，只监听指定网卡

### 示例 4：配置文件方式

`realconsole.yaml`:
```yaml
web:
  enabled: true
  bind: "0.0.0.0"          # 局域网访问
  port: 7788
  allowed_origins:
    - "*"                   # 允许所有源（开发环境）
    # - "http://192.168.1.*"  # 仅允许局域网（生产环境）
```

```bash
# 使用配置文件启动
realconsole web
```

### 示例 5：自定义端口

```bash
# 避免端口冲突
realconsole web --bind 0.0.0.0 --port 8080

# 访问: http://192.168.1.100:8080
```

## 限制和未来计划

### 当前限制

- 不支持多用户同时登录
- 无身份验证机制
- 不支持 HTTPS
- 流式输出暂未完全实现

### 未来计划

- [ ] 添加身份验证（用户名/密码）
- [ ] 支持 HTTPS/TLS
- [ ] 完整的流式输出支持
- [ ] 会话持久化
- [ ] 多标签页支持
- [ ] 文件上传/下载功能

## 相关资源

- [用户指南](./user-guide.md)
- [配置说明](../../02-practice/user/configuration.md)
- [架构设计](../../01-understanding/web-architecture.md)

---

**版本**: v1.52.0
**更新时间**: 2026-01-08
