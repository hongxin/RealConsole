// CSV 文件解析模块
// v1.45.0: 支持从 CSV 文件读取数据并生成图表

use anyhow::{anyhow, Result};
use std::path::Path;

use super::types::{AxisConfig, ChartData, ChartOptions, ChartType, Series};

/// CSV 数据结构
#[derive(Debug, Clone)]
pub struct CsvData {
    /// 列标题
    pub headers: Vec<String>,
    /// 数据行
    pub records: Vec<Vec<String>>,
}

impl CsvData {
    /// 获取列数
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }

    /// 获取行数（不包括 header）
    pub fn row_count(&self) -> usize {
        self.records.len()
    }

    /// 获取指定列的数据
    pub fn get_column(&self, col_index: usize) -> Result<Vec<String>> {
        if col_index >= self.column_count() {
            return Err(anyhow!("列索引超出范围: {}", col_index));
        }

        Ok(self
            .records
            .iter()
            .map(|row| row.get(col_index).cloned().unwrap_or_default())
            .collect())
    }

    /// 根据列名获取数据
    pub fn get_column_by_name(&self, col_name: &str) -> Result<Vec<String>> {
        let col_index = self
            .headers
            .iter()
            .position(|h| h == col_name)
            .ok_or_else(|| anyhow!("找不到列: {}", col_name))?;

        self.get_column(col_index)
    }

    /// 将列数据转换为 f64 数组
    pub fn column_as_numbers(&self, col_index: usize) -> Result<Vec<f64>> {
        let col_data = self.get_column(col_index)?;
        col_data
            .iter()
            .map(|s| {
                s.trim()
                    .parse::<f64>()
                    .map_err(|_| anyhow!("无法将 '{}' 转换为数字", s))
            })
            .collect()
    }

    /// 转换为 ChartData
    ///
    /// # 参数
    /// - `chart_type`: 图表类型
    /// - `title`: 图表标题
    /// - `x_col`: X 轴列索引或列名
    /// - `y_cols`: Y 轴列索引或列名（可以有多个系列）
    pub fn to_chart_data(
        &self,
        chart_type: ChartType,
        title: impl Into<String>,
        x_col: &str,
        y_cols: &[&str],
    ) -> Result<ChartData> {
        if y_cols.is_empty() {
            return Err(anyhow!("至少需要一个 Y 轴列"));
        }

        // 获取 X 轴数据
        let x_data = self.get_column_by_name(x_col)?;

        // 获取所有 Y 轴数据系列
        let mut series = Vec::new();
        for y_col in y_cols {
            let y_data = self.column_as_numbers(
                self.headers
                    .iter()
                    .position(|h| h == y_col)
                    .ok_or_else(|| anyhow!("找不到列: {}", y_col))?,
            )?;

            series.push(Series::new(y_col.to_string(), y_data));
        }

        let chart_data = ChartData {
            chart_type,
            title: title.into(),
            x_axis: AxisConfig::category(x_data),
            y_axis: AxisConfig::value(None),
            y_axis_secondary: None,  // v1.47.0
            series,
            options: ChartOptions::default(),
            labels: None,
            indicators: None,  // v1.49.0
            heatmap_data: None,  // v1.49.0
        };

        // 验证数据
        chart_data
            .validate()
            .map_err(|e| anyhow!("数据验证失败: {}", e))?;

        Ok(chart_data)
    }
}

/// 从 CSV 文件读取数据
///
/// # 参数
/// - `path`: CSV 文件路径
///
/// # 返回
/// - `Result<CsvData>`: CSV 数据结构
///
/// # 错误
/// - 文件不存在
/// - CSV 格式错误
pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<CsvData> {
    let path = path.as_ref();

    // 检查文件是否存在
    if !path.exists() {
        return Err(anyhow!("文件不存在: {}", path.display()));
    }

    // 使用 csv crate 读取文件
    let mut reader = csv::Reader::from_path(path)
        .map_err(|e| anyhow!("无法读取 CSV 文件: {}", e))?;

    // 读取 headers
    let headers = reader
        .headers()
        .map_err(|e| anyhow!("无法读取 CSV header: {}", e))?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    // 读取所有记录
    let mut records = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| anyhow!("读取 CSV 记录失败: {}", e))?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();

        // 验证列数匹配
        if row.len() != headers.len() {
            return Err(anyhow!(
                "CSV 数据列数不一致：期望 {} 列，实际 {} 列",
                headers.len(),
                row.len()
            ));
        }

        records.push(row);
    }

    Ok(CsvData { headers, records })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_csv_file() {
        // 创建临时 CSV 文件
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "月份,销售额,成本").unwrap();
        writeln!(file, "1月,120,80").unwrap();
        writeln!(file, "2月,132,85").unwrap();
        writeln!(file, "3月,101,70").unwrap();

        let csv_data = parse_csv_file(file.path()).unwrap();

        assert_eq!(csv_data.headers, vec!["月份", "销售额", "成本"]);
        assert_eq!(csv_data.row_count(), 3);
        assert_eq!(csv_data.column_count(), 3);
    }

    #[test]
    fn test_get_column() {
        let csv_data = CsvData {
            headers: vec!["A".to_string(), "B".to_string()],
            records: vec![
                vec!["1".to_string(), "2".to_string()],
                vec!["3".to_string(), "4".to_string()],
            ],
        };

        let col_a = csv_data.get_column(0).unwrap();
        assert_eq!(col_a, vec!["1", "3"]);

        let col_b = csv_data.get_column_by_name("B").unwrap();
        assert_eq!(col_b, vec!["2", "4"]);
    }

    #[test]
    fn test_column_as_numbers() {
        let csv_data = CsvData {
            headers: vec!["值".to_string()],
            records: vec![
                vec!["1.5".to_string()],
                vec!["2.3".to_string()],
                vec!["3.7".to_string()],
            ],
        };

        let numbers = csv_data.column_as_numbers(0).unwrap();
        assert_eq!(numbers, vec![1.5, 2.3, 3.7]);
    }

    #[test]
    fn test_to_chart_data() {
        let csv_data = CsvData {
            headers: vec!["月份".to_string(), "销售额".to_string()],
            records: vec![
                vec!["1月".to_string(), "120".to_string()],
                vec!["2月".to_string(), "132".to_string()],
            ],
        };

        let chart = csv_data
            .to_chart_data(ChartType::Line, "月度销售", "月份", &["销售额"])
            .unwrap();

        assert_eq!(chart.title, "月度销售");
        assert_eq!(chart.chart_type, ChartType::Line);
        assert_eq!(chart.series.len(), 1);
        assert_eq!(chart.series[0].data, vec![120.0, 132.0]);
    }
}
