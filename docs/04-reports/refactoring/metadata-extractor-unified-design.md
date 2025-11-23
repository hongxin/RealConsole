# 元数据提取器统一架构设计

**日期**: 2025-11-24
**版本**: v1.53.0
**类型**: 架构重构
**优先级**: 高

---

## 🎯 重构目标

### 问题陈述

当前 Chart 和 Image 提取逻辑存在显著重复：

```rust
// Chart 提取 (websocket.rs:2539)
fn extract_and_process_chart_data(response: &str) -> (String, Option<ChartData>) {
    if let Some(pos) = response.find("__CHART__") {
        let section = &response[pos + 9..];
        if let Some(data_pos) = section.find("__CHART_DATA__:") {
            let json_str = &section[data_pos + 15..];
            // ... 解析和转换
        }
    }
}

// Image 提取 (websocket.rs:2685)
fn extract_and_process_image_data(content: &str) -> (String, Option<ImageData>) {
    if let Some(start) = content.find("__IMAGE__") {
        let section = &content[start + 9..];
        if let Some(data_pos) = section.find("__IMAGE_DATA__:") {
            let json_str = &section[data_pos + 15..];
            // ... 解析和验证
        }
    }
}
```

**重复模式**：
1. 查找类型标记（`__CHART__`, `__IMAGE__`）
2. 查找数据标记（`__CHART_DATA__:`, `__IMAGE_DATA__:`）
3. 提取 JSON 字符串
4. 解析/验证数据
5. 清理内容（移除标记）

**问题**：
- ❌ 代码重复率 ~80%
- ❌ 每增加新类型需要复制整个模式
- ❌ 字符串标记魔法值分散
- ❌ 错误处理不统一
- ❌ 缺少监控和日志

### 设计目标

1. **统一抽象** - 一个 trait 支持所有元数据类型
2. **零重复** - 提取逻辑只写一次
3. **易扩展** - 新类型只需实现一个小接口
4. **强类型** - 减少字符串依赖，编译期检查
5. **可监控** - 统一的错误处理和日志

---

## 🏗️ 架构设计

### 1. 核心 Trait: MetadataExtractor

```rust
/// 元数据提取器统一接口
///
/// 每种元数据类型（Chart, Image, 未来的 Table/Video 等）都实现此 trait
pub trait MetadataExtractor: Send + Sync {
    /// 元数据类型（用于日志和监控）
    type Metadata: serde::Serialize + serde::de::DeserializeOwned;

    /// 主标记（如 "__CHART__"）
    fn primary_marker(&self) -> &'static str;

    /// 数据标记前缀（如 "__CHART_DATA__:"）
    fn data_marker(&self) -> &'static str;

    /// 从 JSON 解析元数据（子类特定逻辑）
    fn parse_metadata(&self, json_str: &str) -> anyhow::Result<Self::Metadata>;

    /// 验证元数据有效性（可选，默认为空）
    fn validate(&self, _metadata: &Self::Metadata) -> anyhow::Result<()> {
        Ok(())
    }

    /// 提取元数据（通用实现，所有子类共享）
    fn extract(&self, content: &str) -> ExtractionResult<Self::Metadata> {
        extract_metadata_generic(self, content)
    }
}

/// 提取结果
pub struct ExtractionResult<T> {
    /// 清理后的内容（移除了元数据标记）
    pub clean_content: String,
    /// 提取的元数据（如果成功）
    pub metadata: Option<T>,
    /// 提取过程的指标（用于监控）
    pub metrics: ExtractionMetrics,
}

/// 提取指标（监控和调试）
pub struct ExtractionMetrics {
    /// 是否找到主标记
    pub found_primary_marker: bool,
    /// 是否找到数据标记
    pub found_data_marker: bool,
    /// JSON 解析是否成功
    pub json_parse_success: bool,
    /// 验证是否成功
    pub validation_success: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 耗时（纳秒）
    pub duration_ns: u64,
}
```

### 2. 通用提取算法

