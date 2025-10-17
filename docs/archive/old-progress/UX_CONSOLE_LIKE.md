# UX 改进：Console-Like 体验 (v0.5.2)

**时间**: 2025-10-16
**目标**: 让 RealConsole 的视觉体验与普通 console/shell 完全一致

## 用户需求

基于实际使用反馈，用户希望：
1. **去掉启动后的空行**：第一行之后不要有空行
2. **标准提示符**：使用 `username folder_name %` 格式（类似 zsh）
3. **机器回应无特殊标记**：不要 "AI:" 前缀，直接输出结果

**核心理念**：让用户在使用 RealConsole 时获得与普通 console 在视觉上一致的体验

## 改进前后对比

### Before (v0.5.0)

#### 交互模式
```bash
$ realconsole
RealConsole v0.5.0 | 直接输入问题或 /help | Ctrl-D 退出
                                                    ← 多余空行
» 你好                                              ← 特殊提示符
AI: 你好！我是...                                   ← 有 "AI:" 前缀
```

#### 流式输出
```bash
» 帮我写个函数
AI: 好的，我来帮你...                                ← 有 "AI:" 前缀
```

### After (v0.5.2)

#### 交互模式
```bash
$ realconsole
RealConsole v0.5.2 | 直接输入问题或 /help | Ctrl-D 退出
hongxin real-console % 你好                          ← 无空行，标准提示符
你好！我是一个AI助手...                              ← 无 "AI:" 前缀
hongxin real-console % 现在几点了
现在是 2025年10月16日 01:58:20                      ← 直接输出结果
```

#### 流式输出
```bash
hongxin real-console % 帮我写个函数
好的，我来帮你...                                     ← 无 "AI:" 前缀，直接流式输出
```

## 实现细节

### 1. 去掉启动后的空行

**src/repl.rs**:
```rust
fn print_welcome() {
    let version = env!("CARGO_PKG_VERSION");
    println!("{} {} {} {} {} {} {}",
        "RealConsole".bold().cyan(),
        format!("v{}", version).dimmed(),
        "|".dimmed(),
        "直接输入问题或".dimmed(),
        "/help".cyan(),
        "|".dimmed(),
        "Ctrl-D 退出".dimmed()
    );
    // 去掉 println!(); ← 删除这行
}
```

### 2. 标准 Shell 提示符

**src/repl.rs**:
```rust
/// 构建标准的 shell 提示符
fn build_prompt() -> String {
    // 获取用户名
    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());

    // 获取当前目录名（不是完整路径）
    let current_dir = env::current_dir()
        .ok()
        .and_then(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "~".to_string());

    // 构建提示符：username current_folder %
    format!("{} {} % ", username, current_dir)
}

pub fn run(agent: &Agent) -> RustyResult<()> {
    let mut rl = DefaultEditor::new()?;
    print_welcome();

    let prompt = build_prompt();  // 构建一次，重复使用

    loop {
        let readline = rl.readline(&prompt);  // 使用标准提示符
        // ...
    }
}
```

### 3. 去掉机器回应前缀

**src/agent.rs** (handle_text_streaming):
```rust
fn handle_text_streaming(&self, text: &str) -> String {
    // Before:
    // print!("{} ", "AI:".bold().cyan());
    // let _ = io::stdout().flush();

    // After:
    // 不显示 "AI:" 前缀，让输出更接近普通 console
    // 直接开始流式输出

    let start = Instant::now();
    match tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let manager = self.llm_manager.read().await;
            manager.chat_stream(text, |token| {
                print!("{}", token);  // 直接输出，无前缀
                let _ = io::stdout().flush();
            }).await
        })
    }) {
        // ...
    }
}
```

## 提示符格式说明

### 标准 Shell 提示符

**格式**: `username current_folder %`

**示例**:
```bash
hongxin real-console %          # macOS/Linux
john project %                  # 其他用户
user myapp %                    # 默认用户名
```

**对比主流 Shell**:

| Shell | 格式 | 示例 |
|-------|------|------|
| **zsh** | `username folder %` | `hongxin real-console %` |
| **bash** | `username folder $` | `hongxin real-console $` |
| **fish** | `username folder >` | `hongxin real-console >` |
| **RealConsole** | `username folder %` | `hongxin real-console %` |

我们选择 **zsh 风格**（%），因为：
- 现代 macOS 默认使用 zsh
- 更加清晰易读
- % 不与其他符号冲突

### 环境变量获取

**用户名**：
- 优先使用 `$USER` (Unix/Linux/macOS)
- 回退到 `$USERNAME` (Windows)
- 默认 "user"

