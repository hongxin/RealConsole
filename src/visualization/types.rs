// 可视化数据类型定义
// v1.44.0: 基础图表数据结构
// v1.52.0: 添加图像数据结构（远程运维场景）

use serde::{Deserialize, Serialize};

/// 图表数据（顶层结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    /// 图表类型
    pub chart_type: ChartType,
    /// 图表标题
    pub title: String,
    /// X轴配置
    pub x_axis: AxisConfig,
    /// Y轴配置（主轴）
    pub y_axis: AxisConfig,
    /// v1.47.0: 副 Y 轴配置（可选，用于双 Y 轴图表）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_secondary: Option<AxisConfig>,
    /// 数据系列
    pub series: Vec<Series>,
    /// 图表选项
    pub options: ChartOptions,
    /// v1.45.0: 饼图标签（可选，用于饼图扇区名称）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    /// v1.49.0: 雷达图指标维度（可选，用于雷达图）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indicators: Option<Vec<String>>,
    /// v1.49.0: 热力图数据（可选，格式: [[x_index, y_index, value], ...]）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heatmap_data: Option<Vec<(usize, usize, f64)>>,
}

impl ChartData {
    /// 创建简单折线图
    pub fn simple_line(
        title: impl Into<String>,
        x_labels: Vec<String>,
        y_data: Vec<f64>,
    ) -> Self {
        Self {
            chart_type: ChartType::Line,
            title: title.into(),
            x_axis: AxisConfig::category(x_labels),
            y_axis: AxisConfig::value(None),
            y_axis_secondary: None,  // v1.47.0
            series: vec![Series {
                name: "数据".to_string(),
                data: y_data,
                color: None,
                points: None,
                sizes: None,         // v1.48.0
                y_axis_index: None,  // v1.47.0
                chart_type: None,    // v1.47.0
            }],
            options: ChartOptions::default(),
            labels: None,
            indicators: None,  // v1.49.0
            heatmap_data: None,  // v1.49.0
        }
    }

    /// 验证数据有效性
    pub fn validate(&self) -> Result<(), String> {
        // 检查系列不为空
        if self.series.is_empty() {
            return Err("至少需要一个数据系列".to_string());
        }

        // v1.45.0: 饼图特殊验证
        if self.chart_type == ChartType::Pie {
            // 如果有 labels，检查长度匹配
            if let Some(labels) = &self.labels {
                for series in &self.series {
                    if series.data.len() != labels.len() {
                        return Err(format!(
                            "系列 '{}' 数据长度({})与标签长度({})不匹配",
                            series.name,
                            series.data.len(),
                            labels.len()
                        ));
                    }
                }
            }
            return Ok(());
        }

        // v1.45.0: 散点图特殊验证
        if self.chart_type == ChartType::Scatter {
            for series in &self.series {
                if let Some(points) = &series.points {
                    if points.is_empty() {
                        return Err(format!("散点图系列 '{}' 的坐标点不能为空", series.name));
                    }
                } else {
                    return Err(format!(
                        "散点图系列 '{}' 必须包含 points 数据",
                        series.name
                    ));
                }
            }
            return Ok(());
        }

        // v1.48.0: 气泡图特殊验证
        if self.chart_type == ChartType::Bubble {
            for series in &self.series {
                // 检查 points 存在
                if let Some(points) = &series.points {
                    if points.is_empty() {
                        return Err(format!("气泡图系列 '{}' 的坐标点不能为空", series.name));
                    }
                    // 检查 sizes 存在
                    if let Some(sizes) = &series.sizes {
                        if sizes.is_empty() {
                            return Err(format!("气泡图系列 '{}' 的气泡大小不能为空", series.name));
                        }
                        // 检查 points 和 sizes 长度一致
                        if points.len() != sizes.len() {
                            return Err(format!(
                                "气泡图系列 '{}' 的坐标点数量({})与气泡大小数量({})不匹配",
                                series.name,
                                points.len(),
                                sizes.len()
                            ));
                        }
                    } else {
                        return Err(format!(
                            "气泡图系列 '{}' 必须包含 sizes 数据",
                            series.name
                        ));
                    }
                } else {
                    return Err(format!(
                        "气泡图系列 '{}' 必须包含 points 数据",
                        series.name
                    ));
                }
            }
            return Ok(());
        }

        // v1.49.0: 雷达图特殊验证
        if self.chart_type == ChartType::Radar {
            // 检查 indicators 存在且不为空
            if let Some(indicators) = &self.indicators {
                if indicators.is_empty() {
                    return Err("雷达图必须包含至少一个指标维度".to_string());
                }
                // 检查系列数据长度与指标数量匹配
                for series in &self.series {
                    if series.data.len() != indicators.len() {
                        return Err(format!(
                            "雷达图系列 '{}' 数据长度({})与指标数量({})不匹配",
                            series.name,
                            series.data.len(),
                            indicators.len()
                        ));
                    }
                }
            } else {
                return Err("雷达图必须包含 indicators 数据".to_string());
            }
            return Ok(());
        }

        // v1.49.0: 热力图特殊验证
        if self.chart_type == ChartType::Heatmap {
            // 检查 heatmap_data 存在且不为空
            if let Some(heatmap_data) = &self.heatmap_data {
                if heatmap_data.is_empty() {
                    return Err("热力图必须包含热力数据".to_string());
                }
            } else {
                return Err("热力图必须包含 heatmap_data".to_string());
            }
            return Ok(());
        }

        // 折线图和柱状图：检查系列数据长度与 X 轴匹配
        if let Some(x_data) = &self.x_axis.data {
            for series in &self.series {
                if series.data.len() != x_data.len() {
                    return Err(format!(
                        "系列 '{}' 数据长度({})与X轴({})不匹配",
                        series.name,
                        series.data.len(),
                        x_data.len()
                    ));
                }
            }
        }

        Ok(())
    }
}