```rust
/// 通用元数据提取逻辑（所有类型共享）
fn extract_metadata_generic<T, E>(
    extractor: &E,
    content: &str,
) -> ExtractionResult<T>
where
    E: MetadataExtractor<Metadata = T>,
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let start_time = std::time::Instant::now();
    let mut metrics = ExtractionMetrics::default();

    // 1. 查找主标记
    let Some(primary_pos) = content.find(extractor.primary_marker()) else {
        return ExtractionResult {
            clean_content: content.to_string(),
            metadata: None,
            metrics,
        };
    };
    metrics.found_primary_marker = true;

    // 2. 提取标记后的部分
    let section = &content[primary_pos + extractor.primary_marker().len()..];

    // 3. 查找数据标记
    let Some(data_pos) = section.find(extractor.data_marker()) else {
        metrics.error = Some("Data marker not found".to_string());
        return ExtractionResult {
            clean_content: content.to_string(),
            metadata: None,
            metrics,
        };
    };
    metrics.found_data_marker = true;

    // 4. 提取 JSON 字符串
    let json_str = &section[data_pos + extractor.data_marker().len()..];

    // 5. 解析元数据
    let metadata = match extractor.parse_metadata(json_str) {
        Ok(data) => {
            metrics.json_parse_success = true;
            data
        }
        Err(e) => {
            metrics.error = Some(format!("Parse error: {}", e));
            log::error!("[MetadataExtractor] {} parse failed: {}",
                extractor.primary_marker(), e);
            return ExtractionResult {
                clean_content: content.to_string(),
                metadata: None,
                metrics,
            };
        }
    };

    // 6. 验证元数据
    if let Err(e) = extractor.validate(&metadata) {
        metrics.error = Some(format!("Validation error: {}", e));
        log::error!("[MetadataExtractor] {} validation failed: {}",
            extractor.primary_marker(), e);
        return ExtractionResult {
            clean_content: content.to_string(),
            metadata: None,
            metrics,
        };
    }
    metrics.validation_success = true;

    // 7. 清理内容
    let clean_content = content[..primary_pos].trim().to_string();

    // 8. 记录指标
    metrics.duration_ns = start_time.elapsed().as_nanos() as u64;

    ExtractionResult {
        clean_content,
        metadata: Some(metadata),
        metrics,
    }
}
```

### 3. Chart 提取器实现

```rust
pub struct ChartExtractor;

impl MetadataExtractor for ChartExtractor {
    type Metadata = ChartData;

    fn primary_marker(&self) -> &'static str {
        "__CHART__"
    }

    fn data_marker(&self) -> &'static str {
        "__CHART_DATA__:"
    }

    fn parse_metadata(&self, json_str: &str) -> anyhow::Result<ChartData> {
        // 解析工具参数 JSON
        let params: serde_json::Value = serde_json::from_str(json_str)?;

        // 转换为 ChartData（保留现有转换逻辑）
        convert_tool_params_to_chart_data(params)
    }
}

// 使用示例
fn extract_chart_data(response: &str) -> (String, Option<ChartData>) {
    let extractor = ChartExtractor;
    let result = extractor.extract(response);

    // 记录监控指标
    if let Some(ref err) = result.metrics.error {
        log::warn!("[Chart] Extraction failed: {}", err);
    }

    (result.clean_content, result.metadata)
}
```

### 4. Image 提取器实现

```rust
pub struct ImageExtractor;

impl MetadataExtractor for ImageExtractor {
    type Metadata = ImageData;

    fn primary_marker(&self) -> &'static str {
        "__IMAGE__"
    }

    fn data_marker(&self) -> &'static str {
        "__IMAGE_DATA__:"
    }

    fn parse_metadata(&self, json_str: &str) -> anyhow::Result<ImageData> {
        // 直接反序列化
        Ok(serde_json::from_str(json_str)?)
    }

    fn validate(&self, metadata: &ImageData) -> anyhow::Result<()> {
        // 使用现有验证逻辑
        metadata.validate()
    }
}

// 使用示例
fn extract_image_data(content: &str) -> (String, Option<ImageData>) {
    let extractor = ImageExtractor;
    let result = extractor.extract(content);

    (result.clean_content, result.metadata)
}
```

---

## 📊 收益分析

### 代码质量提升

| 指标 | 重构前 | 重构后 | 改进 |
|------|--------|--------|------|
| 提取函数数量 | 2 个 | 1 个通用 + 2 个小实现 | 统一 |
| 代码行数 | ~80 行 | ~120 行（含监控） | +50% 功能 |
| 重复代码率 | ~80% | 0% | -100% |
| 新增类型成本 | ~40 行 | ~15 行 | -62% |
| 错误处理覆盖 | 部分 | 完整 | +100% |
| 监控能力 | 无 | 完整指标 | +∞ |

### 可扩展性

**添加新类型（如 Table）只需**：

```rust
pub struct TableExtractor;

impl MetadataExtractor for TableExtractor {
    type Metadata = TableData;

    fn primary_marker(&self) -> &'static str { "__TABLE__" }
    fn data_marker(&self) -> &'static str { "__TABLE_DATA__:" }

    fn parse_metadata(&self, json_str: &str) -> anyhow::Result<TableData> {
        Ok(serde_json::from_str(json_str)?)
    }
}
```

