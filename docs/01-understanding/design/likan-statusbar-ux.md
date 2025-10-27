# 离坎炼化炉 - 状态栏 UX 改进

**日期**: 2025-10-27
**版本**: Phase 4.3.1
**状态**: ✅ 完成

---

## 🎯 问题描述

### 原有问题

炼化炉循环完成时，直接使用 `println!` 输出到控制台：

```
🌊🔥 离坎炼化炉循环完成:
   - 发现模式: 8
   - 高置信度模式: 3
   - 耗时: 156ms
```

**用户体验问题**：
1. ❌ 打断用户正在输入的命令
2. ❌ 多行输出占据屏幕空间
3. ❌ 与用户输入混在一起，难以区分
4. ❌ 无法持续观察状态，需要等待下次触发

---

## 💡 解决方案

参考 **Claude Code** 的 CLI 设计，实现**底部状态栏**：

```
> user typing here...
🌊🔥 [5m ago] 8 (3 ⭐) patterns | next: 23m
```

### 核心理念

**极简设计三原则**：

1. **简洁（Simplicity）**
   - 单行显示，不占据多行
   - 关键信息一目了然

2. **不干扰（Non-intrusive）**
   - 固定底部，不会突然插入
   - 状态更新静默进行

3. **持续可见（Persistent）**
   - 始终显示，无需等待触发
   - 实时倒计时，知道下次循环时间

---

## 🏗️ 技术实现

### 1. 状态栏模块

**文件**: `src/likan/statusbar.rs`

**核心组件**:

```rust
pub struct LiKanStatusBar {
    /// 进度条（用作状态栏）
    bar: ProgressBar,

    /// 当前状态
    status: Arc<RwLock<FurnaceStatus>>,
}

pub struct FurnaceStatus {
    /// 上次循环时间
    pub last_cycle: Option<Instant>,

    /// 当前模式数量
    pub pattern_count: usize,

    /// 高置信度模式数量
    pub high_confidence_count: usize,

    /// 循环间隔（秒）
    pub cycle_interval_secs: u64,
}
```

### 2. 状态栏格式

**格式模板**:
```
🌊🔥 [<时间>] <模式数> patterns | next: <倒计时>
```

**示例**:

| 状态 | 显示 |
|------|------|
| 初始化 | `🌊🔥 [waiting] 0 patterns \| initializing...` |
| 等待首次 | `🌊🔥 [waiting] 0 patterns \| next: 5m` |
| 循环完成 | `🌊🔥 [2m ago] 8 patterns \| next: 3m` |
| 有高质量模式 | `🌊🔥 [1m ago] 12 (5 ⭐) patterns \| next: 4m` |
| 即将触发 | `🌊🔥 [4m ago] 15 (7 ⭐) patterns \| next: soon` |

### 3. 时间格式化

**极简时间显示**:

```rust
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)      // 30s
    } else if secs < 3600 {
        format!("{}m", secs / 60)  // 15m
    } else {
        format!("{}h", secs / 3600) // 2h
    }
}
```

### 4. 集成到后台循环

**修改点**:

1. **创建状态栏**（`start_likan_background_cycle`）:
```rust
let statusbar = Arc::new(LiKanStatusBar::new());
let status = statusbar.status();
self.likan_statusbar = Some(Arc::clone(&statusbar));
```

2. **更新状态**（循环完成时）:
```rust
// 不再 println!，而是更新状态
{
    let mut s = status.write().await;
    s.last_cycle = Some(Instant::now());
    s.pattern_count = report.patterns_found;
    s.high_confidence_count = report.high_confidence_patterns;
}

statusbar.update().await; // 立即刷新显示
```

3. **定期刷新**（每分钟）:
```rust
loop {
    tokio::time::sleep(Duration::from_secs(60)).await;
    statusbar.update().await; // 更新倒计时
    // ... 检查是否触发循环
}
```

---

## 🎨 设计对比

### Before（打断式）

```
> cargo build
   Compiling realconsole v1.8.1
    Finished dev [unoptimized + debuginfo] target(s) in 5.23s

🌊🔥 离坎炼化炉循环完成:
   - 发现模式: 8
   - 高置信度模式: 3
   - 耗时: 156ms

> ls -la      ← 用户刚才在输入这个，被打断了！
```

### After（状态栏式）

```
> cargo build
   Compiling realconsole v1.8.1
    Finished dev [unoptimized + debuginfo] target(s) in 5.23s
> ls -la
total 256
drwxr-xr-x  18 user  staff   576 Oct 27 15:30 .
...
> git status  ← 用户输入不受干扰
🌊🔥 [2m ago] 8 (3 ⭐) patterns | next: 3m  ← 底部固定显示
```

---

## ✨ 用户体验提升

### 1. 不干扰输入

