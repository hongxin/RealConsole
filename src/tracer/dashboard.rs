//! Dashboard - 四象分区统一视图
//!
//! 基于易经哲学的四象布局：
//! - 太阳（☰☰）- Statistics 维度：命令频率、使用模式
//! - 太阴（☷☷）- Memory 维度：对话上下文、知识积累
//! - 少阳（☲☵）- Coordination 维度：执行追踪、协同流程
//! - 少阴（☵☲）- BlackBox 维度：LLM 调用、智能黑盒
//!
//! 离卦（☲）- 向外照明：展示系统状态，提供可视化
//! 坎卦（☵）- 向内深入：分析系统规律，检测异常

use super::{TraceStats, UnifiedTracer};
use crate::utils::string::truncate_safe;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

/// Dashboard 配置
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// 是否启用四象分区布局
    pub four_quadrants: bool,

    /// 是否显示健康度评分
    pub show_health_score: bool,

    /// 是否显示异常检测
    pub show_anomalies: bool,

    /// 是否显示智能建议
    pub show_suggestions: bool,

    /// 时间窗口（小时）
    pub time_window_hours: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            four_quadrants: true,
            show_health_score: true,
            show_anomalies: true,
            show_suggestions: true,
            time_window_hours: 24,
        }
    }
}

/// 系统健康度评分
#[derive(Debug, Clone)]
pub struct HealthScore {
    /// 总体评分 (0-100)
    pub overall: u8,

    /// 命令成功率 (0-100)
    pub success_rate: u8,

    /// LLM 响应质量 (0-100)
    pub llm_quality: u8,

    /// 系统活跃度 (0-100)
    pub activity_level: u8,

    /// 异常程度 (0-100，越低越好)
    pub anomaly_score: u8,
}

impl HealthScore {
    /// 计算综合健康度
    pub fn calculate(stats: &TraceStats, anomalies: &[Anomaly]) -> Self {
        // 1. 命令成功率
        let success_count = stats.by_status.get("Success").copied().unwrap_or(0);
        let success_rate = if stats.total_entries > 0 {
            ((success_count as f64 / stats.total_entries as f64) * 100.0) as u8
        } else {
            100
        };

        // 2. LLM 响应质量（基于平均响应时间和成功率）
        // TODO: 需要从 LlmLogger 获取更详细的数据
        let llm_quality = 75; // 暂时固定值

        // 3. 系统活跃度（基于平均条目/小时）
        let activity_level = if stats.avg_entries_per_hour > 10.0 {
            100
        } else if stats.avg_entries_per_hour > 5.0 {
            80
        } else if stats.avg_entries_per_hour > 1.0 {
            60
        } else {
            40
        };

        // 4. 异常程度（异常越多，分数越高）
        let anomaly_score = if anomalies.is_empty() {
            0
        } else if anomalies.len() < 3 {
            30
        } else if anomalies.len() < 10 {
            60
        } else {
            90
        };

        // 5. 综合评分（加权平均）
        let overall = ((success_rate as f64 * 0.4)
            + (llm_quality as f64 * 0.2)
            + (activity_level as f64 * 0.2)
            + ((100 - anomaly_score) as f64 * 0.2)) as u8;

        Self {
            overall,
            success_rate,
            llm_quality,
            activity_level,
            anomaly_score,
        }
    }

    /// 获取健康等级
    pub fn level(&self) -> &'static str {
        match self.overall {
            90..=100 => "优秀",
            75..=89 => "良好",
            60..=74 => "一般",
            40..=59 => "较差",
            _ => "危险",
        }
    }

    /// 获取健康等级颜色
    pub fn level_color(&self) -> colored::Color {
        match self.overall {
            90..=100 => colored::Color::Green,
            75..=89 => colored::Color::Cyan,
            60..=74 => colored::Color::Yellow,
            40..=59 => colored::Color::Magenta,
            _ => colored::Color::Red,
        }
    }
}

/// 异常类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnomalyType {
    /// 高失败率
    HighFailureRate,

    /// LLM 响应慢
    SlowLlmResponse,

    /// 重复错误
    RepeatedErrors,

    /// 异常模式
    UnusualPattern,

    /// 资源异常
    ResourceAnomaly,
}

