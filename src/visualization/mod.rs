// RealConsole 可视化模块
// v1.44.0: 基础可视化功能
//
// 模块职责：
// - 定义图表数据结构
// - 提供图表生成接口
// - 支持多种图表类型（折线、柱状、饼图、散点等）

pub mod csv;
pub mod examples;
pub mod parser;
pub mod templates;
pub mod types;

pub use csv::{parse_csv_file, CsvData};
pub use examples::{ChartExample, ExampleDifficulty, ExampleLibrary};
pub use parser::{ChartCommand, ChartCommandParser};
pub use templates::{ChartTemplate, TemplateCategory, TemplateEngine};
pub use types::{AxisConfig, ChartData, ChartOptions, ChartType, Series};

/// 可视化模块版本
pub const VERSION: &str = "1.0.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_data_serialization() {
        let chart = ChartData {
            chart_type: ChartType::Line,
            title: "测试图表".to_string(),
            x_axis: AxisConfig {
                name: None,
                data: Some(vec!["A".to_string(), "B".to_string()]),
                axis_type: Some("category".to_string()),
            },
            y_axis: AxisConfig {
                name: Some("值".to_string()),
                data: None,
                axis_type: Some("value".to_string()),
            },
            y_axis_secondary: None,  // v1.47.0
            series: vec![Series {
                name: "系列1".to_string(),
                data: vec![10.0, 20.0],
                color: None,
                points: None,
                sizes: None,         // v1.48.0
                y_axis_index: None,  // v1.47.0
                chart_type: None,    // v1.47.0
            }],
            options: ChartOptions {
                show_legend: true,
                show_toolbox: true,
                smooth: false,
            },
            labels: None,
            indicators: None,  // v1.49.0
            heatmap_data: None,  // v1.49.0
        };

        // 测试序列化
        let json = serde_json::to_string(&chart).expect("序列化失败");
        assert!(json.contains("line"));
        assert!(json.contains("测试图表"));

        // 测试反序列化
        let deserialized: ChartData = serde_json::from_str(&json).expect("反序列化失败");
        assert_eq!(deserialized.title, "测试图表");
        assert_eq!(deserialized.series.len(), 1);
    }
}