/// 图表类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    /// 折线图
    Line,
    /// 柱状图
    Bar,
    /// 饼图
    Pie,
    /// 散点图
    Scatter,
    /// 面积图 (v1.47.0)
    Area,
    /// 气泡图 (v1.48.0 - Phase 3 P2)
    Bubble,
    /// 雷达图 (v1.49.0 - Phase 3 P3)
    Radar,
    /// 热力图 (v1.49.0 - Phase 3 P3)
    Heatmap,
}

impl ChartType {
    /// 从字符串解析
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "line" => Some(Self::Line),
            "bar" => Some(Self::Bar),
            "pie" => Some(Self::Pie),
            "scatter" => Some(Self::Scatter),
            "area" => Some(Self::Area),  // v1.47.0
            "bubble" => Some(Self::Bubble),  // v1.48.0
            "radar" => Some(Self::Radar),  // v1.49.0
            "heatmap" => Some(Self::Heatmap),  // v1.49.0
            _ => None,
        }
    }
}

/// 坐标轴配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    /// 轴名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 轴数据（类目轴）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<String>>,
    /// 轴类型：value（数值轴）/ category（类目轴）/ time（时间轴）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axis_type: Option<String>,
}

impl AxisConfig {
    /// 创建类目轴
    pub fn category(data: Vec<String>) -> Self {
        Self {
            name: None,
            data: Some(data),
            axis_type: Some("category".to_string()),
        }
    }

    /// 创建数值轴
    pub fn value(name: Option<String>) -> Self {
        Self {
            name,
            data: None,
            axis_type: Some("value".to_string()),
        }
    }

    /// 创建时间轴
    pub fn time(name: Option<String>) -> Self {
        Self {
            name,
            data: None,
            axis_type: Some("time".to_string()),
        }
    }
}

/// 数据系列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    /// 系列名称
    pub name: String,
    /// 数据点（用于折线图、柱状图、饼图）
    pub data: Vec<f64>,
    /// 颜色（可选，十六进制格式 #RRGGBB）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// v1.45.0: 散点图数据点（可选，用于散点图的二维坐标）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<(f64, f64)>>,
    /// v1.48.0: 气泡大小（可选，用于气泡图，与 points 配合使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sizes: Option<Vec<f64>>,
    /// v1.47.0: Y 轴索引（可选，0=主轴, 1=副轴，默认0）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_index: Option<usize>,
    /// v1.47.0: 系列图表类型（可选，用于混合图表。如果指定，覆盖 ChartData 的 chart_type）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_type: Option<ChartType>,
}

