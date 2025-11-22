# 可视化功能 Phase 2 实施计划

> **开始时间**: 2025-01-22
> **预计版本**: v1.45.0 - v1.46.0
> **目标周期**: 2-3 周

## 执行摘要

基于 MVP (v1.44.0) 的成功实施，Phase 2 将专注于三个核心方向：
1. **完善图表类型**：实现饼图、散点图，完整覆盖 4 种基础图表
2. **数据文件支持**：CSV/JSON 导入，大幅提升实用性
3. **增强交互体验**：图例筛选、数据导出、工具提示优化

## 优先级排序（一分为三原则）

### P0 - 核心功能（必须完成）
**目标**: 完整实现 ChartType 枚举中已定义但未实现的图表类型

1. **饼图实现** (v1.45.0)
   - 解析器扩展：支持饼图特定参数
   - 前端渲染：ECharts 饼图配置
   - 测试用例：至少 3 个饼图场景

2. **散点图实现** (v1.45.0)
   - 二维数据解析：`--data "x1,y1 x2,y2"`
   - 前端渲染：散点图样式和主题
   - 测试用例：散点分布、趋势分析

### P1 - 高价值功能（强烈建议）
**目标**: 实现最高性价比的增强功能

3. **CSV 文件支持** (v1.45.0)
   - 文件读取工具：`!chart csv <file> --type <chart_type>`
   - 自动列检测：第一行作为 header
   - 错误处理：文件不存在、格式错误

4. **数据导出** (v1.46.0)
   - 前端导出：PNG/SVG 图片
   - 数据导出：CSV 格式
   - UI 集成：Round 卡片添加导出按钮

### P2 - 体验优化（根据时间决定）
**目标**: 提升用户交互体验

5. **图例交互** (v1.46.0)
   - 图例点击切换系列显示/隐藏
   - 多系列图表必备功能

6. **数据预览** (v1.46.0)
   - CSV 导入前预览数据表格
   - 列选择器：用户选择 X 轴和系列

## 详细实施路线

### 阶段 1: 饼图实现 (3-4 小时)

**任务分解**:
1. **解析器扩展** (`src/visualization/parser.rs`)
   ```rust
   // 支持饼图特定语法
   !chart pie --title "市场份额" --series "产品A:35,产品B:25,产品C:40"
   ```
   - 饼图不需要 X 轴，series 名称作为扇区标签
   - 支持 `--labels "A,B,C"` 可选参数

2. **数据结构调整** (`src/visualization/types.rs`)
   - 为 `ChartData` 添加 `labels: Option<Vec<String>>` 字段
   - 饼图验证逻辑：labels 长度与 data 长度匹配

3. **前端渲染** (`src/web/frontend.rs`)
   ```javascript
   case 'pie':
       option = {
           series: [{
               type: 'pie',
               radius: '70%',
               data: seriesData.map((item, index) => ({
                   name: labels[index],
                   value: item.data[0],
                   itemStyle: { color: themeColors[index % themeColors.length] }
               })),
               label: { formatter: '{b}: {c} ({d}%)' }
           }]
       };
   ```

4. **测试用例**
   - 简单饼图
   - 环形图（`--donut` 选项）
   - 玫瑰图（`--rose` 选项）

**验收标准**:
- ✅ 3 个单元测试通过
- ✅ 饼图正确渲染
- ✅ 鼠标悬停显示百分比
- ✅ 主题颜色正确应用

### 阶段 2: 散点图实现 (3-4 小时)

**任务分解**:
1. **解析器扩展**
   ```rust
   // 支持二维数据
   !chart scatter --title "身高体重分布" --data "170,65 175,70 160,55 180,80"
   ```
   - 新增 `--data` 参数解析
   - 每个点格式：`x,y`，点之间空格分隔

2. **数据结构**
   - `Series` 结构添加 `points: Option<Vec<(f64, f64)>>`
   - 散点图验证：points 不为空

3. **前端渲染**
   ```javascript
   case 'scatter':
       option = {
           xAxis: { type: 'value' },
           yAxis: { type: 'value' },
           series: [{
               type: 'scatter',
               symbolSize: 10,
               data: series.points.map(([x, y]) => [x, y])
           }]
       };
   ```

4. **测试用例**
   - 简单散点分布
   - 多系列散点（不同类别）
   - 气泡图（`--size` 参数控制大小）

**验收标准**:
- ✅ 散点图正确渲染
- ✅ 支持多系列对比
- ✅ Tooltip 显示坐标值
- ✅ 缩放和平移功能正常

### 阶段 3: CSV 文件支持 (4-5 小时)

**任务分解**:
1. **CSV 解析库集成**
   ```toml
   [dependencies]
   csv = "1.3"  # 轻量级 CSV 解析器
   ```

2. **文件读取工具** (`src/visualization/csv.rs`)
   ```rust
   pub fn parse_csv_file(path: &str) -> Result<CsvData> {
       let mut reader = csv::Reader::from_path(path)?;
       let headers = reader.headers()?.clone();
       let records: Vec<Vec<String>> = reader.records()
           .map(|r| r?.iter().map(|s| s.to_string()).collect())
           .collect::<Result<_, _>>()?;
       Ok(CsvData { headers, records })
   }
   ```

