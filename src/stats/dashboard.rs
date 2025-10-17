//! 系统仪表板
//!
//! 提供美观的统计信息显示
//!
//! 设计哲学：
//! - 极简主义：清晰、简洁、无冗余
//! - 易变哲学：自适应、灵活、拥抱变化

use super::collector::StatsCollector;
use colored::Colorize;
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

/// 仪表板宽度常量
const DASHBOARD_WIDTH: usize = 64;

/// 仪表板
pub struct Dashboard {
    collector: Arc<StatsCollector>,
}

impl Dashboard {
    /// 创建新的仪表板
    pub fn new(collector: Arc<StatsCollector>) -> Self {
        Self { collector }
    }

    /// 渲染完整仪表板
    pub async fn render(&self) -> String {
        let mut output = String::new();

        // 顶部边框
        output.push_str(&self.render_header());
        output.push('\n');

        // 会话统计
        output.push_str(&self.render_section_header("会话统计"));
        output.push_str(&self.render_session_stats().await);

        // LLM 统计
        output.push_str(&self.render_separator());
        output.push_str(&self.render_section_header("LLM 统计"));
        output.push_str(&self.render_llm_stats().await);

        // 工具使用统计
        output.push_str(&self.render_separator());
        output.push_str(&self.render_section_header("工具使用 Top 5"));
        output.push_str(&self.render_tool_stats().await);

        // 性能指标
        output.push_str(&self.render_separator());
        output.push_str(&self.render_section_header("性能指标"));
        output.push_str(&self.render_performance_stats().await);

        // 底部边框
        output.push_str(&self.render_footer());

        output
    }

    /// 渲染简洁版仪表板
    pub async fn render_compact(&self) -> String {
        let command_metrics = self.collector.get_command_metrics().await;
        let llm_metrics = self.collector.get_llm_metrics().await;
        let tool_metrics = self.collector.get_tool_metrics().await;

        let session_duration = command_metrics.session_duration();
        let hours = session_duration.as_secs() / 3600;
        let minutes = (session_duration.as_secs() % 3600) / 60;

        format!(
            "Stats | {}h {}m | {} LLM | {} Tools | {:.1}% Success",
            hours,
            minutes,
            llm_metrics.total_calls,
            tool_metrics.total_calls,
            command_metrics.success_rate() * 100.0
        )
    }

    /// 渲染头部
    fn render_header(&self) -> String {
        let title = "RealConsole System Dashboard v0.9.0";
        self.render_box_line(&title.bold().to_string(), '╔', '╗', '═')
    }

    /// 渲染章节标题
    fn render_section_header(&self, title: &str) -> String {
        let padded = self.pad_line(title, DASHBOARD_WIDTH);
        format!("║{}║\n", padded)
    }

    /// 渲染分隔线
    fn render_separator(&self) -> String {
        format!("╠{}╣\n", "═".repeat(DASHBOARD_WIDTH))
    }

    /// 渲染底部
    fn render_footer(&self) -> String {
        let hint = "Type /dashboard to refresh";
        format!(
            "╚{}╝\n\n{}\n",
            "═".repeat(DASHBOARD_WIDTH),
            self.pad_line(hint, DASHBOARD_WIDTH)
        )
    }

    /// 渲染会话统计
    async fn render_session_stats(&self) -> String {
        let metrics = self.collector.get_command_metrics().await;

        let duration = metrics.session_duration();
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;

        let success_rate = metrics.success_rate() * 100.0;
        let success_color = if success_rate >= 90.0 {
            "green"
        } else if success_rate >= 70.0 {
            "yellow"
        } else {
            "red"
        };

        let mut output = String::new();
        output.push_str(&self.render_data_line(
            "Runtime",
            &format!("{}h {}m", hours, minutes),
            None,
        ));
        output.push_str(&self.render_data_line(
            "Commands",
            &format!("{}", metrics.total_commands),
            None,
        ));
        output.push_str(&self.render_data_line(
            "Success Rate",
            &format!("{:.1}%", success_rate),
            Some(success_color),
        ));

        output
    }

    /// 渲染 LLM 统计
    async fn render_llm_stats(&self) -> String {
        let metrics = self.collector.get_llm_metrics().await;

        let avg_time = metrics.avg_response_time_ms() as f64 / 1000.0;
        let time_color = if avg_time < 1.0 {
            "green"
        } else if avg_time < 2.0 {
            "yellow"
        } else {
            "red"
        };

        let mut output = String::new();
        output.push_str(&self.render_data_line(
            "Total Calls",
            &format!("{}", metrics.total_calls),
            None,
        ));
        output.push_str(&self.render_data_line(
            "Avg Response",
            &format!("{:.2}s", avg_time),
            Some(time_color),
        ));
        output.push_str(&self.render_data_line(
            "Token Usage",
            &format!("{}", metrics.estimated_tokens),
            Some("cyan"),
        ));
        output.push_str(&self.render_data_line(
            "Est. Cost",
            &format!("${:.3}", metrics.estimated_cost),
            Some("cyan"),
        ));

        output
    }