impl Series {
    /// 创建新系列（用于折线图、柱状图、饼图）
    pub fn new(name: impl Into<String>, data: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            data,
            color: None,
            points: None,
            sizes: None,         // v1.48.0
            y_axis_index: None,  // v1.47.0
            chart_type: None,    // v1.47.0
        }
    }

    /// v1.45.0: 创建散点图系列
    pub fn new_scatter(name: impl Into<String>, points: Vec<(f64, f64)>) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(), // 散点图不使用 data 字段
            color: None,
            points: Some(points),
            sizes: None,         // v1.48.0
            y_axis_index: None,  // v1.47.0
            chart_type: None,    // v1.47.0
        }
    }

    /// v1.48.0: 创建气泡图系列
    pub fn new_bubble(name: impl Into<String>, points: Vec<(f64, f64)>, sizes: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(), // 气泡图不使用 data 字段
            color: None,
            points: Some(points),
            sizes: Some(sizes),
            y_axis_index: None,
            chart_type: None,
        }
    }

    /// 设置颜色
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// v1.47.0: 设置 Y 轴索引（0=主轴, 1=副轴）
    pub fn with_y_axis_index(mut self, index: usize) -> Self {
        self.y_axis_index = Some(index);
        self
    }

    /// v1.47.0: 设置系列图表类型（用于混合图表）
    pub fn with_chart_type(mut self, chart_type: ChartType) -> Self {
        self.chart_type = Some(chart_type);
        self
    }
}

/// 图表选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    /// 是否显示图例
    pub show_legend: bool,
    /// 是否显示工具栏
    pub show_toolbox: bool,
    /// 是否平滑曲线（仅折线图）
    pub smooth: bool,
}

impl Default for ChartOptions {
    fn default() -> Self {
        Self {
            show_legend: true,
            show_toolbox: true,
            smooth: false,
        }
    }
}

// ==================== v1.52.0: 图像数据结构 ====================

/// v1.52.0: 图像数据（用于远程运维场景）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// 图像类型：base64 或 url
    pub image_type: String,
    /// MIME 类型：image/png, image/jpeg, image/gif, etc.
    pub mime_type: String,
    /// 图像数据（base64 编码字符串或 URL）
    pub data: String,
    /// 替代文本（用于无障碍访问和加载失败时显示）
    pub alt_text: String,
    /// 文件名（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// 文件大小（字节，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<usize>,
}

impl ImageData {
    /// 创建 base64 图像
    pub fn from_base64(
        mime_type: impl Into<String>,
        data: impl Into<String>,
        alt_text: impl Into<String>,
    ) -> Self {
        Self {
            image_type: "base64".to_string(),
            mime_type: mime_type.into(),
            data: data.into(),
            alt_text: alt_text.into(),
            filename: None,
            size_bytes: None,
        }
    }

    /// 创建 URL 图像
    pub fn from_url(url: impl Into<String>, alt_text: impl Into<String>) -> Self {
        Self {
            image_type: "url".to_string(),
            mime_type: "image/unknown".to_string(),
            data: url.into(),
            alt_text: alt_text.into(),
            filename: None,
            size_bytes: None,
        }
    }

    /// 设置文件名
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// 设置文件大小
    pub fn with_size(mut self, size_bytes: usize) -> Self {
        self.size_bytes = Some(size_bytes);
        self
    }