- ✅ 状态栏固定底部，不会突然出现在输入行中间
- ✅ 用户可以专注于输入命令
- ✅ 屏幕内容保持整洁

### 2. 持续可见

- ✅ 无需等待触发，随时可以看到状态
- ✅ 倒计时让用户知道下次循环时间
- ✅ 模式数量实时显示，了解学习进度

### 3. 极简信息

- ✅ 单行显示，信息密度高
- ✅ 使用图标（🌊🔥、⭐）代替文字
- ✅ 时间格式简洁（5m 而非 5 minutes）

### 4. 专业感

- ✅ 类似 VS Code、Claude Code 的状态栏
- ✅ 符合现代 CLI 工具的设计语言
- ✅ 体现系统的智能和主动性

---

## 📊 信息架构

### 显示元素（从左到右）

1. **🌊🔥** - 离坎图标（固定）
   - 视觉识别，一眼看出是炼化炉

2. **[时间]** - 上次循环时间
   - `[waiting]` - 等待首次
   - `[2m ago]` - 2分钟前完成

3. **模式数** - 学习成果
   - `8 patterns` - 普通模式
   - `8 (3 ⭐) patterns` - 含高质量模式

4. **next: 时间** - 下次循环
   - `next: 3m` - 3分钟后触发
   - `next: soon` - 即将触发

### 信息优先级

```
核心信息 > 上次循环 > 高质量提示 > 下次时间

必须显示: 🌊🔥 [时间] X patterns | next: Ym
可选显示: (N ⭐) - 仅当有高质量模式时
```

---

## 🔧 技术细节

### 使用 `indicatif`

选择 `indicatif` 的原因：

1. **成熟稳定** - 项目已使用（`spinner.rs`）
2. **底层支持** - 处理终端大小变化、光标控制
3. **简单 API** - `ProgressBar` + `set_message()`
4. **持久显示** - `enable_steady_tick()` 自动刷新

### 线程安全

```rust
// 状态在多个异步任务间共享
pub status: Arc<RwLock<FurnaceStatus>>

// 后台任务更新
{
    let mut s = status.write().await;
    s.pattern_count = 10;
}

// 状态栏读取
let status = self.status.read().await;
let msg = format_message(&status);
```

### 自动清理

```rust
impl Drop for LiKanStatusBar {
    fn drop(&mut self) {
        self.bar.finish_and_clear(); // 退出时清理
    }
}
```

---

## 📦 文件清单

### 新增文件

- `src/likan/statusbar.rs` (170+ 行)
  - `LiKanStatusBar` - 状态栏主体
  - `FurnaceStatus` - 状态数据
  - `format_duration()` - 时间格式化
  - 单元测试

### 修改文件

- `src/likan/mod.rs` - 导出 statusbar
- `src/agent.rs` - 集成状态栏
  - 新增字段 `likan_statusbar`
  - 修改 `start_likan_background_cycle()`

---

## 🎯 后续可优化

### 配置选项

未来可添加配置：

```yaml
likan:
  statusbar:
    enabled: true          # 是否启用状态栏
    position: bottom       # 显示位置（bottom/top）
    format: minimal        # 显示格式（minimal/detailed）
```

### 交互能力

未来可以支持：

```bash
# 点击状态栏切换详细信息
🌊🔥 [2m ago] 8 patterns | next: 3m  ← 点击
↓
详细模式:
- Frequency: 5 patterns
- Sequence: 2 patterns
- ErrorFix: 1 pattern
```

### 多语言支持

```rust
// 英文
"🌊🔥 [2m ago] 8 patterns | next: 3m"

// 中文
"🌊🔥 [2分钟前] 8个模式 | 下次: 3分钟"
```

---

## 💡 设计哲学

### 易经智慧

**离卦（☲）的显现**：
- 离为明，照亮进展
- 但不刺眼，静默守护
- 在而不扰，明而不炫

**极简主义**：
- "少则得，多则惑"
- 只显示必要信息
- 形式服从功能

**用户中心**：
- 不是炫技，而是服务
- 状态栏是"仆人"，不是"主人"
- 主角永远是用户的工作

---

## 📈 效果总结

| 方面 | Before | After | 提升 |
|------|--------|-------|------|
| 干扰度 | 高（打断输入） | 无（底部固定） | ⭐⭐⭐⭐⭐ |
| 可见性 | 低（需等触发） | 高（持续显示） | ⭐⭐⭐⭐⭐ |
| 信息量 | 多行详细 | 单行精简 | ⭐⭐⭐⭐ |
| 专业感 | 中等 | 高（类 VS Code） | ⭐⭐⭐⭐⭐ |

---

**完成者**: Claude & RealConsole Team
**参考**: Claude Code, VS Code Status Bar

---

> "形在而神不扰，明在而光不炫"
> "工具当如水，润物而无声"
>
> 🌊🔥✨
