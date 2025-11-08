# Web 意图占卜 UI 设计 - 充分发挥 Web 界面优势

**撰写日期**: 2025-11-08 深夜
**目标**: 设计一套充分利用 Web 技术优势的意图占卜可视化系统

---

## 🎯 设计原则

### Web vs CLI 的本质差异

| 维度 | CLI | Web | 设计策略 |
|------|-----|-----|---------|
| 视觉 | 单色文本 | 丰富色彩、动画 | **最大化利用视觉表现力** |
| 交互 | 键盘输入 | 鼠标、触摸、拖拽 | **设计直观的交互方式** |
| 布局 | 线性流 | 多维布局 | **使用卡片、面板、分层** |
| 动画 | 字符闪烁 | CSS3、Canvas、WebGL | **创造沉浸式体验** |
| 音效 | 系统蜂鸣 | 丰富音频 | **增强仪式感** |
| 状态 | 无状态 | 持久化状态 | **保存用户偏好和历史** |

### 核心设计理念

**不要简单地把 CLI 搬到 Web 上**
**而要重新设计适合 Web 的交互范式**

---

## 🎨 视觉设计系统

### 1. 卦象可视化（SVG + CSS Animation）

#### 1.1 爻画的动态生成

**技术选型**: SVG（矢量图，可缩放，易于动画）

```html
<!-- 单个阳爻 -->
<svg class="yao yang" viewBox="0 0 100 20">
  <line x1="0" y1="10" x2="100" y2="10"
        stroke="#FFD700"
        stroke-width="4"
        stroke-linecap="round"
        class="yao-line"/>
  <circle cx="50" cy="10" r="15"
          fill="url(#goldGradient)"
          opacity="0.2"
          class="yao-glow"/>
</svg>

<!-- 单个阴爻 -->
<svg class="yao yin" viewBox="0 0 100 20">
  <line x1="0" y1="10" x2="40" y2="10"
        stroke="#FFD700"
        stroke-width="4"
        stroke-linecap="round"/>
  <line x1="60" y1="10" x2="100" y2="10"
        stroke="#FFD700"
        stroke-width="4"
        stroke-linecap="round"/>
  <circle cx="50" cy="10" r="8"
          fill="rgba(255, 215, 0, 0.3)"/>
</svg>
```

**CSS 动画**:
```css
.yao-line {
    stroke-dasharray: 100;
    stroke-dashoffset: 100;
    animation: drawYao 0.5s ease-out forwards;
}

@keyframes drawYao {
    to {
        stroke-dashoffset: 0;
    }
}

.yao-glow {
    animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
    0%, 100% { opacity: 0.1; r: 15; }
    50% { opacity: 0.3; r: 18; }
}
```

#### 1.2 六爻卦象的组合

**布局**: Flexbox 垂直排列（从下往上）

```html
<div class="hexagram-container">
  <!-- 上卦 -->
  <div class="trigram upper">
    <div class="yao" data-position="shang">上爻</div>
    <div class="yao" data-position="wu">五爻</div>
    <div class="yao" data-position="si">四爻</div>
  </div>

  <!-- 下卦 -->
  <div class="trigram lower">
    <div class="yao" data-position="san">三爻</div>
    <div class="yao" data-position="er">二爻</div>
    <div class="yao" data-position="chu">初爻</div>
  </div>
</div>
```

**CSS**:
```css
.hexagram-container {
    display: flex;
    flex-direction: column-reverse;  /* 从下往上 */
    gap: 4px;
    align-items: center;
    padding: 24px;
    background: radial-gradient(
        circle at center,
        rgba(255, 215, 0, 0.1) 0%,
        transparent 70%
    );
}

.yao {
    width: 200px;
    height: 24px;
    margin: 8px 0;
    position: relative;
    transition: all 0.3s ease;
}

.yao:hover {
    transform: scale(1.1);
    filter: drop-shadow(0 0 10px rgba(255, 215, 0, 0.8));
}

/* 爻位标记（鼠标悬停显示） */
.yao::before {
    content: attr(data-position);
    position: absolute;
    left: -60px;
    top: 50%;
    transform: translateY(-50%);
    opacity: 0;
    transition: opacity 0.3s;
    color: #FFD700;
    font-size: 12px;
}

.yao:hover::before {
    opacity: 1;
}
```

### 2. 蓍草演算可视化（Canvas Animation）

#### 2.1 Canvas 绘制蓍草

**为什么用 Canvas？**
- 需要绘制大量动态元素（49 根蓍草）
- 需要流畅的动画（60fps）
- 需要复杂的视觉效果（分堆、揲数）