3. **命令语法**
   ```bash
   !chart csv sales.csv --type line --x-col "月份" --y-col "销售额"
   ```

4. **前端文件上传**（可选，暂不实现）
   - Phase 2 先支持服务器本地文件
   - Phase 3 再考虑浏览器文件上传

5. **测试用例**
   - 读取简单 CSV
   - 多列数据
   - 错误处理（文件不存在、格式错误）

**验收标准**:
- ✅ 成功读取 CSV 文件
- ✅ 自动识别列名
- ✅ 生成对应图表
- ✅ 错误提示友好

### 阶段 4: 数据导出 (3-4 小时)

**任务分解**:
1. **前端导出按钮**
   - Round 卡片右上角添加导出菜单
   - 选项：PNG、SVG、CSV

2. **图片导出**（ECharts 内置）
   ```javascript
   function exportChart(format) {
       const url = chartInstance.getDataURL({
           type: format,  // 'png' or 'svg'
           pixelRatio: 2,
           backgroundColor: '#fff'
       });
       // 下载文件
       const link = document.createElement('a');
       link.href = url;
       link.download = `chart-${Date.now()}.${format}`;
       link.click();
   }
   ```

3. **数据导出**
   ```javascript
   function exportData() {
       const csvContent = chartData.series.map(s =>
           [s.name, ...s.data].join(',')
       ).join('\n');
       // 下载 CSV
   }
   ```

**验收标准**:
- ✅ 导出按钮 UI 美观
- ✅ PNG 导出清晰
- ✅ CSV 导出数据完整
- ✅ 文件名包含时间戳

### 阶段 5: 图例交互 (2-3 小时)

**任务分解**:
1. **ECharts 配置**
   ```javascript
   legend: {
       selected: {},  // 空对象表示默认全选
       selectedMode: 'multiple'  // 允许多选
   }
   ```

2. **事件监听**（可选）
   ```javascript
   chartInstance.on('legendselectchanged', (params) => {
       console.log('图例选择变化:', params.selected);
   });
   ```

**验收标准**:
- ✅ 点击图例切换系列显示
- ✅ 多系列图表支持部分隐藏
- ✅ 图例状态视觉反馈明显

## 技术风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| CSV 解析性能（大文件） | 中 | 中 | 限制文件大小（1MB），异步读取 |
| 饼图数据验证复杂度 | 低 | 低 | 复用现有验证框架 |
| 散点图数据格式冲突 | 低 | 低 | 明确文档和错误提示 |
| 导出功能浏览器兼容性 | 低 | 低 | 使用标准 API + Polyfill |

## 代码统计预估

| 功能 | 新增代码 | 修改代码 | 测试代码 |
|------|---------|---------|---------|
| 饼图 | ~150 行 | ~50 行 | ~80 行 |
| 散点图 | ~180 行 | ~50 行 | ~80 行 |
| CSV 支持 | ~200 行 | ~30 行 | ~100 行 |
| 数据导出 | ~120 行 | ~20 行 | ~50 行 |
| 图例交互 | ~50 行 | ~10 行 | ~30 行 |
| **总计** | **~700 行** | **~160 行** | **~340 行** |

## 测试策略

### 单元测试
- **解析器**: 每种新图表类型 3+ 测试
- **数据验证**: 边界条件和错误场景
- **CSV 解析**: 各种 CSV 格式

### 集成测试
- **端到端脚本**: `scripts/test/test_chart_phase2.sh`
- **测试用例**: 每个功能至少 2 个场景

### 性能测试
- **大数据集**: 1000+ 点散点图
- **大 CSV**: 1MB 文件读取时间

## 文档计划

1. **用户文档** (`docs/02-practice/user/visualization-guide.md`)
   - 所有图表类型示例
   - CSV 导入教程
   - 导出功能说明

2. **开发者文档** (`docs/02-practice/developer/visualization-api.md`)
   - ChartData API 完整说明
   - 扩展新图表类型指南

3. **更新日志** (`CHANGELOG.md`)
   - v1.45.0 和 v1.46.0 发布说明

## 发布计划

### v1.45.0 - 图表完善版
**发布时间**: 实施开始后 1 周

**包含功能**:
- ✅ 饼图完整实现
- ✅ 散点图完整实现
- ✅ CSV 文件支持
- ✅ 测试覆盖 85%+

### v1.46.0 - 交互增强版
**发布时间**: 实施开始后 2-3 周

**包含功能**:
- ✅ 数据导出（PNG/SVG/CSV）
- ✅ 图例交互
- ✅ 完整文档
- ✅ 性能优化

## 成功指标

1. **功能完整性**: 4 种图表类型全部实现 ✅
2. **测试覆盖率**: ≥ 85% ✅
3. **性能指标**:
   - 1000 点散点图渲染 < 1s
   - 1MB CSV 读取 < 2s
4. **用户体验**:
   - 导出功能使用流畅
   - 错误提示清晰
5. **代码质量**:
   - Clippy 零警告
   - 文档完整

## 下一步行动

1. **立即开始**: 饼图实现（最高优先级）
2. **并行准备**: CSV 解析库调研
3. **文档先行**: 更新用户指南框架

---

**制定时间**: 2025-01-22
**制定者**: Claude + User
**批准状态**: ✅ 待用户批准