impl AnomalyType {
    pub fn icon(&self) -> &'static str {
        match self {
            AnomalyType::HighFailureRate => "⚠️",
            AnomalyType::SlowLlmResponse => "🐌",
            AnomalyType::RepeatedErrors => "🔁",
            AnomalyType::UnusualPattern => "🔍",
            AnomalyType::ResourceAnomaly => "📊",
        }
    }

    pub fn severity(&self) -> &'static str {
        match self {
            AnomalyType::HighFailureRate => "高",
            AnomalyType::SlowLlmResponse => "中",
            AnomalyType::RepeatedErrors => "高",
            AnomalyType::UnusualPattern => "低",
            AnomalyType::ResourceAnomaly => "中",
        }
    }
}

/// 异常检测结果
#[derive(Debug, Clone)]
pub struct Anomaly {
    /// 异常类型
    pub anomaly_type: AnomalyType,

    /// 描述
    pub description: String,

    /// 严重程度 (1-5)
    pub severity: u8,

    /// 相关数据
    pub data: HashMap<String, String>,
}

/// 智能建议
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// 建议类型
    pub category: String,

    /// 建议内容
    pub message: String,

    /// 优先级 (1-5)
    pub priority: u8,

    /// 可执行的命令（可选）
    pub command: Option<String>,
}

/// Dashboard 数据
pub struct DashboardData {
    /// 统计信息
    pub stats: TraceStats,

    /// 健康度评分
    pub health: HealthScore,

    /// 异常检测结果
    pub anomalies: Vec<Anomaly>,

    /// 智能建议
    pub suggestions: Vec<Suggestion>,
}

/// Dashboard 生成器
pub struct Dashboard {
    tracer: Arc<UnifiedTracer>,
    config: DashboardConfig,
}

impl Dashboard {
    /// 创建 Dashboard
    pub fn new(tracer: Arc<UnifiedTracer>, config: DashboardConfig) -> Self {
        Self { tracer, config }
    }

    /// 创建默认 Dashboard
    pub fn with_defaults(tracer: Arc<UnifiedTracer>) -> Self {
        Self::new(tracer, DashboardConfig::default())
    }

    /// 收集 Dashboard 数据
    pub async fn collect_data(&self) -> Result<DashboardData> {
        // 1. 获取统计信息
        let stats = self.tracer.stats().await?;

        // 2. 检测异常
        let anomalies = self.detect_anomalies(&stats).await;

        // 3. 计算健康度
        let health = HealthScore::calculate(&stats, &anomalies);

        // 4. 生成智能建议
        let suggestions = self.generate_suggestions(&stats, &health, &anomalies).await;

        Ok(DashboardData {
            stats,
            health,
            anomalies,
            suggestions,
        })
    }

    /// 检测异常（坎卦 - 向内深入）
    async fn detect_anomalies(&self, stats: &TraceStats) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // 1. 检测高失败率
        let failed_count = stats.by_status.get("Failed").copied().unwrap_or(0);
        let failure_rate = if stats.total_entries > 0 {
            (failed_count as f64 / stats.total_entries as f64) * 100.0
        } else {
            0.0
        };