```javascript
class YarrowStalksAnimation {
    constructor(canvasId) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.stalks = [];
        this.initStalks(49);  // 大衍之数
    }

    initStalks(count) {
        for (let i = 0; i < count; i++) {
            this.stalks.push({
                x: Math.random() * this.canvas.width,
                y: Math.random() * this.canvas.height,
                angle: Math.random() * Math.PI * 2,
                length: 60,
                width: 2,
                color: '#FFD700'
            });
        }
    }

    // 分二：将蓍草分成两堆
    async fenEr() {
        const mid = this.canvas.width / 2;

        // 动画：蓍草向两边移动
        await this.animateStalks(stalk => {
            const targetX = stalk.x < mid
                ? mid - 100 - Math.random() * 50
                : mid + 100 + Math.random() * 50;
            return { x: targetX };
        }, 500);

        // 显示两堆的数量
        const leftCount = this.stalks.filter(s => s.x < mid).length;
        const rightCount = this.stalks.length - leftCount;
        this.showCount(mid - 150, leftCount, 'left');
        this.showCount(mid + 150, rightCount, 'right');
    }

    // 挂一：从右手取一根放在小指间
    async guaYi() {
        const rightStalks = this.stalks.filter(s => s.x > this.canvas.width / 2);
        const guaStalk = rightStalks[0];

        // 动画：一根蓍草飞到特殊位置
        await this.animateSingle(guaStalk, {
            x: this.canvas.width / 2,
            y: this.canvas.height - 50,
            scale: 1.5,
            color: '#FF4500'  // 高亮显示
        }, 300);
    }

    // 揲四：四根四根地数
    async sheeSi() {
        // 动画：蓍草四根一组地分组
        const groups = [];
        for (let i = 0; i < this.stalks.length; i += 4) {
            groups.push(this.stalks.slice(i, i + 4));
        }

        // 逐组动画
        for (const group of groups) {
            await this.animateGroup(group, 100);
        }
    }

    // 通用动画方法
    async animateStalks(updateFn, duration) {
        const startTime = Date.now();
        const initialStates = this.stalks.map(s => ({...s}));
        const targetStates = this.stalks.map(updateFn);

        return new Promise(resolve => {
            const animate = () => {
                const elapsed = Date.now() - startTime;
                const progress = Math.min(elapsed / duration, 1);
                const eased = this.easeInOutCubic(progress);

                this.stalks.forEach((stalk, i) => {
                    const initial = initialStates[i];
                    const target = targetStates[i];

                    Object.keys(target).forEach(key => {
                        stalk[key] = initial[key] + (target[key] - initial[key]) * eased;
                    });
                });

                this.draw();

                if (progress < 1) {
                    requestAnimationFrame(animate);
                } else {
                    resolve();
                }
            };

            animate();
        });
    }

    easeInOutCubic(t) {
        return t < 0.5
            ? 4 * t * t * t
            : 1 - Math.pow(-2 * t + 2, 3) / 2;
    }

    draw() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        // 绘制背景纹理
        this.drawBackground();

        // 绘制所有蓍草
        this.stalks.forEach(stalk => {
            this.ctx.save();
            this.ctx.translate(stalk.x, stalk.y);
            this.ctx.rotate(stalk.angle);

            // 蓍草本体
            this.ctx.strokeStyle = stalk.color;
            this.ctx.lineWidth = stalk.width;
            this.ctx.lineCap = 'round';
            this.ctx.beginPath();
            this.ctx.moveTo(-stalk.length / 2, 0);
            this.ctx.lineTo(stalk.length / 2, 0);
            this.ctx.stroke();

            // 光晕效果
            const gradient = this.ctx.createRadialGradient(0, 0, 0, 0, 0, stalk.length / 2);
            gradient.addColorStop(0, 'rgba(255, 215, 0, 0.3)');
            gradient.addColorStop(1, 'rgba(255, 215, 0, 0)');
            this.ctx.fillStyle = gradient;
            this.ctx.fillRect(-stalk.length / 2, -10, stalk.length, 20);

            this.ctx.restore();
        });
    }

    drawBackground() {
        // 绘制古朴的背景纹理
        this.ctx.fillStyle = '#0a0a0a';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

        // 添加噪点纹理
        this.ctx.globalAlpha = 0.05;
        for (let i = 0; i < 1000; i++) {
            this.ctx.fillStyle = '#FFD700';
            this.ctx.fillRect(
                Math.random() * this.canvas.width,
                Math.random() * this.canvas.height,
                1, 1
            );
        }
        this.ctx.globalAlpha = 1;
    }
}
```

#### 2.2 演算步骤的数字显示

**设计**: 大字号数字 + 操作名称 + 动画过渡