    /// 渲染工具统计
    async fn render_tool_stats(&self) -> String {
        let metrics = self.collector.get_tool_metrics().await;
        let top_tools = metrics.top_tools(5);

        let mut output = String::new();

        if top_tools.is_empty() {
            output.push_str(&self.render_data_line("Status", "No data yet", Some("dimmed")));
        } else {
            let max_usage = top_tools.first().map(|(_, count)| *count).unwrap_or(1);

            for (i, (tool_name, count)) in top_tools.iter().enumerate() {
                let percentage = (*count as f32 / max_usage as f32) * 100.0;
                let bar = self.render_progress_bar(percentage, 15);

                let line = format!(
                    "{}. {} ({}) {}",
                    i + 1,
                    tool_name,
                    count,
                    bar
                );

                let padded = self.pad_line(&line, DASHBOARD_WIDTH);
                output.push_str(&format!("║{}║\n", padded));
            }
        }

        output
    }

    /// 渲染性能统计
    async fn render_performance_stats(&self) -> String {
        let perf_metrics = self.collector.get_performance_metrics().await;

        let p50 = perf_metrics.p50() as f64 / 1000.0;
        let p95 = perf_metrics.p95() as f64 / 1000.0;
        let p99 = perf_metrics.p99() as f64 / 1000.0;

        let slowest_cmd = perf_metrics
            .slowest_command
            .as_deref()
            .unwrap_or("N/A");
        let max_time = perf_metrics.max_response_ms as f64 / 1000.0;

        let mut output = String::new();
        output.push_str(&self.render_data_line(
            "P50 Response",
            &format!("{:.2}s", p50),
            Some("green"),
        ));
        output.push_str(&self.render_data_line(
            "P95 Response",
            &format!("{:.2}s", p95),
            Some("yellow"),
        ));
        output.push_str(&self.render_data_line(
            "P99 Response",
            &format!("{:.2}s", p99),
            Some("red"),
        ));
        output.push_str(&self.render_data_line(
            "Slowest Cmd",
            &format!("\"{}\" ({:.2}s)", self.truncate_str(slowest_cmd, 30), max_time),
            None,
        ));

        output
    }

    /// 渲染进度条
    fn render_progress_bar(&self, percentage: f32, width: usize) -> String {
        let filled = ((percentage / 100.0) * width as f32) as usize;
        let empty = width.saturating_sub(filled);

        format!(
            "{}{}",
            "█".repeat(filled).bright_green(),
            "░".repeat(empty).dimmed()
        )
    }

    /// 为值添加颜色
    fn colorize_value(&self, value: &str, color: &str) -> String {
        match color {
            "green" => value.green().to_string(),
            "yellow" => value.yellow().to_string(),
            "red" => value.red().to_string(),
            "cyan" => value.cyan().to_string(),
            "dimmed" => value.dimmed().to_string(),
            _ => value.to_string(),
        }
    }

    /// 截断字符串
    fn truncate_str(&self, s: &str, max_len: usize) -> String {
        let display_width = self.display_width(s);
        if display_width <= max_len {
            s.to_string()
        } else {
            // 逐字符截断直到满足宽度要求
            let mut result = String::new();
            let mut current_width = 0;

            for ch in s.chars() {
                let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
                if current_width + ch_width > max_len - 3 {
                    break;
                }
                result.push(ch);
                current_width += ch_width;
            }

            format!("{}...", result)
        }
    }

    // ========== 核心辅助方法（易变哲学：适应性设计）==========

    /// 计算字符串的显示宽度（去除 ANSI 颜色代码）
    fn display_width(&self, s: &str) -> usize {
        let stripped = self.strip_ansi(s);
        UnicodeWidthStr::width(stripped.as_str())
    }