**目录名**：
- 使用 `env::current_dir()` 获取当前路径
- 只取目录名（file_name），不是完整路径
- 根目录显示为 "~"

## 用户体验提升

### 1. 视觉一致性

**Before**: 明显是 AI 助手
```bash
» 你好
AI: 你好！我是...
```

**After**: 看起来像普通 shell
```bash
hongxin real-console % 你好
你好！我是...
```

### 2. 减少视觉干扰

**Before**:
- 空行占据屏幕空间
- "AI:" 前缀提醒用户这是 AI
- "»" 符号不标准

**After**:
- 紧凑布局
- 无前缀，沉浸式体验
- 标准提示符，符合习惯

### 3. 更好的集成体验

**脚本集成**：
```bash
# 看起来就像普通命令
$ echo "现在几点了" | realconsole
现在是 2025年10月16日 01:58:20
```

**Shell 历史**：
```bash
$ history | tail -5
  501  ls -la
  502  cd project
  503  realconsole --once "显示最大的文件"
  504  git status
  505  realconsole                    ← 无违和感
```

## 测试结果

### 测试 1：启动无空行
```bash
$ ./target/release/realconsole
RealConsole v0.5.2 | 直接输入问题或 /help | Ctrl-D 退出
hongxin real-console %                                      ← 无空行
```
✅ 通过

### 测试 2：标准提示符
```bash
hongxin real-console % pwd
/Users/hongxin/Workspace/claude-ai-playground/real-console
hongxin real-console % 你好
你好！...
```
✅ 提示符格式正确

### 测试 3：无 AI 前缀
```bash
hongxin real-console % 帮我计算 1+1
2
```
✅ 无 "AI:" 前缀

### 测试 4：--once 模式
```bash
$ ./target/release/realconsole --once "现在几点了"
现在是 2025年10月16日 01:58:20
```
✅ 输出简洁

## 设计权衡

### 保留的标识

我们**保留**了启动行：
```bash
RealConsole v0.5.2 | 直接输入问题或 /help | Ctrl-D 退出
```

**原因**：
- 用户需要知道这是 RealConsole，不是普通 shell
- 显示版本信息（重要）
- 提供帮助提示
- 只显示一次，不干扰后续使用

### 移除的元素

| 元素 | 原因 |
|------|------|
| 启动后空行 | 浪费屏幕空间 |
| "»" 提示符 | 不标准，不符合习惯 |
| "AI:" 前缀 | 打破沉浸感 |

### 最小化哲学

**核心理念**：
> 只显示必要信息，其他一概不显示

这与 v0.5.2 的 **Minimal 显示模式** 完美契合。

## 进一步优化方向

### Phase 1: 可配置提示符
```yaml
display:
  prompt: "{user} {dir} % "  # 可自定义格式
  # 或预设样式
  prompt_style: zsh  # zsh | bash | fish | minimal
```

### Phase 2: 彩色提示符
```rust
// 用户名用绿色，目录名用蓝色
format!("{} {} % ",
    username.green(),
    current_dir.blue()
)
```

### Phase 3: Git 状态
```bash
hongxin real-console (main) %      ← 显示 git 分支
```

### Phase 4: 退出码显示
```bash
hongxin real-console % ls /nonexistent
ls: /nonexistent: No such file or directory
hongxin real-console [1] %         ← 显示上一个命令的退出码
```

## 用户反馈预期

### 预期正面反馈

1. **"看起来就像普通 shell"** ✨
   - 视觉一致性
   - 无违和感

2. **"更加简洁干净"** 🎯
   - 无干扰信息
   - 屏幕利用率高

3. **"更容易集成到工作流"** 🔧
   - 脚本友好
   - 历史记录清晰

### 潜在问题

1. **"提示符不知道是 AI"**
   - 解决：启动行已说明
   - 解决：/help 提供帮助

2. **"想要自定义提示符"**
   - 解决：未来版本支持配置

## 结论

v0.5.2 的 Console-Like 改进让 RealConsole 的用户体验提升到新高度：

✅ **视觉一致性**：与普通 shell 完全一致
✅ **极简设计**：只显示必要信息
✅ **标准化**：遵循主流 shell 规范
✅ **沉浸式**：无干扰标记

**这是 RealConsole 从"AI 工具"到"日常 Shell"的重要一步！** 🚀

---

**完成时间**: 2025-10-16 02:00
**版本**: v0.5.2
**改进项**: 3
**测试**: 4/4 通过
**状态**: ✅ 已完成