```html
<div class="yarrow-counter">
  <div class="operation-name">大衍之数</div>
  <div class="stalk-count">49</div>
  <div class="operation-desc">其用四十有九</div>
</div>
```

```css
.yarrow-counter {
    text-align: center;
    font-family: 'STKaiti', 'SimSun', serif;
    color: #FFD700;
}

.operation-name {
    font-size: 18px;
    margin-bottom: 8px;
    opacity: 0.7;
}

.stalk-count {
    font-size: 72px;
    font-weight: bold;
    text-shadow:
        0 0 10px rgba(255, 215, 0, 0.5),
        0 0 20px rgba(255, 215, 0, 0.3),
        0 0 30px rgba(255, 215, 0, 0.1);
    transition: all 0.5s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

.stalk-count.changing {
    transform: scale(1.2) rotateY(360deg);
}

.operation-desc {
    font-size: 14px;
    margin-top: 8px;
    opacity: 0.6;
    font-style: italic;
}
```

### 3. 八卦符号的现代化呈现

#### 3.1 SVG 渐变和阴影

```html
<svg class="trigram-symbol" viewBox="0 0 100 100">
  <defs>
    <!-- 金色渐变 -->
    <linearGradient id="goldGradient" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#FFD700" />
      <stop offset="100%" stop-color="#FFA500" />
    </linearGradient>

    <!-- 光晕滤镜 -->
    <filter id="glow">
      <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
      <feMerge>
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
  </defs>

  <!-- 乾卦 ☰ -->
  <g class="qian" filter="url(#glow)">
    <rect x="10" y="20" width="80" height="6" rx="3" fill="url(#goldGradient)"/>
    <rect x="10" y="47" width="80" height="6" rx="3" fill="url(#goldGradient)"/>
    <rect x="10" y="74" width="80" height="6" rx="3" fill="url(#goldGradient)"/>
  </g>
</svg>
```

#### 3.2 交互式八卦轮盘

**设计**: 用户可以旋转轮盘查看不同卦象

```html
<div class="bagua-wheel">
  <svg viewBox="0 0 300 300" class="wheel-svg">
    <!-- 中心太极图 -->
    <circle cx="150" cy="150" r="40" fill="url(#taijiGradient)"/>

    <!-- 八个卦象，均匀分布 -->
    <g class="trigram-group" data-trigram="qian" transform="rotate(0 150 150)">
      <path d="M 150 50 L 150 100" class="trigram-line"/>
      <text x="150" y="35" class="trigram-label">乾</text>
      <use href="#qianSymbol" x="135" y="60"/>
    </g>

    <!-- 其他七个卦象... -->
  </svg>

  <div class="wheel-controls">
    <button class="rotate-left">⬅</button>
    <button class="rotate-right">➡</button>
  </div>
</div>
```

```css
.bagua-wheel {
    position: relative;
    width: 300px;
    height: 300px;
    margin: 20px auto;
}

.wheel-svg {
    transition: transform 0.5s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

.wheel-svg.rotating {
    animation: spin 2s linear infinite;
}

@keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}

.trigram-group {
    cursor: pointer;
    transition: all 0.3s;
}

.trigram-group:hover {
    filter: drop-shadow(0 0 10px rgba(255, 215, 0, 0.8));
    transform: scale(1.1);
}
```

### 4. 粒子效果系统

#### 4.1 成卦时的光粒子爆发

**技术**: Canvas + 粒子系统

```javascript
class ParticleSystem {
    constructor(canvasId) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.particles = [];
    }

    // 在卦象形成时爆发粒子
    burstAt(x, y, count = 100) {
        for (let i = 0; i < count; i++) {
            this.particles.push({
                x, y,
                vx: (Math.random() - 0.5) * 10,
                vy: (Math.random() - 0.5) * 10,
                life: 1.0,
                decay: 0.01 + Math.random() * 0.02,
                size: 2 + Math.random() * 3,
                color: this.randomColor()
            });
        }
    }

    randomColor() {
        const colors = [
            'rgba(255, 215, 0, ',   // 金色
            'rgba(255, 140, 0, ',   // 橙色
            'rgba(255, 69, 0, ',    // 红橙
            'rgba(255, 255, 255, '  // 白色
        ];
        return colors[Math.floor(Math.random() * colors.length)];
    }

    update() {
        this.particles = this.particles.filter(p => {
            p.x += p.vx;
            p.y += p.vy;
            p.vy += 0.1;  // 重力
            p.life -= p.decay;
            return p.life > 0;
        });
    }

    draw() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        this.particles.forEach(p => {
            this.ctx.save();
            this.ctx.globalAlpha = p.life;
            this.ctx.fillStyle = p.color + p.life + ')';
            this.ctx.beginPath();
            this.ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
            this.ctx.fill();

            // 添加光晕
            const gradient = this.ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.size * 2);
            gradient.addColorStop(0, p.color + p.life + ')');
            gradient.addColorStop(1, p.color + '0)');
            this.ctx.fillStyle = gradient;
            this.ctx.fillRect(p.x - p.size * 2, p.y - p.size * 2, p.size * 4, p.size * 4);

            this.ctx.restore();
        });
    }

    animate() {
        this.update();
        this.draw();

        if (this.particles.length > 0) {
            requestAnimationFrame(() => this.animate());
        }
    }
}
```

