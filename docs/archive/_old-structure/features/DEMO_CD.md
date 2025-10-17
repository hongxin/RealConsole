# CD 命令与动态提示符演示

## 功能说明

当执行 `!cd <directory>` 命令后，提示符会自动更新以反映当前目录。

## 实现细节

### 1. 动态提示符 (src/repl.rs:22-23)
```rust
loop {
    // 每次循环重新构建提示符，以反映当前目录
    let prompt = build_prompt();
```

### 2. CD 命令处理 (src/agent.rs:256-314)
- 检测 `cd` 命令并在主进程中执行
- 支持相对路径（`cd ..`）
- 支持绝对路径（`cd /tmp`）
- 支持 HOME 目录（`cd ~` 或 `cd`）
- 展开 `~` 为实际 HOME 路径

### 3. 提示符格式 (src/repl.rs:84-105)
```
username current_folder_name %
```
- 橙色显示 (RGB 255, 165, 0)
- 仅显示当前目录名（非完整路径）

## 使用示例

```bash
# 启动 RealConsole
./target/release/realconsole

# 初始提示符
hongxin real-console %

# 执行 cd 命令
!cd ..

# 输出
/Users/hongxin/Workspace/claude-ai-playground

# 新提示符（目录名已更新）
hongxin claude-ai-playground %

# 再次切换
!cd real-console

# 输出
/Users/hongxin/Workspace/claude-ai-playground/real-console

# 提示符恢复
hongxin real-console %
```

## 测试用例

### 相对路径
```bash
!cd ..              # 返回上级目录
!cd ./src           # 进入子目录
!cd ../other-dir    # 进入兄弟目录
```

### 绝对路径
```bash
!cd /tmp            # 进入 /tmp
!cd /Users/hongxin  # 进入指定路径
```

### HOME 目录
```bash
!cd                 # 进入 HOME (无参数)
!cd ~               # 进入 HOME (波浪号)
!cd ~/Documents     # 进入 HOME 的子目录
```

### 错误处理
```bash
!cd /nonexistent    # 显示错误: 切换目录失败
```

## 技术要点

1. **为什么需要特殊处理 cd？**
   - 普通 shell 命令在子进程中执行
   - 子进程的目录变更不影响父进程（RealConsole）
   - cd 必须在主进程中执行才能生效

2. **提示符何时更新？**
   - 每次 REPL 循环开始时重新构建
   - 自动反映 `std::env::current_dir()` 的变化

3. **与标准 shell 的一致性**
   - 提示符格式：`username folder %`（类似 zsh）
   - 橙色主题：区分于标准 shell，体现极简设计
   - cd 行为：完全兼容标准 cd 命令

## 版本

- 实现版本：v0.5.2
- Phase：UX 优化（极简主义设计）
