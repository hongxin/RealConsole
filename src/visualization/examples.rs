//! 图表示例库
//!
//! v1.51.0: 提供完整的实战示例，帮助用户快速学习和应用
//!
//! ## 设计理念
//!
//! - **实战导向**: 每个示例都是真实场景的完整实现
//! - **渐进学习**: 按难度分级（Beginner/Intermediate/Advanced）
//! - **代码示例**: 包含完整的命令行示例
//! - **分类清晰**: 与模板系统保持一致的分类

use super::templates::TemplateCategory;
use super::types::ChartData;
use serde::{Deserialize, Serialize};

/// 示例难度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExampleDifficulty {
    /// 初级 - 基础用法
    Beginner,
    /// 中级 - 常见场景
    Intermediate,
    /// 高级 - 复杂应用
    Advanced,
}

impl ExampleDifficulty {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "beginner" | "初级" => Some(Self::Beginner),
            "intermediate" | "中级" => Some(Self::Intermediate),
            "advanced" | "高级" => Some(Self::Advanced),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Beginner => "初级",
            Self::Intermediate => "中级",
            Self::Advanced => "高级",
        }
    }
}

/// 图表示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartExample {
    /// 示例 ID
    pub id: String,
    /// 示例标题
    pub title: String,
    /// 详细描述
    pub description: String,
    /// 分类
    pub category: TemplateCategory,
    /// 难度
    pub difficulty: ExampleDifficulty,
    /// 完整的图表数据
    pub chart_data: ChartData,
    /// 命令行代码示例
    pub code_snippet: String,
    /// 标签
    pub tags: Vec<String>,
    /// 学习要点
    pub learning_points: Vec<String>,
}

impl ChartExample {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        category: TemplateCategory,
        difficulty: ExampleDifficulty,
        chart_data: ChartData,
        code_snippet: impl Into<String>,
        tags: Vec<String>,
        learning_points: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            category,
            difficulty,
            chart_data,
            code_snippet: code_snippet.into(),
            tags,
            learning_points,
        }
    }
}

/// 示例库管理器
pub struct ExampleLibrary {
    examples: Vec<ChartExample>,
}

impl ExampleLibrary {
    /// 创建示例库并加载内置示例
    pub fn new() -> Self {
        Self {
            examples: Self::load_builtin_examples(),
        }
    }

    /// 获取所有示例
    pub fn all_examples(&self) -> &[ChartExample] {
        &self.examples
    }

    /// 按 ID 查找示例
    pub fn find_by_id(&self, id: &str) -> Option<&ChartExample> {
        self.examples.iter().find(|e| e.id == id)
    }