---

## 🎮 交互设计系统

### 1. 拖拽修改执行计划

#### 1.1 拖拽排序步骤

```javascript
class DraggableSteps {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.initDragAndDrop();
    }

    initDragAndDrop() {
        const steps = this.container.querySelectorAll('.intent-step');

        steps.forEach(step => {
            step.draggable = true;

            step.addEventListener('dragstart', (e) => {
                e.dataTransfer.effectAllowed = 'move';
                e.dataTransfer.setData('text/html', step.innerHTML);
                step.classList.add('dragging');
            });

            step.addEventListener('dragend', (e) => {
                step.classList.remove('dragging');
            });

            step.addEventListener('dragover', (e) => {
                e.preventDefault();
                const dragging = this.container.querySelector('.dragging');
                const afterElement = this.getDragAfterElement(e.clientY);

                if (afterElement == null) {
                    this.container.appendChild(dragging);
                } else {
                    this.container.insertBefore(dragging, afterElement);
                }
            });
        });
    }

    getDragAfterElement(y) {
        const draggableElements = [
            ...this.container.querySelectorAll('.intent-step:not(.dragging)')
        ];

        return draggableElements.reduce((closest, child) => {
            const box = child.getBoundingClientRect();
            const offset = y - box.top - box.height / 2;

            if (offset < 0 && offset > closest.offset) {
                return { offset: offset, element: child };
            } else {
                return closest;
            }
        }, { offset: Number.NEGATIVE_INFINITY }).element;
    }
}
```

```css
.intent-step {
    transition: transform 0.2s, opacity 0.2s;
    cursor: move;
}

.intent-step.dragging {
    opacity: 0.5;
    transform: rotate(5deg);
}

.intent-step:hover {
    transform: translateX(5px);
    box-shadow: 0 4px 8px rgba(255, 215, 0, 0.3);
}
```

#### 1.2 滑动调整参数

**设计**: 某些数值参数可以通过滑动条调整

```html
<div class="param-slider">
  <label>max_depth（搜索深度）</label>
  <input type="range" min="1" max="20" value="10"
         class="slider" id="maxDepthSlider">
  <span class="slider-value">10</span>
</div>
```

```javascript
const slider = document.getElementById('maxDepthSlider');
const valueDisplay = document.querySelector('.slider-value');

slider.addEventListener('input', (e) => {
    valueDisplay.textContent = e.target.value;

    // 实时预览效果
    updateParamPreview('max_depth', e.target.value);
});
```

```css
.slider {
    -webkit-appearance: none;
    width: 100%;
    height: 8px;
    border-radius: 5px;
    background: linear-gradient(
        to right,
        rgba(255, 215, 0, 0.2) 0%,
        rgba(255, 215, 0, 0.5) 50%,
        rgba(255, 215, 0, 0.2) 100%
    );
    outline: none;
}

.slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #FFD700;
    cursor: pointer;
    box-shadow: 0 0 10px rgba(255, 215, 0, 0.8);
    transition: all 0.2s;
}

.slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
    box-shadow: 0 0 20px rgba(255, 215, 0, 1);
}
```

### 2. 点击铜钱"抛卦"

**设计**: 用户可以手动"抛铜钱"来触发占卜

```html
<div class="coin-toss-area">
  <div class="coins">
    <div class="coin" data-coin="1">
      <div class="coin-face front">字</div>
      <div class="coin-face back">背</div>
    </div>
    <div class="coin" data-coin="2">
      <div class="coin-face front">字</div>
      <div class="coin-face back">背</div>
    </div>
    <div class="coin" data-coin="3">
      <div class="coin-face front">字</div>
      <div class="coin-face back">背</div>
    </div>
  </div>
  <button class="toss-button">抛掷铜钱</button>
  <div class="result">等待抛掷...</div>
</div>
```

