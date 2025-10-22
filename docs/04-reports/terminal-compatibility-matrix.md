# 终端兼容性测试矩阵

**版本**: v1.4.0
**更新日期**: 2025-10-22
**测试平台**: macOS, Linux

---

## 测试方法

### 自动化测试
```bash
./tests/terminal_compatibility.sh
```

### 手动测试
1. 启动 `realconsole`
2. 执行以下命令：
   ```
   /context
   /context start
   /context status
   /context show
   /context stop
   ```
3. 观察：
   - 终端是否崩溃
   - 显示是否正常
   - 性能是否流畅

---

## 兼容性矩阵

### macOS

| 终端 | 版本 | 颜色 | Unicode | Emoji | Context | 稳定性 | 推荐 | 备注 |
|------|------|------|---------|-------|---------|--------|------|------|
| **iTerm2** | 3.5+ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⭐⭐⭐ | 最佳选择，emoji 部分支持 |
| **Terminal.app** | 2.14+ | ✅ | ✅ | ❌ | ⚠️ | ⚠️ | ⭐ | emoji 可能导致崩溃 |
| **Alacritty** | 0.13+ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⭐⭐⭐ | GPU 加速，性能最佳 |
| **WezTerm** | 20240203+ | ✅ | ✅ | ✅ | ✅ | ✅ | ⭐⭐⭐ | 全功能支持 |
| **Kitty** | 0.35+ | ✅ | ✅ | ✅ | ✅ | ✅ | ⭐⭐ | 图形协议丰富 |
| **Hyper** | 3.4+ | ✅ | ✅ | ⚠️ | ✅ | ⚠️ | ⭐ | 基于 Electron，较慢 |

### Linux

| 终端 | 版本 | 颜色 | Unicode | Emoji | Context | 稳定性 | 推荐 | 备注 |
|------|------|------|---------|-------|---------|--------|------|------|
| **Alacritty** | 0.13+ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⭐⭐⭐ | 推荐首选 |
| **Gnome Terminal** | 3.50+ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⭐⭐ | GNOME 默认 |
| **Konsole** | 24.02+ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⭐⭐ | KDE 默认 |
| **Tilix** | 1.9+ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⭐⭐ | 分屏功能强 |
| **st (suckless)** | 0.9+ | ✅ | ✅ | ❌ | ✅ | ✅ | ⭐ | 极简主义 |
| **xterm** | 388+ | ✅ | ⚠️ | ❌ | ✅ | ✅ | ⭐ | 传统终端 |

### Windows

| 终端 | 版本 | 颜色 | Unicode | Emoji | Context | 稳定性 | 推荐 | 备注 |
|------|------|------|---------|-------|---------|--------|------|------|
| **Windows Terminal** | 1.20+ | ✅ | ✅ | ✅ | ✅ | ✅ | ⭐⭐⭐ | 官方推荐 |
| **Alacritty** | 0.13+ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⭐⭐⭐ | 跨平台一致 |
| **ConEmu** | 231119+ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⭐⭐ | 功能丰富 |
| **cmd.exe** | - | ⚠️ | ⚠️ | ❌ | ⚠️ | ✅ | ❌ | 不推荐 |
| **PowerShell** | 7.4+ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⭐ | 基本支持 |

---

## 功能测试项

### 1. 颜色支持
- ✅ 完全支持：256 色或真彩色
- ⚠️ 部分支持：仅支持 8/16 色
- ❌ 不支持：无颜色

### 2. Unicode 支持
- ✅ 完全支持：中文、日文、特殊符号显示正常
- ⚠️ 部分支持：部分字符显示异常
- ❌ 不支持：乱码

### 3. Emoji 支持
- ✅ 完全支持：所有 emoji 正常显示，无崩溃
- ⚠️ 部分支持：emoji 可能显示异常但不崩溃
- ❌ 不支持：emoji 导致崩溃或严重错误

### 4. Context 功能
- ✅ 完全支持：所有 `/context` 命令正常工作
- ⚠️ 部分支持：部分命令有问题
- ❌ 不支持：context 功能导致崩溃

### 5. 稳定性
- ✅ 稳定：长时间使用无崩溃
- ⚠️ 一般：偶尔出现问题
- ❌ 不稳定：频繁崩溃

---

## 已知问题

### macOS Terminal.app
- **问题**: 使用 emoji 时可能导致整个终端崩溃重启
- **版本**: macOS 13.x - 14.x
- **严重性**: 严重
- **解决方案**:
  - 在配置中设置 `display.use_emoji: false`
  - 或使用其他终端模拟器（iTerm2, Alacritty）
- **根因**: Terminal.app 的 emoji 渲染器存在 bug

### Hyper (macOS/Linux/Windows)
- **问题**: 启动慢，CPU 占用高
- **版本**: 所有版本
- **严重性**: 中等
- **解决方案**: 使用原生终端
- **根因**: 基于 Electron，性能开销大

### xterm (Linux)
- **问题**: 中文显示宽度计算错误
- **版本**: < 388
- **严重性**: 中等
- **解决方案**: 升级到最新版本
- **根因**: 旧版本 Unicode 支持不完善

---

## 推荐配置

### 最佳体验（完整功能）
```yaml
display:
  mode: standard
  use_emoji: true    # 仅在支持的终端使用
  use_colors: true
```

**推荐终端**: WezTerm, Windows Terminal, Kitty

### 稳定优先（兼容性优先）
```yaml
display:
  mode: minimal
  use_emoji: false   # 关闭 emoji，避免兼容性问题
  use_colors: true
```

**推荐终端**: iTerm2, Alacritty, Gnome Terminal

### 极简模式（最大兼容）
```yaml
display:
  mode: minimal
  use_emoji: false
  use_colors: false  # 纯文本模式
```

**推荐终端**: 所有终端

---

## 测试报告模板

### 提交终端兼容性问题
如果发现新的兼容性问题，请提交 issue 并包含以下信息：

```markdown
## 终端信息
- 操作系统: macOS 14.5
- 终端: iTerm2 3.5.0
- $TERM: xterm-256color
- Shell: zsh 5.9

## 问题描述
[详细描述问题]

## 复现步骤
1. 启动 realconsole
2. 执行 `/context start`
3. ...

## 期望行为
[描述期望的正常行为]

## 实际行为
[描述实际发生的问题]

## 截图
[如果可能，提供截图]

## 配置文件
\`\`\`yaml
display:
  use_emoji: false
  use_colors: true
\`\`\`

## 测试脚本输出
\`\`\`bash
$ ./tests/terminal_compatibility.sh
[粘贴输出]
\`\`\`
```

---

## 贡献

欢迎贡献新的终端测试结果！请提交 PR 更新此文档。

**测试流程**：
1. 运行 `./tests/terminal_compatibility.sh`
2. 记录测试结果
3. 更新兼容性矩阵
4. 提交 PR

---

**维护者**: RealConsole Contributors
**最后测试**: 2025-10-22
