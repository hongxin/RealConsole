//! Chart 命令解析器
//!
//! v1.44.0: 解析 !chart 命令并生成 ChartData

use crate::visualization::{AxisConfig, ChartData, ChartOptions, ChartType, Series};
use anyhow::{anyhow, Result};

/// Chart 命令解析器
pub struct ChartCommandParser;

impl ChartCommandParser {
    /// 解析 chart 命令
    ///
    /// 示例命令：
    /// ```
    /// !chart line --title "月度趋势" --x-axis "1月,2月,3月" --series "销售额:120,132,101"
    /// ```
    pub fn parse(command: &str) -> Result<ChartData> {
        // 移除 "chart" 前缀
        let command = command.trim_start_matches("chart").trim();

        // 提取图表类型（第一个参数）
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("缺少图表类型（line/bar/pie/scatter）"));
        }

        let chart_type = ChartType::from_str(parts[0])
            .ok_or_else(|| anyhow!("无效的图表类型: {}，支持: line, bar, pie, scatter", parts[0]))?;

        // 解析参数
        let args = &parts[1..].join(" ");
        let title = Self::extract_arg(args, "--title").unwrap_or_else(|| "图表".to_string());
        let x_labels = Self::extract_arg(args, "--x-axis")
            .map(|s| s.split(',').map(|l| l.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);
        let smooth = args.contains("--smooth");

        // v1.45.0: 解析饼图 labels（可选）
        let labels = Self::extract_arg(args, "--labels")
            .map(|s| s.split(',').map(|l| l.trim().to_string()).collect());

        // v1.45.0: 解析散点图的轴名称（可选）
        let x_name = Self::extract_arg(args, "--x-name");
        let y_name = Self::extract_arg(args, "--y-name");

        // v1.45.0: 散点图特殊处理
        let series = if chart_type == ChartType::Scatter {
            Self::extract_scatter_series(args)?
        } else {
            Self::extract_series(args)?
        };

        if series.is_empty() {
            return Err(anyhow!("至少需要一个数据系列，使用 --series \"名称:值1,值2,值3\""));
        }

        // 构建 ChartData
        let chart_data = ChartData {
            chart_type,
            title,
            x_axis: if chart_type == ChartType::Scatter {
                // 散点图使用数值轴
                AxisConfig::value(x_name)
            } else if x_labels.is_empty() {
                // 如果没有提供 x 轴标签，使用序号
                AxisConfig::category(
                    (0..series[0].data.len())
                        .map(|i| (i + 1).to_string())
                        .collect(),
                )
            } else {
                AxisConfig::category(x_labels)
            },
            y_axis: AxisConfig::value(y_name),
            y_axis_secondary: None,  // v1.47.0: 暂不支持通过命令行指定副轴（需要混合图表功能）
            series,
            options: ChartOptions {
                show_legend: true,
                show_toolbox: true,
                smooth,
            },
            labels, // v1.45.0: 饼图标签
            indicators: None,  // v1.49.0: 雷达图指标
            heatmap_data: None,  // v1.49.0: 热力图数据
        };

        // 验证数据
        chart_data
            .validate()
            .map_err(|e| anyhow!("{}", e))?;

        Ok(chart_data)
    }

    /// 提取参数值
    ///
    /// 支持格式：
    /// - `--arg value` (单词值)
    /// - `--arg "quoted value"` (引号值)
    fn extract_arg(args: &str, name: &str) -> Option<String> {
        let pattern = format!("{} ", name);
        if let Some(start) = args.find(&pattern) {
            let value_start = start + pattern.len();
            let remaining = &args[value_start..];

            // 检查是否是引号值
            if remaining.starts_with('"') {
                // 查找结束引号
                if let Some(end) = remaining[1..].find('"') {
                    return Some(remaining[1..=end].to_string());
                }
            } else {
                // 非引号值，取到下一个空格或结束
                let value = remaining
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
        None
    }

    /// 提取所有 series
    ///
    /// 格式：`--series "名称:值1,值2,值3"`
    fn extract_series(args: &str) -> Result<Vec<Series>> {
        let mut series = Vec::new();
        let mut search_from = 0;

        loop {
            // 查找下一个 --series
            if let Some(pos) = args[search_from..].find("--series ") {
                let abs_pos = search_from + pos;
                let value_start = abs_pos + "--series ".len();

                // 查找引号包裹的值
                if args[value_start..].starts_with('"') {
                    if let Some(end) = args[value_start + 1..].find('"') {
                        let series_str = &args[value_start + 1..value_start + 1 + end];
                        series.push(Self::parse_series(series_str)?);
                        search_from = value_start + 1 + end + 1;
                    } else {
                        return Err(anyhow!("--series 参数缺少结束引号"));
                    }
                } else {
                    // 没有引号，取到下一个空格
                    let value = args[value_start..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("");
                    series.push(Self::parse_series(value)?);
                    search_from = value_start + value.len();
                }
            } else {
                break;
            }
        }

        Ok(series)
    }

    /// 解析单个 series
    ///
    /// 格式：`"名称:值1,值2,值3"` 或 `"值1,值2,值3"` (默认名称为"数据")
    fn parse_series(series_str: &str) -> Result<Series> {
        if let Some(colon_pos) = series_str.find(':') {
            let name = series_str[..colon_pos].trim().to_string();
            let values_str = &series_str[colon_pos + 1..];
            let data = Self::parse_values(values_str)?;
            Ok(Series {
                name,
                data,
                color: None,
                points: None, // v1.45.0: 散点图坐标（将在后续实现）
                sizes: None,  // v1.48.0: 气泡图大小
                y_axis_index: None,  // v1.47.0
                chart_type: None,    // v1.47.0
            })
        } else {
            // 没有名称，使用默认
            let data = Self::parse_values(series_str)?;
            Ok(Series {
                name: "数据".to_string(),
                data,
                color: None,
                points: None, // v1.45.0: 散点图坐标（将在后续实现）
                sizes: None,  // v1.48.0: 气泡图大小
                y_axis_index: None,  // v1.47.0
                chart_type: None,    // v1.47.0
            })
        }
    }

    /// 解析数值列表
    ///
    /// 格式：`"1.5,2.3,3.7"`
    fn parse_values(values_str: &str) -> Result<Vec<f64>> {
        values_str
            .split(',')
            .map(|v| {
                v.trim()
                    .parse::<f64>()
                    .map_err(|_| anyhow!("无效的数值: {}", v))
            })
            .collect()
    }

    /// v1.45.0: 提取散点图 series
    ///
    /// 支持 `--data` 参数解析
    /// 格式：`--data "x1,y1 x2,y2 x3,y3"`
    fn extract_scatter_series(args: &str) -> Result<Vec<Series>> {
        use crate::visualization::types::Series;

        let mut series_list = Vec::new();

        // 查找所有 --data 参数
        let mut search_start = 0;
        while let Some(start) = args[search_start..].find("--data ") {
            let actual_start = search_start + start;
            let value_start = actual_start + "--data ".len();
            let remaining = &args[value_start..];

            // 提取数据值（可能是引号或非引号）
            let data_str = if remaining.starts_with('"') {
                // 引号值
                if let Some(end) = remaining[1..].find('"') {
                    &remaining[1..=end]
                } else {
                    return Err(anyhow!("--data 参数引号未闭合"));
                }
            } else {
                // 非引号值，取到下一个 -- 或结束
                if let Some(next_arg) = remaining.find("--") {
                    remaining[..next_arg].trim()
                } else {
                    remaining.trim()
                }
            };

            // 解析坐标点：格式 "x1,y1 x2,y2 x3,y3"
            let points = Self::parse_scatter_points(data_str)?;

            // 尝试提取系列名称（可选的 --name 参数）
            let name_pattern = "--name ";
            let name = if let Some(name_start) = args[actual_start..].find(name_pattern) {
                let name_value_start = actual_start + name_start + name_pattern.len();
                let name_remaining = &args[name_value_start..];

                if name_remaining.starts_with('"') {
                    if let Some(end) = name_remaining[1..].find('"') {
                        Some(name_remaining[1..=end].to_string())
                    } else {
                        None
                    }
                } else {
                    name_remaining
                        .split_whitespace()
                        .next()
                        .map(|s| s.to_string())
                }
            } else {
                None
            };

            series_list.push(Series::new_scatter(
                name.unwrap_or_else(|| format!("系列{}", series_list.len() + 1)),
                points,
            ));

            search_start = value_start + data_str.len();
            if search_start >= args.len() {
                break;
            }
        }

        if series_list.is_empty() {
            return Err(anyhow!("散点图需要 --data 参数，格式: --data \"x1,y1 x2,y2 x3,y3\""));
        }

        Ok(series_list)
    }

    /// v1.45.0: 解析散点图坐标点
    ///
    /// 格式：`"x1,y1 x2,y2 x3,y3"` 或 `"x1,y1\nx2,y2\nx3,y3"`
    fn parse_scatter_points(data_str: &str) -> Result<Vec<(f64, f64)>> {
        data_str
            .split_whitespace()
            .map(|point_str| {
                let coords: Vec<&str> = point_str.split(',').collect();
                if coords.len() != 2 {
                    return Err(anyhow!("散点图坐标格式错误: {}，应为 'x,y'", point_str));
                }

                let x = coords[0]
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| anyhow!("无效的 X 坐标: {}", coords[0]))?;
                let y = coords[1]
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| anyhow!("无效的 Y 坐标: {}", coords[1]))?;

                Ok((x, y))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_line_chart() {
        let cmd = r#"chart line --title "测试图表" --x-axis "A,B,C" --series "数据:1,2,3""#;
        let chart = ChartCommandParser::parse(cmd).unwrap();

        assert_eq!(chart.title, "测试图表");
        assert_eq!(chart.chart_type, ChartType::Line);
        assert_eq!(chart.x_axis.data.as_ref().unwrap().len(), 3);
        assert_eq!(chart.series.len(), 1);
        assert_eq!(chart.series[0].name, "数据");
        assert_eq!(chart.series[0].data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_multiple_series() {
        let cmd = r#"chart line --title "对比" --x-axis "Q1,Q2,Q3,Q4" --series "2023:100,120,90,150" --series "2024:120,140,110,180""#;
        let chart = ChartCommandParser::parse(cmd).unwrap();

        assert_eq!(chart.series.len(), 2);
        assert_eq!(chart.series[0].name, "2023");
        assert_eq!(chart.series[1].name, "2024");
    }

    #[test]
    fn test_smooth_option() {
        let cmd = r#"chart line --title "平滑" --series "1,2,3" --smooth"#;
        let chart = ChartCommandParser::parse(cmd).unwrap();

        assert!(chart.options.smooth);
    }

    #[test]
    fn test_auto_x_axis() {
        let cmd = r#"chart line --title "自动X轴" --series "10,20,30""#;
        let chart = ChartCommandParser::parse(cmd).unwrap();

        // 应该自动生成 X 轴标签 ["1", "2", "3"]
        assert_eq!(
            chart.x_axis.data.as_ref().unwrap(),
            &vec!["1".to_string(), "2".to_string(), "3".to_string()]
        );
    }

    #[test]
    fn test_validation_fails() {
        // X 轴和数据长度不匹配
        let cmd = r#"chart line --x-axis "A,B" --series "1,2,3""#;
        let result = ChartCommandParser::parse(cmd);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_chart_type() {
        let cmd = r#"chart invalid --series "1,2,3""#;
        let result = ChartCommandParser::parse(cmd);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("无效的图表类型"));
    }

    #[test]
    fn test_missing_series() {
        let cmd = r#"chart line --title "无数据""#;
        let result = ChartCommandParser::parse(cmd);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("至少需要一个数据系列"));
    }

    // v1.45.0: 饼图测试
    #[test]
    fn test_pie_chart_with_labels() {
        let cmd = r#"chart pie --title "市场份额" --labels "产品A,产品B,产品C" --series "份额:35,25,40""#;
        let chart = ChartCommandParser::parse(cmd).unwrap();

        assert_eq!(chart.chart_type, ChartType::Pie);
        assert_eq!(chart.title, "市场份额");
        assert_eq!(
            chart.labels.as_ref().unwrap(),
            &vec!["产品A".to_string(), "产品B".to_string(), "产品C".to_string()]
        );
        assert_eq!(chart.series[0].data, vec![35.0, 25.0, 40.0]);
    }

    #[test]
    fn test_pie_chart_without_labels() {
        let cmd = r#"chart pie --title "销售占比" --series "销售额:120,230,180""#;
        let chart = ChartCommandParser::parse(cmd).unwrap();

        assert_eq!(chart.chart_type, ChartType::Pie);
        assert!(chart.labels.is_none());
        assert_eq!(chart.series[0].data.len(), 3);
    }

    #[test]
    fn test_pie_chart_validation_fails() {
        // Labels 和数据长度不匹配
        let cmd = r#"chart pie --labels "A,B" --series "1,2,3""#;
        let result = ChartCommandParser::parse(cmd);

        assert!(result.is_err());
    }

    // v1.45.0: 散点图测试
    #[test]
    fn test_scatter_chart_simple() {
        let cmd = r#"chart scatter --title "身高体重分布" --data "170,65 175,70 160,55 180,80""#;
        let chart = ChartCommandParser::parse(cmd).unwrap();

        assert_eq!(chart.chart_type, ChartType::Scatter);
        assert_eq!(chart.title, "身高体重分布");
        assert_eq!(chart.series.len(), 1);
        assert_eq!(chart.series[0].points.as_ref().unwrap().len(), 4);
        assert_eq!(chart.series[0].points.as_ref().unwrap()[0], (170.0, 65.0));
    }

    #[test]
    fn test_scatter_chart_with_axis_names() {
        let cmd = r#"chart scatter --title "相关性分析" --x-name "变量X" --y-name "变量Y" --data "1,2 3,4 5,6""#;
        let chart = ChartCommandParser::parse(cmd).unwrap();

        assert_eq!(chart.x_axis.name, Some("变量X".to_string()));
        assert_eq!(chart.y_axis.name, Some("变量Y".to_string()));
        assert_eq!(chart.series[0].points.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_scatter_chart_multiple_series() {
        let cmd = r#"chart scatter --title "多组对比" --data "1,2 3,4" --data "5,6 7,8""#;
        let chart = ChartCommandParser::parse(cmd).unwrap();

        assert_eq!(chart.series.len(), 2);
        assert_eq!(chart.series[0].points.as_ref().unwrap().len(), 2);
        assert_eq!(chart.series[1].points.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_scatter_chart_validation_fails() {
        // 缺少 --data 参数
        let cmd = r#"chart scatter --title "测试""#;
        let result = ChartCommandParser::parse(cmd);

        assert!(result.is_err());
    }

    #[test]
    fn test_scatter_chart_invalid_format() {
        // 坐标格式错误（缺少 y 值）
        let cmd = r#"chart scatter --data "1,2 3""#;
        let result = ChartCommandParser::parse(cmd);

        assert!(result.is_err());
    }
}