    /// 按分类筛选示例
    pub fn filter_by_category(&self, category: TemplateCategory) -> Vec<&ChartExample> {
        self.examples
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    /// 按难度筛选示例
    pub fn filter_by_difficulty(&self, difficulty: ExampleDifficulty) -> Vec<&ChartExample> {
        self.examples
            .iter()
            .filter(|e| e.difficulty == difficulty)
            .collect()
    }

    /// 按关键词搜索示例
    pub fn search(&self, keyword: &str) -> Vec<&ChartExample> {
        let keyword_lower = keyword.to_lowercase();
        self.examples
            .iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&keyword_lower)
                    || e.description.to_lowercase().contains(&keyword_lower)
                    || e.tags.iter().any(|tag| tag.to_lowercase().contains(&keyword_lower))
            })
            .collect()
    }

    /// 获取分类统计
    pub fn category_summary(&self) -> Vec<(TemplateCategory, usize)> {
        use std::collections::HashMap;

        let mut counts: HashMap<TemplateCategory, usize> = HashMap::new();
        for example in &self.examples {
            *counts.entry(example.category).or_insert(0) += 1;
        }

        let mut summary: Vec<(TemplateCategory, usize)> = counts.into_iter().collect();
        summary.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        summary
    }

    /// 获取难度统计
    pub fn difficulty_summary(&self) -> Vec<(ExampleDifficulty, usize)> {
        use std::collections::HashMap;

        let mut counts: HashMap<ExampleDifficulty, usize> = HashMap::new();
        for example in &self.examples {
            *counts.entry(example.difficulty).or_insert(0) += 1;
        }

        vec![
            (ExampleDifficulty::Beginner, *counts.get(&ExampleDifficulty::Beginner).unwrap_or(&0)),
            (ExampleDifficulty::Intermediate, *counts.get(&ExampleDifficulty::Intermediate).unwrap_or(&0)),
            (ExampleDifficulty::Advanced, *counts.get(&ExampleDifficulty::Advanced).unwrap_or(&0)),
        ]
    }

    /// 加载内置示例
    fn load_builtin_examples() -> Vec<ChartExample> {
        vec![
            // ===== 初级示例 (5个) =====
            Self::create_simple_sales_example(),
            Self::create_team_comparison_example(),
            Self::create_monthly_trend_example(),
            Self::create_resource_pie_example(),
            Self::create_quick_scatter_example(),

            // ===== 中级示例 (7个) =====
            Self::create_multi_series_comparison_example(),
            Self::create_quarterly_growth_example(),
            Self::create_user_behavior_example(),
            Self::create_performance_monitoring_example(),
            Self::create_skill_assessment_example(),
            Self::create_correlation_study_example(),
            Self::create_workload_analysis_example(),

            // ===== 高级示例 (3个) =====
            Self::create_complex_funnel_example(),
            Self::create_multi_dimension_radar_example(),
            Self::create_time_series_forecast_example(),
        ]
    }

    // ===== 初级示例 =====

    fn create_simple_sales_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "simple-sales",
            "简单销售数据",
            "最基础的折线图示例，展示6个月的销售数据变化趋势",
            TemplateCategory::Business,
            ExampleDifficulty::Beginner,
            ChartData {
                chart_type: ChartType::Line,
                title: "半年销售趋势".to_string(),
                x_axis: AxisConfig::category(vec![
                    "1月".to_string(), "2月".to_string(), "3月".to_string(),
                    "4月".to_string(), "5月".to_string(), "6月".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("销售额（万）".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("销售额", vec![120.0, 132.0, 101.0, 134.0, 150.0, 170.0])],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: true },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart line --title "半年销售趋势" --x-axis "1月,2月,3月,4月,5月,6月" --series "销售额:120,132,101,134,150,170" --smooth"#,
            vec!["初级".to_string(), "折线图".to_string(), "销售".to_string()],
            vec![
                "折线图的基本用法".to_string(),
                "--smooth 参数使曲线平滑".to_string(),
                "x轴标签用逗号分隔".to_string(),
            ],
        )
    }

    fn create_team_comparison_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "team-comparison",
            "团队绩效对比",
            "使用柱状图对比不同团队的绩效得分",
            TemplateCategory::Team,
            ExampleDifficulty::Beginner,
            ChartData {
                chart_type: ChartType::Bar,
                title: "Q2 团队绩效对比".to_string(),
                x_axis: AxisConfig::category(vec![
                    "开发团队".to_string(), "测试团队".to_string(),
                    "设计团队".to_string(), "运营团队".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("绩效得分".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("得分", vec![85.0, 78.0, 92.0, 88.0])],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: false },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart bar --title "Q2 团队绩效对比" --x-axis "开发团队,测试团队,设计团队,运营团队" --series "得分:85,78,92,88""#,
            vec!["初级".to_string(), "柱状图".to_string(), "团队".to_string()],
            vec![
                "柱状图适合对比数据".to_string(),
                "分类轴显示团队名称".to_string(),
                "数值轴显示具体得分".to_string(),
            ],
        )
    }

    fn create_monthly_trend_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "monthly-trend",
            "月度用户增长",
            "展示移动应用的月度活跃用户增长趋势",
            TemplateCategory::Technical,
            ExampleDifficulty::Beginner,
            ChartData {
                chart_type: ChartType::Line,
                title: "月度活跃用户 (MAU)".to_string(),
                x_axis: AxisConfig::category(vec![
                    "1月".to_string(), "2月".to_string(), "3月".to_string(),
                    "4月".to_string(), "5月".to_string(), "6月".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("用户数（万）".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("MAU", vec![150.0, 165.0, 180.0, 195.0, 210.0, 230.0])],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: true },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart line --title "月度活跃用户 (MAU)" --x-axis "1月,2月,3月,4月,5月,6月" --series "MAU:150,165,180,195,210,230" --smooth"#,
            vec!["初级".to_string(), "折线图".to_string(), "用户增长".to_string()],
            vec![
                "趋势图展示增长情况".to_string(),
                "平滑曲线更易观察趋势".to_string(),
                "Y轴单位需明确标注".to_string(),
            ],
        )
    }

    fn create_resource_pie_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "resource-pie",
            "资源分配情况",
            "用饼图展示服务器资源的分配比例",
            TemplateCategory::Technical,
            ExampleDifficulty::Beginner,
            ChartData {
                chart_type: ChartType::Pie,
                title: "服务器资源分配".to_string(),
                x_axis: AxisConfig::category(vec![]),
                y_axis: AxisConfig::value(None),
                y_axis_secondary: None,
                series: vec![Series::new("占用率", vec![35.0, 25.0, 20.0, 20.0])],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: false },
                labels: Some(vec![
                    "Web服务".to_string(), "数据库".to_string(),
                    "缓存".to_string(), "其他".to_string(),
                ]),
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart pie --title "服务器资源分配" --labels "Web服务,数据库,缓存,其他" --series "占用率:35,25,20,20""#,
            vec!["初级".to_string(), "饼图".to_string(), "资源".to_string()],
            vec![
                "饼图展示占比关系".to_string(),
                "--labels 定义各部分名称".to_string(),
                "数据总和最好是100%".to_string(),
            ],
        )
    }

    fn create_quick_scatter_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "quick-scatter",
            "快速散点图",
            "展示两个变量之间的关系分布",
            TemplateCategory::Exploration,
            ExampleDifficulty::Beginner,
            ChartData {
                chart_type: ChartType::Scatter,
                title: "身高体重关系".to_string(),
                x_axis: AxisConfig::value(Some("身高(cm)".to_string())),
                y_axis: AxisConfig::value(Some("体重(kg)".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new_scatter(
                    "样本数据",
                    vec![
                        (160.0, 55.0), (165.0, 60.0), (170.0, 65.0),
                        (175.0, 70.0), (180.0, 75.0), (175.0, 68.0),
                    ],
                )],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: false },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart scatter --title "身高体重关系" --x-name "身高(cm)" --y-name "体重(kg)" --data "160,55 165,60 170,65 175,70 180,75 175,68""#,
            vec!["初级".to_string(), "散点图".to_string(), "相关性".to_string()],
            vec![
                "散点图展示两变量关系".to_string(),
                "--data 格式为 'x,y x,y'".to_string(),
                "可观察数据分布和相关性".to_string(),
            ],
        )
    }

    // ===== 中级示例 (简化版，这里只实现关键的几个) =====

    fn create_multi_series_comparison_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "multi-series",
            "多系列对比分析",
            "对比2023和2024年的季度销售数据",
            TemplateCategory::Business,
            ExampleDifficulty::Intermediate,
            ChartData {
                chart_type: ChartType::Line,
                title: "年度销售对比".to_string(),
                x_axis: AxisConfig::category(vec![
                    "Q1".to_string(), "Q2".to_string(), "Q3".to_string(), "Q4".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("销售额（万）".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("2023年", vec![450.0, 520.0, 480.0, 580.0]),
                    Series::new("2024年", vec![520.0, 610.0, 590.0, 680.0]),
                ],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: true },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart line --title "年度销售对比" --x-axis "Q1,Q2,Q3,Q4" --series "2023年:450,520,480,580" --series "2024年:520,610,590,680" --smooth"#,
            vec!["中级".to_string(), "多系列".to_string(), "对比".to_string()],
            vec![
                "多个 --series 参数添加多条线".to_string(),
                "图例自动显示系列名称".to_string(),
                "便于对比不同时期数据".to_string(),
            ],
        )
    }

    fn create_quarterly_growth_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "quarterly-growth",
            "季度增长分析",
            "柱状图展示各季度同比增长率",
            TemplateCategory::Business,
            ExampleDifficulty::Intermediate,
            ChartData {
                chart_type: ChartType::Bar,
                title: "季度同比增长率".to_string(),
                x_axis: AxisConfig::category(vec![
                    "Q1".to_string(), "Q2".to_string(), "Q3".to_string(), "Q4".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("增长率(%)".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("增长率", vec![12.5, 15.8, 18.2, 16.7])],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: false },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart bar --title "季度同比增长率" --x-axis "Q1,Q2,Q3,Q4" --series "增长率:12.5,15.8,18.2,16.7""#,
            vec!["中级".to_string(), "增长率".to_string(), "柱状图".to_string()],
            vec![
                "柱状图清晰展示增长率".to_string(),
                "百分比单位需在Y轴标注".to_string(),
                "数据可包含小数".to_string(),
            ],
        )
    }

    fn create_user_behavior_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "user-behavior",
            "用户行为分析",
            "展示用户在网站各页面的停留时间",
            TemplateCategory::Technical,
            ExampleDifficulty::Intermediate,
            ChartData {
                chart_type: ChartType::Bar,
                title: "页面平均停留时间".to_string(),
                x_axis: AxisConfig::category(vec![
                    "首页".to_string(), "产品页".to_string(), "详情页".to_string(),
                    "购物车".to_string(), "结算页".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("时长（秒）".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("停留时长", vec![45.0, 120.0, 180.0, 90.0, 60.0])],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: false },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart bar --title "页面平均停留时间" --x-axis "首页,产品页,详情页,购物车,结算页" --series "停留时长:45,120,180,90,60""#,
            vec!["中级".to_string(), "用户行为".to_string(), "分析".to_string()],
            vec![
                "了解用户关注点".to_string(),
                "优化高停留页面".to_string(),
                "识别流失环节".to_string(),
            ],
        )
    }

    fn create_performance_monitoring_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "performance-monitoring",
            "系统性能监控",
            "监控API接口的响应时间变化",
            TemplateCategory::Technical,
            ExampleDifficulty::Intermediate,
            ChartData {
                chart_type: ChartType::Line,
                title: "API 响应时间监控".to_string(),
                x_axis: AxisConfig::category(vec![
                    "00:00".to_string(), "04:00".to_string(), "08:00".to_string(),
                    "12:00".to_string(), "16:00".to_string(), "20:00".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("响应时间(ms)".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("响应时间", vec![85.0, 75.0, 120.0, 150.0, 180.0, 110.0])],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: true },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart line --title "API 响应时间监控" --x-axis "00:00,04:00,08:00,12:00,16:00,20:00" --series "响应时间:85,75,120,150,180,110" --smooth"#,
            vec!["中级".to_string(), "性能".to_string(), "监控".to_string()],
            vec![
                "实时监控系统性能".to_string(),
                "识别性能瓶颈时段".to_string(),
                "平滑曲线便于观察趋势".to_string(),
            ],
        )
    }

    fn create_skill_assessment_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "skill-assessment",
            "员工技能评估",
            "使用雷达图全方位展示员工技能水平",
            TemplateCategory::Team,
            ExampleDifficulty::Intermediate,
            ChartData {
                chart_type: ChartType::Radar,
                title: "员工技能评估".to_string(),
                x_axis: AxisConfig::category(vec![]),
                y_axis: AxisConfig::value(None),
                y_axis_secondary: None,
                series: vec![Series::new("张三", vec![85.0, 90.0, 75.0, 80.0, 70.0])],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: false },
                labels: None,
                indicators: Some(vec![
                    "技术能力".to_string(), "沟通能力".to_string(), "项目管理".to_string(),
                    "团队协作".to_string(), "创新思维".to_string(),
                ]),
                heatmap_data: None,
            },
            "使用模板: !chart use skill-radar",
            vec!["中级".to_string(), "雷达图".to_string(), "技能".to_string()],
            vec![
                "雷达图适合多维度评估".to_string(),
                "直观展示优势和短板".to_string(),
                "可对比多个员工".to_string(),
            ],
        )
    }

    fn create_correlation_study_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "correlation-study",
            "相关性研究",
            "散点图分析广告投入与销售额的相关关系",
            TemplateCategory::Academic,
            ExampleDifficulty::Intermediate,
            ChartData {
                chart_type: ChartType::Scatter,
                title: "广告投入-销售额相关性".to_string(),
                x_axis: AxisConfig::value(Some("广告投入（万元）".to_string())),
                y_axis: AxisConfig::value(Some("销售额（万元）".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new_scatter(
                    "月度数据",
                    vec![
                        (10.0, 120.0), (15.0, 150.0), (20.0, 180.0),
                        (25.0, 220.0), (30.0, 250.0), (22.0, 190.0),
                        (28.0, 240.0), (18.0, 170.0),
                    ],
                )],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: false },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart scatter --title "广告投入-销售额相关性" --x-name "广告投入（万元）" --y-name "销售额（万元）" --data "10,120 15,150 20,180 25,220 30,250 22,190 28,240 18,170""#,
            vec!["中级".to_string(), "相关性".to_string(), "分析".to_string()],
            vec![
                "散点图识别相关关系".to_string(),
                "观察数据分布趋势".to_string(),
                "辅助决策投入策略".to_string(),
            ],
        )
    }

    fn create_workload_analysis_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "workload-analysis",
            "工作负载分布",
            "饼图展示团队成员的任务分配情况",
            TemplateCategory::Team,
            ExampleDifficulty::Intermediate,
            ChartData {
                chart_type: ChartType::Pie,
                title: "本周任务分配".to_string(),
                x_axis: AxisConfig::category(vec![]),
                y_axis: AxisConfig::value(None),
                y_axis_secondary: None,
                series: vec![Series::new("任务数", vec![8.0, 6.0, 10.0, 7.0, 9.0])],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: false },
                labels: Some(vec![
                    "张三".to_string(), "李四".to_string(), "王五".to_string(),
                    "赵六".to_string(), "孙七".to_string(),
                ]),
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart pie --title "本周任务分配" --labels "张三,李四,王五,赵六,孙七" --series "任务数:8,6,10,7,9""#,
            vec!["中级".to_string(), "工作负载".to_string(), "分布".to_string()],
            vec![
                "识别负载不均".to_string(),
                "优化任务分配".to_string(),
                "提高团队效率".to_string(),
            ],
        )
    }

    // ===== 高级示例 =====

    fn create_complex_funnel_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "complex-funnel",
            "转化漏斗分析",
            "使用柱状图分析用户从访问到购买的完整转化路径",
            TemplateCategory::Business,
            ExampleDifficulty::Advanced,
            ChartData {
                chart_type: ChartType::Bar,
                title: "用户转化漏斗".to_string(),
                x_axis: AxisConfig::category(vec![
                    "访问首页".to_string(), "浏览商品".to_string(), "加入购物车".to_string(),
                    "进入结算".to_string(), "完成支付".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("用户数".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("用户数", vec![10000.0, 6500.0, 3200.0, 1500.0, 1200.0])],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: false },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart bar --title "用户转化漏斗" --x-axis "访问首页,浏览商品,加入购物车,进入结算,完成支付" --series "用户数:10000,6500,3200,1500,1200""#,
            vec!["高级".to_string(), "转化漏斗".to_string(), "用户行为".to_string()],
            vec![
                "识别转化瓶颈环节".to_string(),
                "计算各阶段转化率".to_string(),
                "优化转化流程".to_string(),
                "提升整体转化效率".to_string(),
            ],
        )
    }

    fn create_multi_dimension_radar_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "multi-dimension-radar",
            "多维度对比分析",
            "使用雷达图对比两个产品的多维度指标",
            TemplateCategory::Business,
            ExampleDifficulty::Advanced,
            ChartData {
                chart_type: ChartType::Radar,
                title: "产品竞争力对比".to_string(),
                x_axis: AxisConfig::category(vec![]),
                y_axis: AxisConfig::value(None),
                y_axis_secondary: None,
                series: vec![
                    Series::new("产品A", vec![90.0, 85.0, 75.0, 80.0, 95.0, 70.0]),
                    Series::new("产品B", vec![75.0, 90.0, 85.0, 70.0, 80.0, 88.0]),
                ],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: false },
                labels: None,
                indicators: Some(vec![
                    "性能".to_string(), "易用性".to_string(), "稳定性".to_string(),
                    "价格".to_string(), "品牌".to_string(), "服务".to_string(),
                ]),
                heatmap_data: None,
            },
            "使用模板并修改数据: !chart use multi-factor-comparison",
            vec!["高级".to_string(), "雷达图".to_string(), "对比".to_string()],
            vec![
                "多维度全方位对比".to_string(),
                "直观展示优劣势".to_string(),
                "辅助产品决策".to_string(),
                "支持多个对象对比".to_string(),
            ],
        )
    }

    fn create_time_series_forecast_example() -> ChartExample {
        use super::types::{AxisConfig, ChartOptions, ChartType, Series};

        ChartExample::new(
            "time-series-forecast",
            "时间序列预测",
            "展示历史数据和未来预测趋势",
            TemplateCategory::Business,
            ExampleDifficulty::Advanced,
            ChartData {
                chart_type: ChartType::Line,
                title: "收入预测分析".to_string(),
                x_axis: AxisConfig::category(vec![
                    "Q1".to_string(), "Q2".to_string(), "Q3".to_string(), "Q4".to_string(),
                    "Q1(预测)".to_string(), "Q2(预测)".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("收入（万）".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("实际收入", vec![500.0, 550.0, 580.0, 620.0, 0.0, 0.0]),
                    Series::new("预测收入", vec![0.0, 0.0, 0.0, 620.0, 660.0, 700.0]),
                ],
                options: ChartOptions { show_legend: true, show_toolbox: true, smooth: true },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
            r#"!chart line --title "收入预测分析" --x-axis "Q1,Q2,Q3,Q4,Q1(预测),Q2(预测)" --series "实际收入:500,550,580,620,0,0" --series "预测收入:0,0,0,620,660,700" --smooth"#,
            vec!["高级".to_string(), "预测".to_string(), "时间序列".to_string()],
            vec![
                "区分历史和预测数据".to_string(),
                "使用0值连接两条线".to_string(),
                "辅助战略规划".to_string(),
                "评估预测准确性".to_string(),
            ],
        )
    }
}

impl Default for ExampleLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_library_initialization() {
        let library = ExampleLibrary::new();
        assert_eq!(library.all_examples().len(), 15);
    }

    #[test]
    fn test_find_example_by_id() {
        let library = ExampleLibrary::new();
        let example = library.find_by_id("simple-sales");
        assert!(example.is_some());
        assert_eq!(example.unwrap().title, "简单销售数据");
    }

    #[test]
    fn test_filter_by_category() {
        let library = ExampleLibrary::new();
        let business_examples = library.filter_by_category(TemplateCategory::Business);
        assert!(!business_examples.is_empty());
    }

    #[test]
    fn test_filter_by_difficulty() {
        let library = ExampleLibrary::new();
        let beginner_examples = library.filter_by_difficulty(ExampleDifficulty::Beginner);
        assert_eq!(beginner_examples.len(), 5);
    }

    #[test]
    fn test_search_examples() {
        let library = ExampleLibrary::new();
        let results = library.search("销售");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_all_examples_have_valid_data() {
        let library = ExampleLibrary::new();
        for example in library.all_examples() {
            assert!(!example.id.is_empty());
            assert!(!example.title.is_empty());
            assert!(!example.code_snippet.is_empty());
            if let Err(e) = example.chart_data.validate() {
                panic!("Example '{}' validation failed: {}", example.id, e);
            }
        }
    }
}