        if failure_rate > 20.0 {
            let mut data = HashMap::new();
            data.insert("failure_rate".to_string(), format!("{:.1}%", failure_rate));
            data.insert("failed_count".to_string(), failed_count.to_string());

            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::HighFailureRate,
                description: format!("命令失败率过高：{:.1}%", failure_rate),
                severity: if failure_rate > 50.0 { 5 } else { 3 },
                data,
            });
        }

        // 2. 检测重复错误
        if let Ok(failed_logs) = self.tracer.get_failed_logs(50).await {
            // 统计错误消息出现次数
            let mut error_counts: HashMap<String, usize> = HashMap::new();
            for log in &failed_logs {
                let error_key = log.result_preview.clone();
                *error_counts.entry(error_key).or_insert(0) += 1;
            }

            // 找出重复次数 >= 3 的错误
            let repeated_errors: Vec<_> = error_counts
                .iter()
                .filter(|(_, &count)| count >= 3)
                .collect();

            if !repeated_errors.is_empty() {
                // 找出最严重的重复错误
                let max_count = repeated_errors.iter().map(|(_, &count)| count).max().unwrap_or(0);
                let most_repeated = repeated_errors
                    .iter()
                    .find(|(_, &count)| count == max_count)
                    .map(|(msg, _)| msg.as_str())
                    .unwrap_or("未知错误");

                let mut data = HashMap::new();
                data.insert("error_count".to_string(), max_count.to_string());
                data.insert("unique_errors".to_string(), repeated_errors.len().to_string());
                data.insert("top_error".to_string(), most_repeated.to_string());

                // 使用安全截断工具函数（自动处理 UTF-8 字符边界）
                let error_preview = truncate_safe(most_repeated, 40);

                anomalies.push(Anomaly {
                    anomaly_type: AnomalyType::RepeatedErrors,
                    description: format!("检测到重复错误：{} 次 - {}", max_count, error_preview),
                    severity: if max_count >= 5 { 5 } else { 4 },
                    data,
                });
            }
        }

        // 3. 检测异常模式
        // TODO: 基于历史数据分析
        // 暂时跳过

        anomalies
    }

    /// 生成智能建议（离卦 - 向外照明）
    async fn generate_suggestions(
        &self,
        stats: &TraceStats,
        health: &HealthScore,
        anomalies: &[Anomaly],
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // 1. 基于健康度的建议
        if health.overall < 60 {
            suggestions.push(Suggestion {
                category: "系统健康".to_string(),
                message: format!(
                    "系统整体健康度为 {}，建议检查最近的失败命令",
                    health.level()
                ),
                priority: 4,
                command: Some("/trace search \"Failed\"".to_string()),
            });
        }

        // 2. 基于成功率的建议
        if health.success_rate < 80 {
            suggestions.push(Suggestion {
                category: "命令成功率".to_string(),
                message: format!(
                    "命令成功率仅 {}%，建议查看失败日志",
                    health.success_rate
                ),
                priority: 3,
                command: Some("/trace log 20".to_string()),
            });
        }

        // 3. 基于异常的建议
        for anomaly in anomalies {
            match anomaly.anomaly_type {
                AnomalyType::HighFailureRate => {
                    suggestions.push(Suggestion {
                        category: "故障排查".to_string(),
                        message: "检测到高失败率，建议检查环境配置或权限问题".to_string(),
                        priority: 5,
                        command: Some("/trace stats".to_string()),
                    });
                }
                AnomalyType::RepeatedErrors => {
                    let error_count = anomaly.data.get("error_count")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0);

                    suggestions.push(Suggestion {
                        category: "重复错误".to_string(),
                        message: format!(
                            "同一错误重复出现 {} 次，建议查看错误详情并修复",
                            error_count
                        ),
                        priority: 5,
                        command: Some("/trace log 30".to_string()),
                    });
                }
                _ => {}
            }
        }

        // 4. 基于活跃度的建议
        if stats.total_entries > 0 && stats.avg_entries_per_hour < 1.0 {
            suggestions.push(Suggestion {
                category: "系统使用".to_string(),
                message: "系统活跃度较低，可以尝试更多功能探索".to_string(),
                priority: 1,
                command: Some("/help".to_string()),
            });
        }

        // 按优先级排序
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));

        suggestions
    }

    /// 渲染 Dashboard（离卦 - 照明展示）
    pub async fn render(&self) -> Result<String> {
        let data = self.collect_data().await?;

        let mut output = Vec::new();

        // 标题
        output.push(format!("{}", "═══ 系统 Dashboard ═══".bold().cyan()));
        output.push(String::new());

        // 健康度总览
        if self.config.show_health_score {
            output.push(self.render_health_score(&data.health));
            output.push(String::new());
        }

        // 四象分区
        if self.config.four_quadrants {
            output.push(self.render_four_quadrants(&data.stats));
            output.push(String::new());
        } else {
            // 简化统计
            output.push(self.render_simple_stats(&data.stats));
            output.push(String::new());
        }

        // 异常检测
        if self.config.show_anomalies && !data.anomalies.is_empty() {
            output.push(self.render_anomalies(&data.anomalies));
            output.push(String::new());
        }

        // 智能建议
        if self.config.show_suggestions && !data.suggestions.is_empty() {
            output.push(self.render_suggestions(&data.suggestions));
        }

        Ok(output.join("\n"))
    }

    /// 渲染健康度评分
    fn render_health_score(&self, health: &HealthScore) -> String {
        let level_colored = health.level().color(health.level_color()).bold();

        let bar_length = (health.overall as usize / 2).max(1);
        let bar = "█".repeat(bar_length);

        let mut lines = vec![
            format!("{}", "系统健康度".bold()),
            format!(
                "  {} {} {}% {}",
                "总体评分:".dimmed(),
                bar.color(health.level_color()),
                health.overall.to_string().color(health.level_color()).bold(),
                level_colored
            ),
        ];

        // 详细指标
        lines.push(format!("  {} {}%", "命令成功率".dimmed(), health.success_rate.to_string().cyan()));
        lines.push(format!("  {} {}%", "LLM 质量".dimmed(), health.llm_quality.to_string().cyan()));
        lines.push(format!("  {} {}%", "系统活跃度".dimmed(), health.activity_level.to_string().cyan()));

        if health.anomaly_score > 0 {
            lines.push(format!(
                "  {} {}%",
                "异常程度".dimmed(),
                health.anomaly_score.to_string().red()
            ));
        }

        lines.join("\n")
    }

    /// 渲染四象分区
    fn render_four_quadrants(&self, stats: &TraceStats) -> String {
        let mut lines = vec![format!("{}", "四象分区视图".bold())];

        // 计算各维度数据
        let total = stats.total_entries;
        let dimensions = &stats.by_dimension;

        // 太阳象限 - Statistics
        let stat_count = dimensions.get(&super::Dimension::Statistics).copied().unwrap_or(0);
        let stat_percent = if total > 0 { (stat_count as f64 / total as f64 * 100.0) as usize } else { 0 };

        // 太阴象限 - Memory
        let mem_count = dimensions.get(&super::Dimension::Memory).copied().unwrap_or(0);
        let mem_percent = if total > 0 { (mem_count as f64 / total as f64 * 100.0) as usize } else { 0 };

        // 少阳象限 - Coordination
        let coord_count = dimensions.get(&super::Dimension::Coordination).copied().unwrap_or(0);
        let coord_percent = if total > 0 { (coord_count as f64 / total as f64 * 100.0) as usize } else { 0 };

        // 少阴象限 - BlackBox
        let bb_count = dimensions.get(&super::Dimension::BlackBox).copied().unwrap_or(0);
        let bb_percent = if total > 0 { (bb_count as f64 / total as f64 * 100.0) as usize } else { 0 };

        // 准备四个象限的内容（使用正确的四象符号：两横 digram）
        let top_left_title = format!("{} 太阳 Statistics", "⚌".yellow());      // U+268C DIGRAM FOR GREATER YANG (老阳)
        let top_right_title = format!("{} 少阳 Coordination", "⚍".cyan());     // U+268D DIGRAM FOR LESSER YANG
        let bottom_left_title = format!("{} 太阴 Memory", "⚎".blue());         // U+268E DIGRAM FOR GREATER YIN (老阴)
        let bottom_right_title = format!("{} 少阴 BlackBox", "⚏".magenta());   // U+268F DIGRAM FOR LESSER YIN

        let top_left_desc = "命令频率、使用模式";
        let top_right_desc = "执行追踪、协同流程";
        let bottom_left_desc = "对话上下文、知识积累";
        let bottom_right_desc = "LLM 调用、智能黑盒";

        let top_left_data = format!("{} {:3}% ({:4} 条)", "▸".yellow(), stat_percent, stat_count);
        let top_right_data = format!("{} {:3}% ({:4} 条)", "▸".cyan(), coord_percent, coord_count);
        let bottom_left_data = format!("{} {:3}% ({:4} 条)", "▸".blue(), mem_percent, mem_count);
        let bottom_right_data = format!("{} {:3}% ({:4} 条)", "▸".magenta(), bb_percent, bb_count);

        lines.push(String::new());
        lines.push(format!("┌─────────────────────────┬─────────────────────────┐"));
        lines.push(format_quadrant_row(&top_left_title, &top_right_title));
        lines.push(format_quadrant_row(top_left_desc, top_right_desc));
        lines.push(format_quadrant_row(&top_left_data, &top_right_data));
        lines.push(format!("├─────────────────────────┼─────────────────────────┤"));
        lines.push(format_quadrant_row(&bottom_left_title, &bottom_right_title));
        lines.push(format_quadrant_row(bottom_left_desc, bottom_right_desc));
        lines.push(format_quadrant_row(&bottom_left_data, &bottom_right_data));
        lines.push(format!("└─────────────────────────┴─────────────────────────┘"));

        lines.join("\n")
    }

    /// 渲染简化统计
    fn render_simple_stats(&self, stats: &TraceStats) -> String {
        let mut lines = vec![format!("{}", "统计概览".bold())];

        lines.push(format!("  {} {}", "总条目".dimmed(), stats.total_entries.to_string().green()));

        if let Some((earliest, latest)) = stats.time_range {
            let duration = latest.signed_duration_since(earliest);
            let hours = duration.num_hours();
            lines.push(format!("  {} {} 小时", "时间跨度".dimmed(), hours.to_string().cyan()));
            lines.push(format!(
                "  {} {:.1} 条/小时",
                "平均速率".dimmed(),
                stats.avg_entries_per_hour.to_string().cyan()
            ));
        }

        lines.join("\n")
    }

    /// 渲染异常检测结果
    fn render_anomalies(&self, anomalies: &[Anomaly]) -> String {
        let mut lines = vec![format!("{} 异常检测", "⚠️".yellow().bold())];

        for (i, anomaly) in anomalies.iter().enumerate().take(5) {
            let severity_color = match anomaly.severity {
                5 => colored::Color::Red,
                4 => colored::Color::Magenta,
                3 => colored::Color::Yellow,
                _ => colored::Color::Cyan,
            };

            lines.push(format!(
                "  {}. {} {} [{}]",
                i + 1,
                anomaly.anomaly_type.icon(),
                anomaly.description.color(severity_color),
                format!("严重度: {}", anomaly.anomaly_type.severity()).dimmed()
            ));
        }

        if anomalies.len() > 5 {
            lines.push(format!("  {} 还有 {} 个异常未显示", "...".dimmed(), anomalies.len() - 5));
        }

        lines.join("\n")
    }

    /// 渲染智能建议
    fn render_suggestions(&self, suggestions: &[Suggestion]) -> String {
        let mut lines = vec![format!("{} 智能建议", "💡".cyan().bold())];

        for (i, suggestion) in suggestions.iter().enumerate().take(3) {
            let priority_icon = match suggestion.priority {
                5 => "🔴",
                4 => "🟠",
                3 => "🟡",
                2 => "🟢",
                _ => "⚪",
            };

            lines.push(format!(
                "  {}. {} [{}] {}",
                i + 1,
                priority_icon,
                suggestion.category.cyan(),
                suggestion.message
            ));

            if let Some(ref cmd) = suggestion.command {
                lines.push(format!("     {} {}", "▸".dimmed(), cmd.dimmed()));
            }
        }

        lines.join("\n")
    }
}