```javascript
class CoinToss {
    constructor() {
        this.coins = document.querySelectorAll('.coin');
        this.tossButton = document.querySelector('.toss-button');
        this.resultDiv = document.querySelector('.result');

        this.tossButton.addEventListener('click', () => this.toss());
    }

    async toss() {
        this.tossButton.disabled = true;

        // 同时抛掷三枚铜钱
        const results = await Promise.all([
            this.tossCoin(this.coins[0]),
            this.tossCoin(this.coins[1]),
            this.tossCoin(this.coins[2])
        ]);

        // 计算结果（三正为老阳9，三背为老阴6，二正一背为少阳7，一正二背为少阴8）
        const fronts = results.filter(r => r === 'front').length;
        let yaoType, yaoValue;

        switch(fronts) {
            case 3: yaoType = '老阳'; yaoValue = 9; break;
            case 2: yaoType = '少阳'; yaoValue = 7; break;
            case 1: yaoType = '少阴'; yaoValue = 8; break;
            case 0: yaoType = '老阴'; yaoValue = 6; break;
        }

        this.resultDiv.textContent = `${yaoType}（${yaoValue}）`;
        this.tossButton.disabled = false;

        return yaoValue;
    }

    tossCoin(coinElement) {
        return new Promise(resolve => {
            const duration = 1000 + Math.random() * 500;
            const rotations = 5 + Math.floor(Math.random() * 3);
            const result = Math.random() < 0.5 ? 'front' : 'back';

            coinElement.style.animation = `
                coinFlip ${duration}ms ease-out
            `;

            setTimeout(() => {
                coinElement.classList.add(result);
                coinElement.style.animation = '';
                resolve(result);
            }, duration);
        });
    }
}
```

```css
.coin {
    width: 80px;
    height: 80px;
    position: relative;
    transform-style: preserve-3d;
    transition: transform 0.6s;
    margin: 0 10px;
    cursor: pointer;
}

.coin-face {
    position: absolute;
    width: 100%;
    height: 100%;
    border-radius: 50%;
    backface-visibility: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
    font-weight: bold;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
}

.coin-face.front {
    background: radial-gradient(circle, #FFD700, #FFA500);
    color: #8B4513;
}

.coin-face.back {
    background: radial-gradient(circle, #CD853F, #8B4513);
    color: #FFD700;
    transform: rotateY(180deg);
}

@keyframes coinFlip {
    0% {
        transform: translateY(0) rotateY(0);
    }
    20% {
        transform: translateY(-100px) rotateY(360deg);
    }
    50% {
        transform: translateY(-80px) rotateY(720deg);
    }
    80% {
        transform: translateY(-40px) rotateY(1080deg);
    }
    100% {
        transform: translateY(0) rotateY(1440deg);
    }
}
```

### 3. 悬停显示详细解释

**设计**: 鼠标悬停在卦象或爻位上，显示详细的解释

```html
<div class="yao-container"
     data-yao="chu"
     data-nature="坤，地，承载"
     data-meaning="初爻，事之初始，当顺势而为">
  <svg class="yao-symbol">...</svg>
</div>

<!-- 提示框（Tooltip） -->
<div class="yao-tooltip" id="yaoTooltip">
  <div class="tooltip-header">初爻</div>
  <div class="tooltip-nature">坤，地，承载</div>
  <div class="tooltip-meaning">
    初爻，事之初始，当顺势而为
  </div>
</div>
```

```javascript
class YaoTooltip {
    constructor() {
        this.tooltip = document.getElementById('yaoTooltip');
        this.initListeners();
    }

    initListeners() {
        document.querySelectorAll('.yao-container').forEach(yao => {
            yao.addEventListener('mouseenter', (e) => {
                this.show(e.currentTarget);
            });

            yao.addEventListener('mouseleave', () => {
                this.hide();
            });

            yao.addEventListener('mousemove', (e) => {
                this.updatePosition(e.clientX, e.clientY);
            });
        });
    }

    show(yaoElement) {
        const header = yaoElement.dataset.yao;
        const nature = yaoElement.dataset.nature;
        const meaning = yaoElement.dataset.meaning;

        this.tooltip.querySelector('.tooltip-header').textContent = header;
        this.tooltip.querySelector('.tooltip-nature').textContent = nature;
        this.tooltip.querySelector('.tooltip-meaning').textContent = meaning;

        this.tooltip.classList.add('visible');
    }

    hide() {
        this.tooltip.classList.remove('visible');
    }

    updatePosition(x, y) {
        this.tooltip.style.left = `${x + 15}px`;
        this.tooltip.style.top = `${y + 15}px`;
    }
}
```