**仅 15 行代码** vs 之前的 40 行！

---

## 🚀 实施计划

### Phase 1: 基础设施（2小时）

**文件**: `src/web/metadata_extractor.rs`（新建）

```
✅ 1.1 定义 MetadataExtractor trait
✅ 1.2 实现 extract_metadata_generic 函数
✅ 1.3 定义 ExtractionResult 和 ExtractionMetrics
✅ 1.4 添加单元测试
```

### Phase 2: Chart 迁移（1小时）

**文件**: `src/web/websocket.rs`

```
✅ 2.1 实现 ChartExtractor
✅ 2.2 重构 extract_and_process_chart_data 使用新提取器
✅ 2.3 验证现有测试通过
✅ 2.4 添加 Chart 提取器测试
```

### Phase 3: Image 迁移（30分钟）

**文件**: `src/web/websocket.rs`

```
✅ 3.1 实现 ImageExtractor
✅ 3.2 重构 extract_and_process_image_data 使用新提取器
✅ 3.3 验证现有测试通过
✅ 3.4 添加 Image 提取器测试
```

### Phase 4: 监控集成（1小时）

**文件**: `src/web/websocket.rs`

```
✅ 4.1 添加提取失败计数器
✅ 4.2 添加性能监控（提取耗时）
✅ 4.3 添加详细日志（debug level）
✅ 4.4 创建监控面板文档
```

### Phase 5: 文档和清理（30分钟）

```
✅ 5.1 更新架构文档
✅ 5.2 添加使用示例
✅ 5.3 标记旧代码为 deprecated
✅ 5.4 更新 CHANGELOG
```

**预计总时间**: 5 小时

---

## 🧪 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_extractor_success() {
        let content = r#"✅ 图表已生成__CHART____CHART_DATA__:{"chart_type":"line",...}"#;
        let extractor = ChartExtractor;
        let result = extractor.extract(content);

        assert!(result.metadata.is_some());
        assert!(result.metrics.found_primary_marker);
        assert!(result.metrics.json_parse_success);
        assert_eq!(result.clean_content, "✅ 图表已生成");
    }

    #[test]
    fn test_image_extractor_validation_failure() {
        let content = r#"__IMAGE____IMAGE_DATA__:{"invalid":"json"}"#;
        let extractor = ImageExtractor;
        let result = extractor.extract(content);

        assert!(result.metadata.is_none());
        assert!(result.metrics.error.is_some());
    }

    #[test]
    fn test_extractor_metrics() {
        // 测试监控指标收集
        // ...
    }
}
```

### 集成测试

- ✅ 现有 WebSocket 测试应全部通过
- ✅ Chart 渲染端到端测试
- ✅ Image 显示端到端测试
- ✅ 性能回归测试（提取不应变慢）

---

## 🎨 未来扩展

### 可能的新元数据类型

1. **Table** - 表格数据（v1.54.0）
2. **Video** - 视频链接/嵌入
3. **Audio** - 音频播放
4. **Code** - 代码高亮块
5. **Diff** - 代码差异展示

### 进一步优化

1. **异步提取** - 大型元数据并行处理
2. **缓存** - 重复内容快速返回
3. **流式解析** - 超大 JSON 不阻塞
4. **压缩** - 大型 base64 数据压缩传输

---

## 📝 总结

### 核心价值

1. **一分为三的体现**:
   - **类型维度**: Chart / Image / 未来
   - **职责维度**: 提取 / 解析 / 验证
   - **状态维度**: Success / Partial / Failure (通过 metrics)

2. **极简主义**:
   - 80% 重复代码消除
   - 新类型只需 15 行
   - 统一的错误处理

3. **易变适应**:
   - 轻松添加新类型
   - 监控驱动优化
   - 灵活的扩展点

### 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 现有功能破坏 | 低 | 高 | 保留旧函数，渐进迁移 |
| 性能退化 | 低 | 中 | 基准测试 + 监控 |
| 理解成本增加 | 中 | 低 | 详细文档 + 示例 |

### 成功标准

- ✅ 所有现有测试通过
- ✅ 代码重复率 < 20%
- ✅ 新增类型成本 < 20 行
- ✅ 提取性能无退化（±5%）
- ✅ 监控覆盖 100%

---

**设计者**: Claude + 用户
**审核**: 待定
**状态**: 设计完成，待实施
**下一步**: Phase 1 实施