/// 格式化四象分区的一行，确保左右单元格对齐
///
/// 使用 unicode-width 计算实际显示宽度，处理中英文混合和 emoji
fn format_quadrant_row(left: &str, right: &str) -> String {
    // 每个单元格的目标宽度（不包括边框 "│ " 和 " │"）
    const CELL_WIDTH: usize = 25;

    // 移除 ANSI 颜色代码后计算实际显示宽度
    fn visual_width(s: &str) -> usize {
        // 简单的 ANSI 代码移除（colored crate 生成的）
        let clean = strip_ansi_codes(s);
        clean.width()
    }

    // 简单的 ANSI 代码移除
    fn strip_ansi_codes(s: &str) -> String {
        let mut result = String::new();
        let mut in_escape = false;

        for ch in s.chars() {
            if ch == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if ch == 'm' {
                    in_escape = false;
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    let left_width = visual_width(left);
    let right_width = visual_width(right);

    let left_padding = if left_width < CELL_WIDTH {
        " ".repeat(CELL_WIDTH - left_width)
    } else {
        String::new()
    };

    let right_padding = if right_width < CELL_WIDTH {
        " ".repeat(CELL_WIDTH - right_width)
    } else {
        String::new()
    };

    format!("│ {}{} │ {}{} │", left, left_padding, right, right_padding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_score_calculation() {
        let mut stats = TraceStats {
            total_entries: 100,
            by_dimension: HashMap::new(),
            by_status: HashMap::new(),
            time_range: None,
            avg_entries_per_hour: 10.0,
        };

        stats.by_status.insert("Success".to_string(), 90);
        stats.by_status.insert("Failed".to_string(), 10);

        let anomalies = vec![];
        let health = HealthScore::calculate(&stats, &anomalies);

        assert_eq!(health.success_rate, 90);
        assert!(health.overall >= 70);
        assert_eq!(health.level(), "良好");
    }

    #[test]
    fn test_anomaly_type_properties() {
        let anomaly = AnomalyType::HighFailureRate;
        assert_eq!(anomaly.icon(), "⚠️");
        assert_eq!(anomaly.severity(), "高");
    }
}