```css
.yao-tooltip {
    position: fixed;
    background: rgba(0, 0, 0, 0.9);
    border: 1px solid #FFD700;
    border-radius: 8px;
    padding: 12px;
    max-width: 250px;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.3s;
    z-index: 1000;
    backdrop-filter: blur(10px);
}

.yao-tooltip.visible {
    opacity: 1;
}

.tooltip-header {
    font-size: 16px;
    font-weight: bold;
    color: #FFD700;
    margin-bottom: 8px;
    border-bottom: 1px solid rgba(255, 215, 0, 0.3);
    padding-bottom: 4px;
}

.tooltip-nature {
    font-size: 13px;
    color: #FFA500;
    margin-bottom: 6px;
}

.tooltip-meaning {
    font-size: 12px;
    color: #CCC;
    line-height: 1.5;
}
```

---

## 🔊 音效系统

### 1. 音效库设计

```javascript
class SoundSystem {
    constructor() {
        this.sounds = {
            qigua: new Audio('/sounds/qigua.mp3'),        // 起卦钟声
            chenggua: new Audio('/sounds/chenggua.mp3'),  // 成卦磬声
            yaochange: new Audio('/sounds/yaochange.mp3'),// 爻变音效
            coinflip: new Audio('/sounds/coinflip.mp3'),  // 铜钱翻动
            success: new Audio('/sounds/success.mp3'),    // 执行成功
            error: new Audio('/sounds/error.mp3')         // 执行失败
        };

        this.enabled = true;
        this.volume = 0.5;

        // 预加载
        Object.values(this.sounds).forEach(sound => {
            sound.volume = this.volume;
            sound.load();
        });
    }

    play(name) {
        if (!this.enabled) return;

        const sound = this.sounds[name];
        if (sound) {
            sound.currentTime = 0;
            sound.play().catch(err => {
                console.warn('Sound play failed:', err);
            });
        }
    }

    setVolume(volume) {
        this.volume = Math.max(0, Math.min(1, volume));
        Object.values(this.sounds).forEach(sound => {
            sound.volume = this.volume;
        });
    }

    toggle() {
        this.enabled = !this.enabled;
    }
}

// 使用
const soundSystem = new SoundSystem();

// 起卦时
soundSystem.play('qigua');

// 成卦时
soundSystem.play('chenggua');

// 爻变时
soundSystem.play('yaochange');
```

### 2. 用户控制

```html
<div class="sound-controls">
  <button class="sound-toggle" id="soundToggle">
    <span class="icon-sound-on">🔊</span>
    <span class="icon-sound-off" style="display: none;">🔇</span>
  </button>
  <input type="range" min="0" max="100" value="50"
         class="volume-slider" id="volumeSlider">
</div>
```

---

## 📱 响应式设计

### 1. 移动端适配

```css
/* 桌面端（默认） */
.hexagram-container {
    width: 400px;
    margin: 0 auto;
}

.yao {
    width: 200px;
    height: 24px;
}

/* 平板端 */
@media (max-width: 768px) {
    .hexagram-container {
        width: 300px;
    }

    .yao {
        width: 150px;
        height: 20px;
    }
}

/* 手机端 */
@media (max-width: 480px) {
    .hexagram-container {
        width: 250px;
    }

    .yao {
        width: 120px;
        height: 16px;
    }

    /* 简化动画（性能考虑） */
    .divination-animation {
        animation-duration: 0.5s;
    }
}
```

### 2. 触摸手势支持

```javascript
class TouchGestures {
    constructor(element) {
        this.element = element;
        this.initGestures();
    }

    initGestures() {
        let startX, startY;

        this.element.addEventListener('touchstart', (e) => {
            startX = e.touches[0].clientX;
            startY = e.touches[0].clientY;
        });

        this.element.addEventListener('touchend', (e) => {
            const endX = e.changedTouches[0].clientX;
            const endY = e.changedTouches[0].clientY;

            const deltaX = endX - startX;
            const deltaY = endY - startY;

            // 检测滑动方向
            if (Math.abs(deltaX) > Math.abs(deltaY)) {
                // 水平滑动
                if (deltaX > 50) {
                    this.onSwipeRight();
                } else if (deltaX < -50) {
                    this.onSwipeLeft();
                }
            } else {
                // 垂直滑动
                if (deltaY > 50) {
                    this.onSwipeDown();
                } else if (deltaY < -50) {
                    this.onSwipeUp();
                }
            }
        });
    }

    onSwipeLeft() {
        // 切换到下一个卦象
        console.log('Swipe left: next hexagram');
    }

    onSwipeRight() {
        // 切换到上一个卦象
        console.log('Swipe right: previous hexagram');
    }

    // ...
}
```

---

## 💾 状态持久化

### 1. LocalStorage 保存用户偏好

