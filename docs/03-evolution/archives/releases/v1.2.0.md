# RealConsole v1.2.0 发布说明

**发布日期**: 2025-10-20
**代号**: "Lunar Wisdom & UX Polish" （农历智慧与体验优化）
**主题**: 中国传统历法集成 + 极简主义 UI 改进

---

## 🎯 核心特性

### 1. 农历工具系统 🌙

完整集成中国农历系统，支持公历/农历双向转换和传统八字计算。

**主要功能**:
- ✅ 公历 ↔ 农历双向转换（1901-2101年）
- ✅ 完整八字（四柱）计算：年月日时
- ✅ 闰月自动处理
- ✅ 干支、生肖、星座、节气查询
- ✅ 称骨算命（袁天罡骨重法）
- ✅ LLM 智能调用（自然语言查询）

**使用示例**:
```bash
# 自然语言查询（LLM 自动理解并调用）
% 今天农历几月几日？
📅 公历: 2025-10-20 → 农历: 乙巳年 九月十九
   干支: 丙戌月 甲午日
   生肖: 蛇年  星座: 天秤座
   节气: 寒露已过 4天

% 计算八字，1990年5月20日下午2点出生
🔧 调用工具: lunar_calendar
八字（四柱）: 庚午年 辛巳月 甲寅日 辛未时
  - 年柱: 庚午（金马）
  - 月柱: 辛巳（金蛇）
  - 日柱: 甲寅（木虎）
  - 时柱: 辛未（金羊）
五行: 木2 火2 土0 金3 水0
称骨: 4两1钱（聪明超群，老来富贵之命）

% 2025年春节是几月几号？
📅 农历: 乙巳年 正月初一 → 公历: 2025-01-29
```

**技术实现**:
- 基于 `chinese-lunisolar-calendar` 库
- 实现五虎遁月、五鼠遁日等传统算法
- 支持 `lunar:` 前缀智能识别日期类型
- 三级详细输出（simple/standard/full）

---

### 2. 任务执行可视化增强 📊

优化 `/plan` 和 `/execute` 命令的用户体验。

**改进内容**:

#### A. 三级显示策略
根据 `DisplayMode` 提供渐进式信息呈现：

```bash
# Minimal 模式 - 快速查看
✓ 2/3 · 66% · 45秒

# Standard 模式 - 日常使用（默认）
══ 任务执行结果 ══
✓ 2/3 · 66% · 45秒

失败任务:
✗ 编译项目
  $ cargo build --release
  error: could not compile `realconsole`

# Debug 模式 - 深度调试
══ 任务执行结果 ══
执行计划: 创建项目结构并初始化
阶段数: 3 · 任务数: 4 · 预计: 60秒 · 实际: 45秒

阶段 1 [串行] (15秒)
✓ 创建项目目录 $ mkdir -p myproject (2秒)
  输出: [目录创建成功]

阶段 2 [并行] (20秒)
✓ 初始化 src $ mkdir -p myproject/src (3秒)
✓ 初始化 tests $ mkdir -p myproject/tests (3秒)
...
```

#### B. Spinner 动画
为 `/plan` 命令添加加载动画，提升等待体验：

```bash
/plan 创建 Rust 项目并初始化

⠋ 正在分解任务...  # 旋转动画，等待 LLM 完成任务分解
```

---

### 3. 极简色彩方案 🎨

贯彻"极简主义"设计理念，简化视觉呈现。

**优化原则**: "色彩是信息的载体，而非装饰"

**改进统计**:
- 移除 9 处冗余颜色
- 统一为 3 种功能色彩
- 依靠粗体/淡化区分层次

**色彩三分法**:
- **Cyan（青色）**: 仅用于标题和分隔线
- **Green/Red（绿/红）**: 仅用于状态标识（✓/✗）
- **无色**: 所有数据内容，用 Bold/Dimmed 区分

**视觉对比**:
```
# 优化前（颜色过多）
[●].cyan [串行].cyan
[任务类型].yellow: Shell
[命令].cyan: cargo build

# 优化后（极简）
● 串行                    # 无色+粗体
任务类型: Shell           # 无色
命令: cargo build         # 无色
```

---

### 4. UTF-8 安全修复 🔧

修复中文字符截断导致的 panic 问题。

**问题症状**:
```
thread 'main' panicked at src/display.rs:760:35:
byte index 67 is not a char boundary; it is inside '八' (bytes 66..69)
```

**根本原因**: 使用字节索引而非字符计数

**修复方案**:
```rust
// ❌ 错误 - 字节切片不安全
&text[..max_len - 3]

// ✅ 正确 - 字符迭代器安全截断
text.chars().take(max_len - 3).collect()
```

**测试覆盖**: 新增 `test_truncate_chinese_chars` 测试确保不再复现

---

## 📊 版本统计

### 代码变更
| 指标 | 数量 |
|------|------|
| 新增文件 | 1 个（lunar_tool.rs） |
| 修改文件 | 4 个 |
| 新增代码 | ~747 行 |
| 简化颜色 | 9 处 |
| Bug 修复 | 1 个（UTF-8） |
| 新增测试 | 1 个 |

### 依赖变更
```toml
# 新增依赖
chinese-lunisolar-calendar = "0.1"  # 中国农历库（1901-2100）
```

### 测试覆盖
- ✅ 农历工具：完整功能覆盖
- ✅ UTF-8 截断：专项测试
- ✅ 所有现有测试：100% 通过

---

## 🚀 升级指南

### 兼容性
v1.1.x → v1.2.0 **完全向后兼容**，无破坏性变更。

### 安装/升级
```bash
# 从源码编译并安装
make install

# 或手动构建
cargo build --release
cp target/release/realconsole ~/.local/bin/
```

### 配置
无需修改 `realconsole.yaml`，所有新功能开箱即用。

---

## 💡 使用建议

### 农历查询
```bash
# 快速查询今天
% 今天农历几号

# 查询特定日期
% 2025年春节是几号

# 计算八字（需要时间）
% 帮我算八字，1990-05-20 14:00
```

### 任务执行
```bash
# 配置显示模式
vim ~/.realconsole.yaml

display:
  mode: minimal    # 快速模式
  # mode: standard   # 标准模式（默认）
  # mode: debug      # 调试模式
```

---

## 🎯 设计哲学实践

本版本深度实践 RealConsole 的核心设计理念：

### 极简主义
- 色彩简化（9 处移除）
- 单一工具多功能（lunar_calendar 覆盖所有农历需求）
- 渐进式信息披露（三级显示）

### 一分为三
- 显示模式三态（Minimal/Standard/Debug）
- 农历输出三级（simple/standard/full）
- 色彩三分法（功能/结构/无色）

### 易经智慧
- 农历系统体现中国传统智慧
- 干支五行阴阳平衡
- 八字推算因果演化

---

## 📖 详细文档

- **完整更新日志**: `docs/CHANGELOG.md#v120`
- **用户指南**: `docs/02-practice/user/user-guide.md`
- **开发文档**: `docs/02-practice/developer/developer-guide.md`

---

## 🙏 致谢

感谢所有 RealConsole 用户的反馈和支持！

**维护**: RealConsole Contributors
**许可**: MIT License
**仓库**: https://github.com/hongxin/RealConsole

---

**祝使用愉快！** 🎉