    /// 去除 ANSI 转义序列
    fn strip_ansi(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // 跳过整个 ANSI 序列：ESC [ ... m
                if chars.peek() == Some(&'[') {
                    chars.next(); // 跳过 '['
                    // 跳过直到 'm' 或其他终止字符
                    while let Some(c) = chars.next() {
                        if c == 'm' || c.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// 填充字符串到指定宽度（右对齐空格）
    fn pad_line(&self, s: &str, target_width: usize) -> String {
        let current_width = self.display_width(s);

        if current_width >= target_width {
            // 如果超出，截断
            self.truncate_str(s, target_width)
        } else {
            // 补充空格到目标宽度
            let padding = target_width - current_width;
            format!("{}{}", s, " ".repeat(padding))
        }
    }

    /// 渲染带边框的行
    fn render_box_line(&self, content: &str, left: char, right: char, fill: char) -> String {
        let content_width = self.display_width(content);
        let inner_width = DASHBOARD_WIDTH;

        if content_width >= inner_width {
            // 内容过长，直接使用边框
            format!("{}{}{}\n", left, fill.to_string().repeat(inner_width), right)
        } else {
            // 居中显示
            let padding_total = inner_width - content_width;
            let padding_left = padding_total / 2;
            let padding_right = padding_total - padding_left;

            format!(
                "{}{}{}{}{}\n",
                left,
                " ".repeat(padding_left),
                content,
                " ".repeat(padding_right),
                right
            )
        }
    }

    /// 渲染数据行（label: value）
    fn render_data_line(&self, label: &str, value: &str, value_color: Option<&str>) -> String {
        // 计算实际显示宽度
        let label_width = self.display_width(label);
        let value_width = self.display_width(value);

        // 可用宽度 = 总宽度
        let available_width = DASHBOARD_WIDTH;

        // 点号宽度 = 可用宽度 - 标签宽度 - 空格 - 数值宽度 - 空格
        let dots_width = available_width.saturating_sub(label_width + 1 + value_width + 1);

        // 先构建无颜色的行来确保宽度正确
        let plain_line = format!(
            "{} {} {}",
            label,
            ".".repeat(dots_width),
            value
        );

        // 验证显示宽度
        let actual_width = self.display_width(&plain_line);

        // 如果宽度不匹配，调整
        let final_dots_width = if actual_width > available_width {
            dots_width.saturating_sub(actual_width - available_width)
        } else if actual_width < available_width {
            dots_width + (available_width - actual_width)
        } else {
            dots_width
        };

        // 应用颜色
        let colored_dots = ".".repeat(final_dots_width).dimmed().to_string();
        let colored_value = if let Some(color) = value_color {
            self.colorize_value(value, color)
        } else {
            value.to_string()
        };

        // 构建最终行（带颜色）
        let line = format!(
            "{} {} {}",
            label,
            colored_dots,
            colored_value
        );

        // 计算需要的填充（考虑 ANSI 代码）
        let display_width = self.display_width(&line);
        let padding = if display_width < available_width {
            " ".repeat(available_width - display_width)
        } else {
            String::new()
        };

        format!("║{}{}║\n", line, padding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::StatEvent;
    use std::time::Duration;

    #[tokio::test]
    async fn test_dashboard_render() {
        let collector = Arc::new(StatsCollector::new());

        // 添加一些测试数据
        collector
            .record(StatEvent::LlmCall {
                success: true,
                duration: Duration::from_millis(800),
                tokens: 100,
            })
            .await;

        collector
            .record(StatEvent::ToolCall {
                tool_name: "calculator".to_string(),
                success: true,
                duration: Duration::from_millis(50),
            })
            .await;

        collector
            .record(StatEvent::CommandExecution {
                command: "test command".to_string(),
                success: true,
                duration: Duration::from_millis(1000),
            })
            .await;

        let dashboard = Dashboard::new(collector);
        let output = dashboard.render().await;

        // 验证输出包含关键信息
        assert!(output.contains("RealConsole System Dashboard"));
        assert!(output.contains("会话统计"));
        assert!(output.contains("LLM 统计"));
        assert!(output.contains("工具使用"));
        assert!(output.contains("性能指标"));
    }

    #[tokio::test]
    async fn test_compact_dashboard() {
        let collector = Arc::new(StatsCollector::new());

        collector
            .record(StatEvent::CommandExecution {
                command: "test".to_string(),
                success: true,
                duration: Duration::from_millis(1000),
            })
            .await;

        let dashboard = Dashboard::new(collector);
        let output = dashboard.render_compact().await;

        assert!(output.contains("Stats"));
        assert!(output.contains("LLM"));
        assert!(output.contains("Tools"));
    }

    #[test]
    fn test_strip_ansi() {
        let dashboard = Dashboard::new(Arc::new(StatsCollector::new()));

        let colored_text = "\x1b[1;32mGreen Text\x1b[0m";
        let stripped = dashboard.strip_ansi(colored_text);
        assert_eq!(stripped, "Green Text");

        let plain_text = "Plain Text";
        let stripped_plain = dashboard.strip_ansi(plain_text);
        assert_eq!(stripped_plain, "Plain Text");
    }

    #[test]
    fn test_display_width() {
        let dashboard = Dashboard::new(Arc::new(StatsCollector::new()));

        // ASCII 字符
        assert_eq!(dashboard.display_width("Hello"), 5);

        // 中文字符（每个占 2 个宽度）
        assert_eq!(dashboard.display_width("你好"), 4);

        // 带颜色的文本
        let colored = "Hello".green().to_string();
        assert_eq!(dashboard.display_width(&colored), 5);
    }

    #[test]
    fn test_pad_line() {
        let dashboard = Dashboard::new(Arc::new(StatsCollector::new()));

        let padded = dashboard.pad_line("Hello", 10);
        assert_eq!(dashboard.display_width(&padded), 10);
        assert!(padded.starts_with("Hello"));
        assert!(padded.ends_with("     ")); // 5 spaces
    }
}