```javascript
class UserPreferences {
    constructor() {
        this.prefs = this.load();
    }

    load() {
        const saved = localStorage.getItem('divination_prefs');
        return saved ? JSON.parse(saved) : {
            animationSpeed: 'normal',
            soundEnabled: true,
            soundVolume: 0.5,
            showYarrowAnimation: true,
            showHexagram: true,
            showJudgement: true,
            theme: 'dark'
        };
    }

    save() {
        localStorage.setItem('divination_prefs', JSON.stringify(this.prefs));
    }

    get(key) {
        return this.prefs[key];
    }

    set(key, value) {
        this.prefs[key] = value;
        this.save();
    }
}

const userPrefs = new UserPreferences();
```

### 2. 保存占卜历史

```javascript
class DivinationHistory {
    constructor() {
        this.history = this.load();
    }

    load() {
        const saved = localStorage.getItem('divination_history');
        return saved ? JSON.parse(saved) : [];
    }

    save() {
        // 只保留最近100次
        const recent = this.history.slice(-100);
        localStorage.setItem('divination_history', JSON.stringify(recent));
    }

    add(divination) {
        this.history.push({
            ...divination,
            timestamp: new Date().toISOString()
        });
        this.save();
    }

    getRecent(count = 10) {
        return this.history.slice(-count).reverse();
    }
}
```

---

## 🎭 主题系统

### 1. 多套配色方案

```css
/* 暗色主题（默认） */
:root {
    --bg-primary: #0a0a0a;
    --bg-secondary: #1a1a1a;
    --text-primary: #ffffff;
    --text-secondary: #cccccc;
    --accent-gold: #FFD700;
    --accent-orange: #FFA500;
}

/* 亮色主题 */
[data-theme="light"] {
    --bg-primary: #f5f5f5;
    --bg-secondary: #ffffff;
    --text-primary: #333333;
    --text-secondary: #666666;
    --accent-gold: #B8860B;
    --accent-orange: #FF8C00;
}

/* 古典主题 */
[data-theme="classical"] {
    --bg-primary: #2c1810;
    --bg-secondary: #3d2317;
    --text-primary: #f4e4c1;
    --text-secondary: #d4c4a1;
    --accent-gold: #d4af37;
    --accent-orange: #cd853f;
}
```

### 2. 主题切换

```javascript
class ThemeManager {
    constructor() {
        this.currentTheme = userPrefs.get('theme') || 'dark';
        this.apply();
    }

    apply() {
        document.documentElement.setAttribute('data-theme', this.currentTheme);
    }

    switch(theme) {
        this.currentTheme = theme;
        this.apply();
        userPrefs.set('theme', theme);
    }
}
```

---

## 📊 性能优化

### 1. 按需加载资源

```javascript
class ResourceLoader {
    constructor() {
        this.loaded = {};
    }

    async loadCanvas() {
        if (this.loaded.canvas) return;

        // 动态导入 Canvas 相关代码
        const { YarrowStalksAnimation } = await import('./yarrow-animation.js');
        this.loaded.canvas = true;
    }

    async loadSounds() {
        if (this.loaded.sounds) return;

        // 延迟加载音效
        const { SoundSystem } = await import('./sound-system.js');
        this.loaded.sounds = true;
    }
}
```

### 2. 动画帧率控制

```javascript
class AnimationController {
    constructor() {
        this.fps = 60;
        this.fpsInterval = 1000 / this.fps;
        this.then = Date.now();
    }

    animate(callback) {
        requestAnimationFrame(() => this.animate(callback));

        const now = Date.now();
        const elapsed = now - this.then;

        if (elapsed > this.fpsInterval) {
            this.then = now - (elapsed % this.fpsInterval);
            callback();
        }
    }
}
```

---

## ✨ 特殊效果

### 1. 背景粒子流动

```javascript
class BackgroundParticles {
    constructor(canvasId) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.particles = [];

        // 创建100个背景粒子
        for (let i = 0; i < 100; i++) {
            this.particles.push({
                x: Math.random() * this.canvas.width,
                y: Math.random() * this.canvas.height,
                vx: (Math.random() - 0.5) * 0.5,
                vy: (Math.random() - 0.5) * 0.5,
                size: Math.random() * 2,
                opacity: Math.random() * 0.5
            });
        }

        this.animate();
    }

    animate() {
        this.ctx.fillStyle = 'rgba(10, 10, 10, 0.1)';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

        this.particles.forEach(p => {
            p.x += p.vx;
            p.y += p.vy;

            // 边界反弹
            if (p.x < 0 || p.x > this.canvas.width) p.vx *= -1;
            if (p.y < 0 || p.y > this.canvas.height) p.vy *= -1;

            // 绘制粒子
            this.ctx.fillStyle = `rgba(255, 215, 0, ${p.opacity})`;
            this.ctx.beginPath();
            this.ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
            this.ctx.fill();

            // 绘制连接线
            this.particles.forEach(other => {
                const dx = other.x - p.x;
                const dy = other.y - p.y;
                const distance = Math.sqrt(dx * dx + dy * dy);

                if (distance < 100) {
                    const opacity = (1 - distance / 100) * 0.2;
                    this.ctx.strokeStyle = `rgba(255, 215, 0, ${opacity})`;
                    this.ctx.lineWidth = 0.5;
                    this.ctx.beginPath();
                    this.ctx.moveTo(p.x, p.y);
                    this.ctx.lineTo(other.x, other.y);
                    this.ctx.stroke();
                }
            });
        });

        requestAnimationFrame(() => this.animate());
    }
}
```

