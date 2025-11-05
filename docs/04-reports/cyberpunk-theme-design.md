# 🌃 赛博朋克主题设计文档

**版本**: v1.26.0
**日期**: 2025-11-05
**主题**: Cyberpunk (未来赛博朋克)

## 设计理念

将 RealConsole Web 终端打造成充满未来感的赛博朋克风格界面，融合霓虹色彩、发光效果、动态动画，营造出《银翼杀手》和《赛博朋克 2077》般的视觉体验，同时保持终端的可读性和可用性。

## 核心视觉元素

### 1. 色彩系统

#### 主色调 - 霓虹色系
- **霓虹青色** (#00f0ff): 主要文字、输入、边框
- **霓虹粉/品红** (#ff006e): 强调元素、Markdown 边框、列表
- **霓虹绿色** (#39ff14): Prompt、代码块、思考状态
- **霓虹紫色** (#a239ea): 引用块、辅助元素
- **霓虹黄色** (#ffea00): 警告、高亮

#### 背景色系
- **深蓝黑渐变**: #0a0e27 → #0d1117 → #1a0b2e
- **透明黑**: rgba(5, 8, 20, 0.85) - 终端背景
- **半透明青色**: 各种透明度的青色作为辅助背景

### 2. 发光效果 (Glow)

所有关键元素都添加了多层 `text-shadow` 和 `box-shadow` 实现霓虹发光：

```css
/* 标准发光模式 */
text-shadow:
    0 0 10px rgba(0, 240, 255, 0.8),  /* 内层强光 */
    0 0 20px rgba(0, 240, 255, 0.4),  /* 中层辉光 */
    0 0 30px rgba(0, 240, 255, 0.2);  /* 外层光晕 */
```

### 3. 动态效果

#### a. 扫描线 (Scanlines)
模拟老式 CRT 显示器效果：
- 水平扫描线覆盖整个页面
- 8秒循环向下移动
- 30% 透明度，不影响内容可读性

#### b. 脉动动画 (Pulse)
- 标题霓虹脉动（2s 周期）
- 终端边框彩虹脉动（3s 周期）
- 状态指示器脉动（2s 周期）
- Spinner 发光脉动（1s 周期）

#### c. Prompt 闪烁
绿色 Prompt 以 1.5s 周期微弱闪烁，模拟终端光标

### 4. 网格背景 (Grid)

使用 CSS `repeating-linear-gradient` 创建透视网格效果：
- 40px × 40px 网格
- 霓虹青色 (3% 透明度)
- 固定背景 (background-attachment: fixed)

## 详细设计规范

### 页面整体

```css
body {
    /* 深色背景 + 网格 + 渐变 */
    background:
        repeating-linear-gradient(/* 水平网格 */),
        repeating-linear-gradient(/* 垂直网格 */),
        linear-gradient(/* 深色渐变 */);
}

/* 扫描线效果 */
body::before {
    animation: scanlines 8s linear infinite;
}
```

### Header 区域

**标题**:
- 青色→粉色渐变文字
- 多层霓虹发光
- 2s 脉动动画

**语言切换按钮**:
- 霓虹青色边框
- Hover 时增强发光
- Active 状态内发光

### 终端容器

```css
#terminal-container {
    background: rgba(5, 8, 20, 0.85);
    border: 2px solid rgba(0, 240, 255, 0.5);
    box-shadow: 多层发光;
}

/* 边框彩虹脉动 */
#terminal-container::before {
    background: linear-gradient(青色→粉色→青色);
    animation: border-glow 3s;
}
```

### 输入字段

- **Prompt**: 霓虹绿色 + 发光 + 闪烁动画
- **输入文字**: 霓虹青色 + 微弱发光
- **分割线**: 霓虹青色上边框 + 发光阴影

### ANSI 颜色映射

| 原始 | 赛博朋克色 | 发光效果 |
|------|-----------|---------|
| Red | #ff0055 | ✅ |
| Green | #39ff14 | ✅ |
| Yellow | #ffea00 | ✅ |
| Blue | #00f0ff | ✅ |
| Cyan | #00ffff | ✅ |
| White | #ffffff | ✅ |

### Markdown 样式

#### 标题
- 青色→粉色渐变
- 发光效果
- 过滤器阴影

#### 代码块
```css
pre {
    background: rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(57, 255, 20, 0.3);
    box-shadow: 外发光 + 内发光;
}

code {
    color: #39ff14;  /* 霓虹绿 */
    text-shadow: 0 0 5px rgba(57, 255, 20, 0.3);
}
```

#### 列表
- Bullet/Number: 霓虹粉色 + 发光
- 内容: 霓虹青色

#### 引用块
- 左边框: 霓虹紫色
- 背景: 紫色半透明
- 左侧发光阴影

#### 链接
- 默认: 霓虹粉色 + 下划线
- Hover: 增强发光 + 颜色加深

#### 分隔线
- 青色→粉色→青色渐变
- 两端淡出
- 发光效果

#### 表格
- 边框: 霓虹青色 (30% 透明)
- 表头: 青色背景 + 发光文字
- 单元格: 淡青色背景

### 滚动条

```css
::-webkit-scrollbar-thumb {
    background: rgba(0, 240, 255, 0.3);
    box-shadow:
        0 0 5px rgba(0, 240, 255, 0.5),
        inset 0 0 3px rgba(0, 240, 255, 0.3);
}

::-webkit-scrollbar-thumb:hover {
    /* 增强发光 */
    box-shadow: 0 0 10px rgba(0, 240, 255, 0.8);
}
```

### 状态指示器

- 霓虹青色边框
- 半透明背景
- 文字发光
- 2s 脉动动画

## 动画时间线

| 元素 | 动画 | 周期 | 效果 |
|------|------|------|------|
| 扫描线 | scanlines | 8s | 上→下移动 |
| 标题 | neon-pulse | 2s | 亮度+发光脉动 |
| 终端边框 | border-glow | 3s | 彩虹脉动 |
| Prompt | prompt-blink | 1.5s | 透明度闪烁 |
| Spinner | spinner-glow | 1s | 发光脉动 |
| 状态栏 | status-pulse | 2s | 发光脉动 |

## 性能优化

### 1. 硬件加速
关键动画使用 `transform` 和 `opacity`，触发 GPU 加速

### 2. 分层渲染
使用 `will-change` 提示浏览器优化：
```css
.hybrid-terminal {
    will-change: contents;
}
```

### 3. 透明度控制
发光效果使用低透明度，避免过度重绘

## 可访问性考虑

### 1. 对比度
- 文字与背景对比度 > 7:1
- 霓虹效果不影响可读性

### 2. 动画敏感
可通过添加媒体查询禁用动画：
```css
@media (prefers-reduced-motion: reduce) {
    * {
        animation: none !important;
        transition: none !important;
    }
}
```

### 3. 颜色盲友好
- 不仅依赖颜色区分信息
- 使用边框、粗细、大小等辅助手段

## 浏览器兼容性

| 功能 | Chrome | Firefox | Safari | Edge |
|------|--------|---------|--------|------|
| 渐变背景 | ✅ | ✅ | ✅ | ✅ |
| 文字渐变 | ✅ | ✅ | ✅ | ✅ |
| 多层阴影 | ✅ | ✅ | ✅ | ✅ |
| 动画 | ✅ | ✅ | ✅ | ✅ |
| 背景滤镜 | ✅ | ⚠️ 部分 | ✅ | ✅ |

## 主题变体 (未来)

可基于此设计创建主题系统：

1. **Cyberpunk Blue** (当前) - 青色主导
2. **Cyberpunk Pink** - 粉色主导
3. **Matrix Green** - 绿色主导（黑客帝国风格）
4. **Tron Legacy** - 蓝+橙组合
5. **Classic Terminal** - 回归传统终端样式

## 测试建议

### 视觉验证
1. 启动 Web 服务器
2. 检查所有霓虹效果是否正常
3. 验证动画流畅性
4. 测试不同屏幕尺寸

### 性能验证
1. 打开浏览器开发者工具
2. 监控 FPS（应保持 60fps）
3. 检查 CPU 使用率（应 < 10%）
4. 确认无明显掉帧

### 功能验证
1. 输入命令 - 检查颜色和发光
2. LLM 对话 - 验证 Markdown 渲染
3. 代码块 - 确认绿色发光
4. 列表 - 验证粉色 bullet

## 文件修改

**修改文件**: `src/web/server.rs`
**修改行数**: 1022-1707 (CSS 部分)
**新增代码**: ~700 行

## 示例截图说明

用户应该看到：
- 深色背景 + 微弱网格
- 标题呈现青→粉渐变并微微脉动
- 终端容器有青色发光边框
- 输入提示符呈现闪烁的绿色发光
- Markdown 内容带有粉色左边框
- 所有文字都有适度的霓虹发光
- 页面顶部有细微的扫描线效果

---

**主题完成** ✅
**未来感指数**: ⭐⭐⭐⭐⭐
**赛博朋克度**: 🌃🌃🌃🌃🌃
