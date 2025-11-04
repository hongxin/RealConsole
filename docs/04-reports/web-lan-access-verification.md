# Web 终端局域网访问验证报告

> 📅 **日期**: 2025-11-04
> 📌 **版本**: v1.23.0
> 🔍 **目的**: 验证 RealConsole Web 终端在局域网环境中的可用性

---

## 📋 测试概览

### 测试环境
- **操作系统**: macOS (Darwin 25.0.0)
- **本机 IP**: 192.168.3.120
- **测试端口**: 7788
- **网络环境**: 局域网 (192.168.3.0/24)

### 测试项目
✅ 本地访问 (127.0.0.1)
✅ 局域网访问 (0.0.0.0 绑定)
✅ 命令行参数覆盖
✅ 配置文件支持
✅ CORS 设置验证

---

## 🔧 配置说明

### 1. 默认配置（仅本地访问）

**代码位置**: `src/config/settings.rs:1078-1080`

```rust
fn default_web_bind() -> String {
    "127.0.0.1".to_string()  // ✅ 安全默认值
}

fn default_web_port() -> u16 {
    7788
}
```

**启动方式**:
```bash
./target/release/realconsole web
```

**访问地址**:
- ✅ http://127.0.0.1:7788 (本机)
- ❌ http://192.168.3.120:7788 (局域网) - **不可访问**

**适用场景**:
- 个人开发测试
- 不需要远程访问
- 最安全的配置

---

### 2. 局域网访问配置

#### 方式 A: 命令行参数（推荐）

**代码位置**: `src/main.rs:247-252`

```rust
// 命令行参数覆盖配置
if let Some(bind_addr) = bind {
    config.web.bind = bind_addr;
}
if let Some(port_num) = port {
    config.web.port = port_num;
}
```

**启动方式**:
```bash
# 绑定到所有网络接口
./target/release/realconsole web --bind 0.0.0.0

# 自定义端口
./target/release/realconsole web --bind 0.0.0.0 --port 8080

# 绑定到特定 IP
./target/release/realconsole web --bind 192.168.3.120
```

**访问地址**:
- ✅ http://127.0.0.1:7788 (本机)
- ✅ http://192.168.3.120:7788 (局域网)
- ✅ http://<局域网内其他设备>:7788

---

#### 方式 B: 配置文件

**文件**: `realconsole.yaml`

```yaml
web:
  enabled: true
  bind: "0.0.0.0"      # 局域网访问
  port: 7788
  allowed_origins:
    - "*"               # 允许所有源（已配置）
```

**启动方式**:
```bash
./target/release/realconsole web
```

---

## 🧪 测试步骤

### 使用测试脚本（推荐）

```bash
# 运行交互式测试脚本
./scripts/test/test_web_lan.sh

# 选项 1: 测试本地访问
# 选项 2: 测试局域网访问
```

### 手动测试

#### 步骤 1: 启动服务（局域网模式）

```bash
./target/release/realconsole web --bind 0.0.0.0
```

**预期输出**:
```
🌐 RealConsole Web 终端启动
   地址: http://0.0.0.0:7788
   提示: 按 Ctrl+C 停止服务
```

#### 步骤 2: 本机测试

打开浏览器访问:
```
http://127.0.0.1:7788
```

✅ **预期**: 正常显示 Web 终端界面

#### 步骤 3: 局域网测试

从同一局域网内的其他设备（手机、平板、另一台电脑）访问:
```
http://192.168.3.120:7788
```

✅ **预期**: 正常显示 Web 终端界面

#### 步骤 4: 功能测试

在浏览器终端中测试:
```bash
# 系统命令
/help
/stats

# Shell 命令
!ls
!pwd

# LLM 对话（需要配置 API Key）
你好
```

---

## 🔒 安全注意事项

### ⚠️ 重要警告

1. **无身份验证**
   - 当前版本没有密码保护
   - 任何能访问 URL 的人都可以使用

2. **命令执行权限**
   - Web 终端具有完整的 Shell 执行能力
   - 与启动进程的用户权限相同

3. **敏感数据暴露**
   - LLM API Key 在配置文件中
   - 命令历史可能包含敏感信息

### ✅ 安全建议

**DO (推荐做法)**:
- ✅ 仅在受信任的局域网中使用
- ✅ 使用防火墙限制访问 IP
- ✅ 定期检查连接日志
- ✅ 使用完毕后立即停止服务
- ✅ 考虑使用特定 IP 绑定而非 0.0.0.0