---

## 🎯 完整的占卜流程（Web 版）

### 用户体验时间线

```
0s: 用户输入 "/decompose 查找所有 Rust 文件"
    ↓ [提交]

0.1s: 【起卦动画开始】
    - 六个圆点旋转闪烁 ⚪⚪⚪ → ⚫⚪⚪
    - 音效：钟声 🔔
    - 背景粒子开始聚集

0.3s: 【演算动画】
    - Canvas 显示49根蓍草
    - 数字变化：49 → 38 → 30 → 22 → 14 → 6
    - 操作文字："分二"、"挂一"、"揲四"、"归奇"
    - 蓍草动画：分堆、聚合

0.8s: 【成卦动画】
    - 爻画从下往上生成（每50ms一爻）
    - 初爻 ——（金色光效）
    - 二爻 ——
    - 三爻 -- --
    - 四爻 ——
    - 五爻 ——
    - 上爻 -- --
    - 粒子爆发效果 ✨
    - 音效：磬声 🔔

1.1s: 【卦象显示】
    - 卦象符号：☵☲
    - 卦名：【水火既济】
    - 卦辞：既济：亨小，利贞。初吉终乱。
    - AI理解：您想要查找所有 Rust 源文件...

1.3s: 【步骤展开】
    - 6个步骤逐个显示
    - 每个步骤带有爻位标记
    - 初爻：检查当前目录（☷坤）
    - 二爻：遍历子目录（☴巽）
    - ...
    - 用户可悬停查看详细解释

1.5s: 【交互就绪】
    - "修改计划"按钮可点击
    - "执行计划"按钮可点击
    - "抛卦重新占卜"按钮可点击

[用户点击"修改计划"]
    ↓
    - 每个步骤前显示 checkbox
    - 可拖拽调整顺序
    - 可调整参数滑动条
    - 实时显示"变爻"效果

[用户点击"确认"]
    ↓
    - 显示"本卦 → 变爻 → 之卦"对比
    - 高亮变化的爻位
    - 音效：爻变声 🎵

[用户点击"执行计划"]
    ↓
    - 步骤逐个执行，实时更新状态
    - 初爻：运行中 ⏳ → 成功 ✅（0.3s）
    - 二爻：运行中 ⏳ → 成功 ✅（0.2s）
    - ...

执行完成:
    - 显示"之卦"最终状态
    - 粒子爆发庆祝效果 🎆
    - 音效：成功声 🎉
    - 执行统计：6/6 成功，用时2.3s
```

---

## 📝 总结：Web 界面的独特价值

| 传统 CLI | 当前 Web | 增强后的 Web（占卜系统） |
|---------|---------|------------------------|
| 纯文本 | 简单 HTML | 丰富动画 + 粒子效果 |
| 线性输出 | 卡片展示 | 卦象可视化 + 交互 |
| 无动画 | 基础过渡 | Canvas 演算 + SVG 生成 |
| 无音效 | 无音效 | 古典音效系统 |
| 无交互 | 按钮点击 | 拖拽、悬停、手势 |
| 无状态 | Session | LocalStorage 持久化 |
| 单一主题 | 单一主题 | 多主题系统 |

**核心优势**：
1. ✨ **视觉冲击力**：动画、粒子、光效
2. 🎮 **交互丰富性**：拖拽、悬停、手势
3. 🔊 **多感官体验**：视觉 + 听觉
4. 📱 **跨设备支持**：桌面 + 移动
5. 💾 **状态持久化**：偏好 + 历史
6. 🎨 **高度可定制**：主题 + 配置

**文化价值**：
- 让易经智慧在现代浏览器中焕发新生
- 用技术手段传承古老文化
- 创造独特的东方美学体验

---

**作者**: Claude Code
**字数**: 约10000字
**完成时间**: 深夜约2小时
**状态**: ✅ Web UI 设计完成，待技术实现