    /// 验证数据有效性
    pub fn validate(&self) -> Result<(), String> {
        // 检查 image_type
        match self.image_type.as_str() {
            "base64" | "url" => {}
            _ => return Err(format!("无效的图像类型: {}", self.image_type)),
        }

        // 检查 data 不为空
        if self.data.is_empty() {
            return Err("图像数据不能为空".to_string());
        }

        // base64 类型检查大小限制（10MB）
        if self.image_type == "base64" {
            const MAX_BASE64_LEN: usize = 14_000_000; // ~10MB base64 编码后
            if self.data.len() > MAX_BASE64_LEN {
                return Err(format!(
                    "Base64 数据过大: {} 字符（最大约 10MB）",
                    self.data.len()
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_type_from_str() {
        assert_eq!(ChartType::parse("line"), Some(ChartType::Line));
        assert_eq!(ChartType::parse("LINE"), Some(ChartType::Line));
        assert_eq!(ChartType::parse("bar"), Some(ChartType::Bar));
        assert_eq!(ChartType::parse("invalid"), None);
    }

    #[test]
    fn test_simple_line_chart() {
        let chart = ChartData::simple_line(
            "测试",
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![1.0, 2.0, 3.0],
        );

        assert_eq!(chart.chart_type, ChartType::Line);
        assert_eq!(chart.title, "测试");
        assert_eq!(chart.series.len(), 1);
        assert!(chart.validate().is_ok());
    }

    #[test]
    fn test_validation() {
        let mut chart = ChartData::simple_line(
            "测试",
            vec!["A".to_string(), "B".to_string()],
            vec![1.0, 2.0, 3.0], // 长度不匹配
        );

        assert!(chart.validate().is_err());

        // 修正数据
        chart.series[0].data = vec![1.0, 2.0];
        assert!(chart.validate().is_ok());
    }

    #[test]
    fn test_series_builder() {
        let series = Series::new("测试", vec![1.0, 2.0, 3.0]).with_color("#FF0000");

        assert_eq!(series.name, "测试");
        assert_eq!(series.data.len(), 3);
        assert_eq!(series.color, Some("#FF0000".to_string()));
    }

    // v1.45.0: 饼图测试
    #[test]
    fn test_pie_chart_with_labels() {
        let chart = ChartData {
            chart_type: ChartType::Pie,
            title: "市场份额".to_string(),
            x_axis: AxisConfig::value(None),
            y_axis: AxisConfig::value(None),
            y_axis_secondary: None,  // v1.47.0
            series: vec![Series::new("份额", vec![35.0, 25.0, 40.0])],
            options: ChartOptions::default(),
            labels: Some(vec!["产品A".to_string(), "产品B".to_string(), "产品C".to_string()]),
            indicators: None,  // v1.49.0
            heatmap_data: None,  // v1.49.0
        };

        assert!(chart.validate().is_ok());
    }

    #[test]
    fn test_pie_chart_validation_fails() {
        let chart = ChartData {
            chart_type: ChartType::Pie,
            title: "测试".to_string(),
            x_axis: AxisConfig::value(None),
            y_axis: AxisConfig::value(None),
            y_axis_secondary: None,  // v1.47.0
            series: vec![Series::new("数据", vec![1.0, 2.0])],
            options: ChartOptions::default(),
            labels: Some(vec!["A".to_string(), "B".to_string(), "C".to_string()]), // 长度不匹配
            indicators: None,  // v1.49.0
            heatmap_data: None,  // v1.49.0
        };

        assert!(chart.validate().is_err());
    }

    // v1.45.0: 散点图测试
    #[test]
    fn test_scatter_chart() {
        let chart = ChartData {
            chart_type: ChartType::Scatter,
            title: "身高体重分布".to_string(),
            x_axis: AxisConfig::value(Some("身高".to_string())),
            y_axis: AxisConfig::value(Some("体重".to_string())),
            y_axis_secondary: None,  // v1.47.0
            series: vec![Series::new_scatter(
                "人群A",
                vec![(170.0, 65.0), (175.0, 70.0), (160.0, 55.0)],
            )],
            options: ChartOptions::default(),
            labels: None,
            indicators: None,  // v1.49.0
            heatmap_data: None,  // v1.49.0
        };

        assert!(chart.validate().is_ok());
    }

    #[test]
    fn test_scatter_chart_validation_fails() {
        let chart = ChartData {
            chart_type: ChartType::Scatter,
            title: "测试".to_string(),
            x_axis: AxisConfig::value(None),
            y_axis: AxisConfig::value(None),
            y_axis_secondary: None,  // v1.47.0
            series: vec![Series::new("数据", vec![1.0, 2.0])], // 没有 points
            options: ChartOptions::default(),
            labels: None,
            indicators: None,  // v1.49.0
            heatmap_data: None,  // v1.49.0
        };

        assert!(chart.validate().is_err());
    }

    #[test]
    fn test_scatter_series_builder() {
        let series =
            Series::new_scatter("测试", vec![(1.0, 2.0), (3.0, 4.0)]).with_color("#00FF00");

        assert_eq!(series.name, "测试");
        assert_eq!(series.points.as_ref().unwrap().len(), 2);
        assert_eq!(series.color, Some("#00FF00".to_string()));
    }
}