**DON'T (禁止事项)**:
- ❌ 不要在公网暴露服务
- ❌ 不要在生产环境使用
- ❌ 不要使用端口转发到公网
- ❌ 不要在不信任的网络中使用

---

## 🔥 防火墙配置

### macOS

**检查防火墙状态**:
```bash
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
```

**允许 realconsole**:
1. 系统偏好设置 > 安全性与隐私 > 防火墙
2. 点击"防火墙选项"
3. 添加 realconsole 到允许列表

**临时允许端口**:
```bash
# 不推荐，macOS 使用应用级防火墙
```

### Linux (iptables)

```bash
# 允许端口 7788
sudo iptables -A INPUT -p tcp --dport 7788 -j ACCEPT

# 仅允许特定 IP 段
sudo iptables -A INPUT -p tcp --dport 7788 -s 192.168.3.0/24 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 7788 -j DROP
```

### Linux (firewalld)

```bash
# 允许端口
sudo firewall-cmd --permanent --add-port=7788/tcp
sudo firewall-cmd --reload
```

---

## 📊 测试结果

### 配置验证 ✅

| 配置项 | 状态 | 说明 |
|--------|------|------|
| 默认 bind | ✅ | 127.0.0.1（安全默认值） |
| 默认 port | ✅ | 7788 |
| CORS 配置 | ✅ | 允许所有源 (*) |
| 命令行参数 | ✅ | --bind 和 --port 工作正常 |
| 配置文件支持 | ✅ | realconsole.yaml web 部分 |

### 访问测试 ✅

| 测试场景 | bind 地址 | 本机访问 | 局域网访问 | 状态 |
|---------|----------|---------|-----------|------|
| 默认配置 | 127.0.0.1 | ✅ | ❌ | ✅ 符合预期 |
| 局域网模式 | 0.0.0.0 | ✅ | ✅ | ✅ 符合预期 |
| 特定 IP | 192.168.3.120 | ✅ | ✅ | ✅ 符合预期 |

### 功能测试 ✅

| 功能 | 状态 | 备注 |
|------|------|------|
| WebSocket 连接 | ✅ | 正常建立 |
| 系统命令 | ✅ | /help, /stats 等 |
| Shell 命令 | ✅ | !ls, !pwd 等 |
| LLM 对话 | ✅ | 需要配置 API Key |
| 历史命令 | ✅ | 上/下箭头浏览 |
| 飞轮动画 | ✅ | 橙色旋转 + 模型名 |
| 中文输入 | ✅ | UTF-8 支持 |
| 提示符 | ✅ | % 符号 |

---

## 🎯 验证结论

### ✅ 局域网访问能力：**完全支持**

RealConsole Web 终端已具备完整的局域网访问能力：

1. **灵活配置**
   - ✅ 支持命令行参数 --bind 和 --port
   - ✅ 支持配置文件 realconsole.yaml
   - ✅ 参数优先级正确（命令行 > 配置文件 > 默认值）

2. **网络绑定**
   - ✅ 支持 127.0.0.1（仅本地）
   - ✅ 支持 0.0.0.0（所有网络接口）
   - ✅ 支持特定 IP 绑定

3. **CORS 配置**
   - ✅ 已配置允许所有源
   - ✅ 支持跨域访问

4. **安全设计**
   - ✅ 默认仅本地访问（安全默认值）
   - ⚠️ 需要用户显式开启局域网访问

### 📝 建议

1. **即时可用**
   - 当前实现已经可以在局域网中使用
   - 无需额外代码修改

2. **使用方式**
   - 推荐使用命令行参数：`realconsole web --bind 0.0.0.0`
   - 快速测试使用脚本：`./scripts/test/test_web_lan.sh`

3. **未来改进**
   - 考虑添加身份验证（用户名/密码）
   - 考虑支持 HTTPS/TLS
   - 添加 IP 白名单功能
   - 添加访问日志记录

---

## 📚 相关文档

- [Web 终端用户指南](../02-practice/user/web-terminal.md)
- [配置说明](../02-practice/user/configuration.md)
- [安全建议](../02-practice/user/security.md)

---

## 🔗 快速链接

**测试脚本**:
```bash
./scripts/test/test_web_lan.sh
```

**启动命令**:
```bash
# 本地访问
./target/release/realconsole web

# 局域网访问
./target/release/realconsole web --bind 0.0.0.0
```

**访问地址** (局域网模式):
- 本机: http://127.0.0.1:7788
- 局域网: http://192.168.3.120:7788

---

**验证完成** ✅
**版本**: v1.23.0
**日期**: 2025-11-04
