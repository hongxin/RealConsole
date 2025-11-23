//! 元数据提取器统一架构
//!
//! v1.53.0: 统一 Chart/Image 等元数据提取逻辑，消除重复代码
//!
//! ## 设计理念
//!
//! - **一分为三**: 类型维度（Chart/Image/...）+ 职责维度（提取/解析/验证）+ 状态维度（Success/Partial/Failure）
//! - **极简主义**: 80% 重复代码消除，新类型只需 15 行
//! - **易变适应**: 通过 trait 轻松扩展新元数据类型
//!
//! ## 使用示例
//!
//! ```rust
//! // Chart 提取
//! let extractor = ChartExtractor;
//! let result = extractor.extract(response);
//! if let Some(chart) = result.metadata {
//!     // 处理图表数据
//! }
//!
//! // Image 提取
//! let extractor = ImageExtractor;
//! let result = extractor.extract(content);
//! ```

use std::time::Instant;

/// 元数据提取器统一接口
///
/// 每种元数据类型（Chart, Image, 未来的 Table/Video 等）都实现此 trait
pub trait MetadataExtractor: Send + Sync {
    /// 元数据类型
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
    fn extract(&self, content: &str) -> ExtractionResult<Self::Metadata>
    where
        Self: Sized,
    {
        extract_metadata_generic(self, content)
    }
}

/// 提取结果
#[derive(Debug)]
pub struct ExtractionResult<T> {
    /// 清理后的内容（移除了元数据标记）
    pub clean_content: String,
    /// 提取的元数据（如果成功）
    pub metadata: Option<T>,
    /// 提取过程的指标（用于监控）
    pub metrics: ExtractionMetrics,
}

/// 提取指标（监控和调试）
#[derive(Debug, Default)]
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

/// 通用元数据提取逻辑（所有类型共享）
///
/// ## 提取流程
///
/// 1. 查找主标记（如 `__CHART__`）
/// 2. 提取标记后的部分
/// 3. 查找数据标记（如 `__CHART_DATA__:`）
/// 4. 提取 JSON 字符串
/// 5. 解析元数据
/// 6. 验证元数据
/// 7. 清理内容（移除标记）
/// 8. 记录监控指标
fn extract_metadata_generic<T, E>(extractor: &E, content: &str) -> ExtractionResult<T>
where
    E: MetadataExtractor<Metadata = T>,
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let start_time = Instant::now();
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
        eprintln!(
            "[MetadataExtractor] {} data marker not found",
            extractor.primary_marker()
        );
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
            eprintln!(
                "[MetadataExtractor] {} parse failed: {}",
                extractor.primary_marker(),
                e
            );
            eprintln!(
                "[MetadataExtractor] JSON preview (first 100 chars): {}",
                &json_str.chars().take(100).collect::<String>()
            );
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
        eprintln!(
            "[MetadataExtractor] {} validation failed: {}",
            extractor.primary_marker(),
            e
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    // 测试用的简单元数据类型
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestMetadata {
        value: String,
    }

    // 测试提取器
    struct TestExtractor;

    impl MetadataExtractor for TestExtractor {
        type Metadata = TestMetadata;

        fn primary_marker(&self) -> &'static str {
            "__TEST__"
        }

        fn data_marker(&self) -> &'static str {
            "__TEST_DATA__:"
        }

        fn parse_metadata(&self, json_str: &str) -> anyhow::Result<TestMetadata> {
            Ok(serde_json::from_str(json_str)?)
        }
    }

    #[test]
    fn test_extraction_success() {
        // 注意：实际使用中，remove_debug_info 会移除 __DEBUG__ 及之后的内容
        // 所以 JSON 后面不会有额外内容
        let content = r#"Some prefix__TEST____TEST_DATA__:{"value":"test"}"#;
        let extractor = TestExtractor;
        let result = extractor.extract(content);

        assert!(result.metadata.is_some());
        assert_eq!(result.metadata.unwrap().value, "test");
        assert_eq!(result.clean_content, "Some prefix");
        assert!(result.metrics.found_primary_marker);
        assert!(result.metrics.found_data_marker);
        assert!(result.metrics.json_parse_success);
        assert!(result.metrics.validation_success);
        assert!(result.metrics.error.is_none());
        assert!(result.metrics.duration_ns > 0);
    }

    #[test]
    fn test_no_primary_marker() {
        let content = "No markers here";
        let extractor = TestExtractor;
        let result = extractor.extract(content);

        assert!(result.metadata.is_none());
        assert_eq!(result.clean_content, content);
        assert!(!result.metrics.found_primary_marker);
    }

    #[test]
    fn test_no_data_marker() {
        let content = "__TEST__ but no data marker";
        let extractor = TestExtractor;
        let result = extractor.extract(content);

        assert!(result.metadata.is_none());
        assert!(result.metrics.found_primary_marker);
        assert!(!result.metrics.found_data_marker);
        assert!(result.metrics.error.is_some());
    }

    #[test]
    fn test_invalid_json() {
        let content = r#"__TEST____TEST_DATA__:{invalid json}"#;
        let extractor = TestExtractor;
        let result = extractor.extract(content);

        assert!(result.metadata.is_none());
        assert!(result.metrics.found_primary_marker);
        assert!(result.metrics.found_data_marker);
        assert!(!result.metrics.json_parse_success);
        assert!(result.metrics.error.is_some());
        assert!(result.metrics.error.as_ref().unwrap().contains("Parse error"));
    }

    #[test]
    fn test_validation_failure() {
        // 带验证的提取器
        struct ValidatingExtractor;

        impl MetadataExtractor for ValidatingExtractor {
            type Metadata = TestMetadata;

            fn primary_marker(&self) -> &'static str {
                "__VALID__"
            }

            fn data_marker(&self) -> &'static str {
                "__VALID_DATA__:"
            }

            fn parse_metadata(&self, json_str: &str) -> anyhow::Result<TestMetadata> {
                Ok(serde_json::from_str(json_str)?)
            }

            fn validate(&self, metadata: &TestMetadata) -> anyhow::Result<()> {
                if metadata.value.is_empty() {
                    anyhow::bail!("Value cannot be empty");
                }
                Ok(())
            }
        }

        let content = r#"__VALID____VALID_DATA__:{"value":""}"#;
        let extractor = ValidatingExtractor;
        let result = extractor.extract(content);

        assert!(result.metadata.is_none());
        assert!(result.metrics.json_parse_success);
        assert!(!result.metrics.validation_success);
        assert!(result.metrics.error.is_some());
        assert!(result
            .metrics
            .error
            .as_ref()
            .unwrap()
            .contains("Validation error"));
    }

    #[test]
    fn test_metrics_collection() {
        let content = r#"Prefix__TEST____TEST_DATA__:{"value":"test"}"#;
        let extractor = TestExtractor;
        let result = extractor.extract(content);

        // 验证所有指标都被正确收集
        assert!(result.metrics.found_primary_marker);
        assert!(result.metrics.found_data_marker);
        assert!(result.metrics.json_parse_success);
        assert!(result.metrics.validation_success);
        assert!(result.metrics.error.is_none());
        assert!(result.metrics.duration_ns > 0);
    }
}
