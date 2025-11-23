// RealConsole 图表模板系统
// v1.50.0: 社区建设工具支撑 - 图表模板
//
// 模块职责：
// - 提供预定义的场景化图表模板
// - 支持模板列表、搜索、应用
// - 降低用户创建图表的门槛

use super::types::{AxisConfig, ChartData, ChartOptions, ChartType, Series};
use serde::{Deserialize, Serialize};

/// 图表模板分类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateCategory {
    /// 业务分析
    Business,
    /// 技术监控
    Technical,
    /// 团队管理
    Team,
    /// 学术研究
    Academic,
    /// 数据探索
    Exploration,
}

impl TemplateCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "business" => Some(Self::Business),
            "technical" => Some(Self::Technical),
            "team" => Some(Self::Team),
            "academic" => Some(Self::Academic),
            "exploration" => Some(Self::Exploration),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Business => "业务分析",
            Self::Technical => "技术监控",
            Self::Team => "团队管理",
            Self::Academic => "学术研究",
            Self::Exploration => "数据探索",
        }
    }
}

/// 图表模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartTemplate {
    /// 模板 ID（全局唯一）
    pub id: String,
    /// 模板名称
    pub name: String,
    /// 分类
    pub category: TemplateCategory,
    /// 描述
    pub description: String,
    /// 使用提示
    pub usage_hint: String,
    /// 标签
    pub tags: Vec<String>,
    /// 占位图表数据
    pub placeholder_data: ChartData,
}

impl ChartTemplate {
    /// 创建新模板
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: TemplateCategory,
        description: impl Into<String>,
        usage_hint: impl Into<String>,
        tags: Vec<String>,
        placeholder_data: ChartData,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category,
            description: description.into(),
            usage_hint: usage_hint.into(),
            tags,
            placeholder_data,
        }
    }

    /// 获取图表类型
    pub fn chart_type(&self) -> ChartType {
        self.placeholder_data.chart_type
    }

    /// 应用模板（返回占位数据的克隆）
    pub fn apply(&self) -> ChartData {
        self.placeholder_data.clone()
    }
}

/// 模板引擎
pub struct TemplateEngine {
    templates: Vec<ChartTemplate>,
}

impl TemplateEngine {
    /// 创建模板引擎并加载内置模板
    pub fn new() -> Self {
        Self {
            templates: Self::load_builtin_templates(),
        }
    }

    /// 获取所有模板
    pub fn all_templates(&self) -> &[ChartTemplate] {
        &self.templates
    }

    /// 按 ID 查找模板
    pub fn find_by_id(&self, id: &str) -> Option<&ChartTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    /// 按分类筛选模板
    pub fn filter_by_category(&self, category: TemplateCategory) -> Vec<&ChartTemplate> {
        self.templates
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// 按关键词搜索模板
    pub fn search(&self, keyword: &str) -> Vec<&ChartTemplate> {
        let keyword_lower = keyword.to_lowercase();
        self.templates
            .iter()
            .filter(|t| {
                t.name.to_lowercase().contains(&keyword_lower)
                    || t.description.to_lowercase().contains(&keyword_lower)
                    || t.tags.iter().any(|tag| tag.to_lowercase().contains(&keyword_lower))
            })
            .collect()
    }

    /// 加载内置模板
    fn load_builtin_templates() -> Vec<ChartTemplate> {
        vec![
            // ===== 业务分析 (5 个) =====
            Self::create_sales_trend_template(),
            Self::create_market_share_template(),
            Self::create_growth_analysis_template(),
            Self::create_conversion_funnel_template(),
            Self::create_revenue_forecast_template(),
            // ===== 技术监控 (5 个) =====
            Self::create_performance_metrics_template(),
            Self::create_error_rate_template(),
            Self::create_resource_usage_template(),
            Self::create_traffic_pattern_template(),
            Self::create_api_latency_template(),
            // ===== 团队管理 (5 个) =====
            Self::create_team_performance_template(),
            Self::create_skill_radar_template(),
            Self::create_workload_distribution_template(),
            Self::create_project_progress_template(),
            Self::create_bug_trend_template(),
            // ===== 学术研究 (3 个) =====
            Self::create_experiment_comparison_template(),
            Self::create_correlation_analysis_template(),
            Self::create_multi_factor_comparison_template(),
            // ===== 数据探索 (2 个) =====
            Self::create_quick_preview_template(),
            Self::create_distribution_analysis_template(),
        ]
    }

    // ===== 业务分析模板 =====

    fn create_sales_trend_template() -> ChartTemplate {
        ChartTemplate::new(
            "sales-trend",
            "月度销售趋势",
            TemplateCategory::Business,
            "展示月度销售额的变化趋势，适合销售报告和业绩分析",
            "替换 x 轴标签（月份）和 series 数据（销售额）即可",
            vec!["销售".to_string(), "趋势".to_string(), "折线图".to_string()],
            ChartData {
                chart_type: ChartType::Line,
                title: "月度销售趋势".to_string(),
                x_axis: AxisConfig::category(vec![
                    "1月".to_string(),
                    "2月".to_string(),
                    "3月".to_string(),
                    "4月".to_string(),
                    "5月".to_string(),
                    "6月".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("销售额（万元）".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("销售额", vec![120.0, 132.0, 145.0, 138.0, 155.0, 170.0])],
                options: ChartOptions {
                    show_legend: true,
                    show_toolbox: true,
                    smooth: true,
                },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_market_share_template() -> ChartTemplate {
        ChartTemplate::new(
            "market-share",
            "市场份额分布",
            TemplateCategory::Business,
            "展示不同产品或服务的市场占有率，适合市场分析和竞品对比",
            "替换 labels（产品名称）和 series 数据（占比）",
            vec!["市场".to_string(), "份额".to_string(), "饼图".to_string()],
            ChartData {
                chart_type: ChartType::Pie,
                title: "市场份额分布".to_string(),
                x_axis: AxisConfig::value(None),
                y_axis: AxisConfig::value(None),
                y_axis_secondary: None,
                series: vec![Series::new("份额", vec![35.0, 25.0, 22.0, 18.0])],
                options: ChartOptions::default(),
                labels: Some(vec![
                    "产品A".to_string(),
                    "产品B".to_string(),
                    "产品C".to_string(),
                    "其他".to_string(),
                ]),
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_growth_analysis_template() -> ChartTemplate {
        ChartTemplate::new(
            "growth-analysis",
            "同比增长分析",
            TemplateCategory::Business,
            "对比不同年份的同期数据，分析增长趋势",
            "替换 x 轴标签（季度/月份）和 series 数据（各年数据）",
            vec!["增长".to_string(), "对比".to_string(), "柱状图".to_string()],
            ChartData {
                chart_type: ChartType::Bar,
                title: "季度同比增长分析".to_string(),
                x_axis: AxisConfig::category(vec![
                    "Q1".to_string(),
                    "Q2".to_string(),
                    "Q3".to_string(),
                    "Q4".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("销售额（万元）".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("2023年", vec![450.0, 480.0, 520.0, 580.0]),
                    Series::new("2024年", vec![520.0, 580.0, 640.0, 720.0]),
                ],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_conversion_funnel_template() -> ChartTemplate {
        ChartTemplate::new(
            "conversion-funnel",
            "转化漏斗分析",
            TemplateCategory::Business,
            "展示用户从访问到转化的各环节流失情况",
            "替换 x 轴标签（各环节名称）和 series 数据（用户数）",
            vec!["转化".to_string(), "漏斗".to_string(), "柱状图".to_string()],
            ChartData {
                chart_type: ChartType::Bar,
                title: "用户转化漏斗".to_string(),
                x_axis: AxisConfig::category(vec![
                    "访问".to_string(),
                    "浏览商品".to_string(),
                    "加入购物车".to_string(),
                    "下单".to_string(),
                    "支付".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("用户数".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("用户数", vec![10000.0, 5000.0, 2000.0, 800.0, 650.0])],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_revenue_forecast_template() -> ChartTemplate {
        ChartTemplate::new(
            "revenue-forecast",
            "营收预测",
            TemplateCategory::Business,
            "展示历史营收和未来预测，适合财务规划",
            "替换 x 轴标签（时间）和 series 数据（实际值和预测值）",
            vec!["营收".to_string(), "预测".to_string(), "面积图".to_string()],
            ChartData {
                chart_type: ChartType::Area,
                title: "营收预测（万元）".to_string(),
                x_axis: AxisConfig::category(vec![
                    "Q1".to_string(),
                    "Q2".to_string(),
                    "Q3".to_string(),
                    "Q4(预测)".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("营收（万元）".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("实际营收", vec![850.0, 920.0, 1050.0, 0.0]),
                    Series::new("预测营收", vec![0.0, 0.0, 1050.0, 1200.0]),
                ],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    // ===== 技术监控模板 =====

    fn create_performance_metrics_template() -> ChartTemplate {
        ChartTemplate::new(
            "performance-metrics",
            "性能指标监控",
            TemplateCategory::Technical,
            "监控系统性能指标（响应时间、吞吐量等）",
            "替换 x 轴标签（时间点）和 series 数据（各指标值）",
            vec!["性能".to_string(), "监控".to_string(), "折线图".to_string()],
            ChartData {
                chart_type: ChartType::Line,
                title: "系统性能监控".to_string(),
                x_axis: AxisConfig::category(vec![
                    "00:00".to_string(),
                    "04:00".to_string(),
                    "08:00".to_string(),
                    "12:00".to_string(),
                    "16:00".to_string(),
                    "20:00".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("响应时间（ms）".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("API响应时间", vec![45.0, 38.0, 65.0, 85.0, 72.0, 55.0]),
                    Series::new("数据库查询", vec![15.0, 12.0, 22.0, 28.0, 24.0, 18.0]),
                ],
                options: ChartOptions {
                    show_legend: true,
                    show_toolbox: true,
                    smooth: true,
                },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_error_rate_template() -> ChartTemplate {
        ChartTemplate::new(
            "error-rate",
            "错误率监控",
            TemplateCategory::Technical,
            "监控系统错误率，及时发现异常",
            "替换 x 轴标签（时间）和 series 数据（错误率）",
            vec!["错误".to_string(), "监控".to_string(), "折线图".to_string()],
            ChartData {
                chart_type: ChartType::Line,
                title: "错误率监控（%）".to_string(),
                x_axis: AxisConfig::category(vec![
                    "周一".to_string(),
                    "周二".to_string(),
                    "周三".to_string(),
                    "周四".to_string(),
                    "周五".to_string(),
                    "周六".to_string(),
                    "周日".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("错误率（%）".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("错误率", vec![0.5, 0.3, 0.4, 0.8, 0.6, 0.2, 0.1])],
                options: ChartOptions {
                    show_legend: true,
                    show_toolbox: true,
                    smooth: true,
                },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_resource_usage_template() -> ChartTemplate {
        ChartTemplate::new(
            "resource-usage",
            "资源使用监控",
            TemplateCategory::Technical,
            "监控CPU、内存、磁盘等资源使用情况",
            "替换 x 轴标签（时间）和 series 数据（资源使用率）",
            vec!["资源".to_string(), "监控".to_string(), "面积图".to_string()],
            ChartData {
                chart_type: ChartType::Area,
                title: "系统资源使用率（%）".to_string(),
                x_axis: AxisConfig::category(vec![
                    "00:00".to_string(),
                    "06:00".to_string(),
                    "12:00".to_string(),
                    "18:00".to_string(),
                    "24:00".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("使用率（%）".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("CPU", vec![35.0, 28.0, 65.0, 72.0, 45.0]),
                    Series::new("内存", vec![55.0, 52.0, 68.0, 75.0, 60.0]),
                    Series::new("磁盘", vec![42.0, 43.0, 45.0, 48.0, 50.0]),
                ],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_traffic_pattern_template() -> ChartTemplate {
        ChartTemplate::new(
            "traffic-pattern",
            "流量模式分析",
            TemplateCategory::Technical,
            "分析不同时段的流量分布模式",
            "需要提供热力图数据格式",
            vec!["流量".to_string(), "模式".to_string(), "热力图".to_string()],
            ChartData {
                chart_type: ChartType::Heatmap,
                title: "流量热力图（每小时访问量）".to_string(),
                x_axis: AxisConfig::category(vec![
                    "周一".to_string(),
                    "周二".to_string(),
                    "周三".to_string(),
                    "周四".to_string(),
                    "周五".to_string(),
                ]),
                y_axis: AxisConfig::category(vec![
                    "00-06".to_string(),
                    "06-12".to_string(),
                    "12-18".to_string(),
                    "18-24".to_string(),
                ]),
                y_axis_secondary: None,
                series: vec![],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: Some(vec![
                    (0, 0, 5.0),
                    (0, 1, 20.0),
                    (0, 2, 35.0),
                    (0, 3, 15.0),
                    (1, 0, 4.0),
                    (1, 1, 22.0),
                    (1, 2, 38.0),
                    (1, 3, 18.0),
                    (2, 0, 6.0),
                    (2, 1, 25.0),
                    (2, 2, 42.0),
                    (2, 3, 20.0),
                    (3, 0, 5.0),
                    (3, 1, 28.0),
                    (3, 2, 45.0),
                    (3, 3, 22.0),
                    (4, 0, 3.0),
                    (4, 1, 18.0),
                    (4, 2, 30.0),
                    (4, 3, 12.0),
                ]),
            },
        )
    }

    fn create_api_latency_template() -> ChartTemplate {
        ChartTemplate::new(
            "api-latency",
            "API 延迟监控",
            TemplateCategory::Technical,
            "监控不同 API 的延迟情况",
            "替换 x 轴标签（API 名称）和 series 数据（延迟值）",
            vec!["API".to_string(), "延迟".to_string(), "柱状图".to_string()],
            ChartData {
                chart_type: ChartType::Bar,
                title: "API 延迟监控（ms）".to_string(),
                x_axis: AxisConfig::category(vec![
                    "/api/users".to_string(),
                    "/api/products".to_string(),
                    "/api/orders".to_string(),
                    "/api/payments".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("延迟（ms）".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("P50", vec![45.0, 38.0, 52.0, 65.0]),
                    Series::new("P95", vec![85.0, 72.0, 95.0, 120.0]),
                    Series::new("P99", vec![150.0, 125.0, 180.0, 220.0]),
                ],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    // ===== 团队管理模板 =====

    fn create_team_performance_template() -> ChartTemplate {
        ChartTemplate::new(
            "team-performance",
            "团队绩效评估",
            TemplateCategory::Team,
            "评估团队各成员或各组的绩效表现",
            "替换 x 轴标签（团队/成员）和 series 数据（评分）",
            vec!["团队".to_string(), "绩效".to_string(), "柱状图".to_string()],
            ChartData {
                chart_type: ChartType::Bar,
                title: "Q4 团队绩效评估".to_string(),
                x_axis: AxisConfig::category(vec![
                    "研发组".to_string(),
                    "产品组".to_string(),
                    "设计组".to_string(),
                    "运营组".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("评分".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("完成率", vec![92.0, 88.0, 95.0, 85.0]),
                    Series::new("质量分", vec![90.0, 85.0, 93.0, 82.0]),
                ],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_skill_radar_template() -> ChartTemplate {
        ChartTemplate::new(
            "skill-radar",
            "技能雷达图",
            TemplateCategory::Team,
            "评估团队成员的多维技能水平",
            "替换 indicators（技能维度）和 series 数据（各成员得分）",
            vec!["技能".to_string(), "评估".to_string(), "雷达图".to_string()],
            ChartData {
                chart_type: ChartType::Radar,
                title: "团队技能评估".to_string(),
                x_axis: AxisConfig::value(None),
                y_axis: AxisConfig::value(None),
                y_axis_secondary: None,
                series: vec![
                    Series::new("张三", vec![90.0, 85.0, 75.0, 88.0, 92.0]),
                    Series::new("李四", vec![85.0, 90.0, 88.0, 82.0, 85.0]),
                ],
                options: ChartOptions::default(),
                labels: None,
                indicators: Some(vec![
                    "编程能力".to_string(),
                    "设计能力".to_string(),
                    "沟通能力".to_string(),
                    "项目管理".to_string(),
                    "创新思维".to_string(),
                ]),
                heatmap_data: None,
            },
        )
    }

    fn create_workload_distribution_template() -> ChartTemplate {
        ChartTemplate::new(
            "workload-distribution",
            "工时分布",
            TemplateCategory::Team,
            "展示团队工时在不同项目或任务上的分配",
            "替换 labels（项目/任务）和 series 数据（工时占比）",
            vec!["工时".to_string(), "分布".to_string(), "饼图".to_string()],
            ChartData {
                chart_type: ChartType::Pie,
                title: "本周工时分布".to_string(),
                x_axis: AxisConfig::value(None),
                y_axis: AxisConfig::value(None),
                y_axis_secondary: None,
                series: vec![Series::new("工时占比", vec![35.0, 28.0, 20.0, 17.0])],
                options: ChartOptions::default(),
                labels: Some(vec![
                    "项目A".to_string(),
                    "项目B".to_string(),
                    "技术债务".to_string(),
                    "会议沟通".to_string(),
                ]),
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_project_progress_template() -> ChartTemplate {
        ChartTemplate::new(
            "project-progress",
            "项目进度跟踪",
            TemplateCategory::Team,
            "跟踪项目各阶段的完成情况",
            "替换 x 轴标签（阶段）和 series 数据（计划vs实际）",
            vec!["项目".to_string(), "进度".to_string(), "柱状图".to_string()],
            ChartData {
                chart_type: ChartType::Bar,
                title: "项目进度跟踪".to_string(),
                x_axis: AxisConfig::category(vec![
                    "需求分析".to_string(),
                    "设计".to_string(),
                    "开发".to_string(),
                    "测试".to_string(),
                    "上线".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("完成度（%）".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("计划进度", vec![100.0, 100.0, 80.0, 50.0, 0.0]),
                    Series::new("实际进度", vec![100.0, 95.0, 75.0, 45.0, 0.0]),
                ],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_bug_trend_template() -> ChartTemplate {
        ChartTemplate::new(
            "bug-trend",
            "Bug 趋势分析",
            TemplateCategory::Team,
            "跟踪 Bug 的新增和解决趋势",
            "替换 x 轴标签（时间）和 series 数据（新增/已解决）",
            vec!["Bug".to_string(), "趋势".to_string(), "折线图".to_string()],
            ChartData {
                chart_type: ChartType::Line,
                title: "Bug 趋势分析".to_string(),
                x_axis: AxisConfig::category(vec![
                    "第1周".to_string(),
                    "第2周".to_string(),
                    "第3周".to_string(),
                    "第4周".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("Bug 数量".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("新增", vec![25.0, 18.0, 22.0, 15.0]),
                    Series::new("已解决", vec![20.0, 22.0, 20.0, 18.0]),
                    Series::new("累计未解决", vec![35.0, 31.0, 33.0, 30.0]),
                ],
                options: ChartOptions {
                    show_legend: true,
                    show_toolbox: true,
                    smooth: true,
                },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    // ===== 学术研究模板 =====

    fn create_experiment_comparison_template() -> ChartTemplate {
        ChartTemplate::new(
            "experiment-comparison",
            "实验结果对比",
            TemplateCategory::Academic,
            "对比不同实验组的结果",
            "替换 x 轴标签（实验组）和 series 数据（各指标）",
            vec!["实验".to_string(), "对比".to_string(), "柱状图".to_string()],
            ChartData {
                chart_type: ChartType::Bar,
                title: "实验结果对比".to_string(),
                x_axis: AxisConfig::category(vec![
                    "对照组".to_string(),
                    "实验组A".to_string(),
                    "实验组B".to_string(),
                    "实验组C".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("指标值".to_string())),
                y_axis_secondary: None,
                series: vec![
                    Series::new("指标1", vec![75.0, 82.0, 88.0, 85.0]),
                    Series::new("指标2", vec![68.0, 72.0, 78.0, 75.0]),
                ],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_correlation_analysis_template() -> ChartTemplate {
        ChartTemplate::new(
            "correlation-analysis",
            "相关性分析",
            TemplateCategory::Academic,
            "分析两个变量之间的相关关系",
            "替换 series 中的 points 数据（坐标对）",
            vec!["相关性".to_string(), "分析".to_string(), "散点图".to_string()],
            ChartData {
                chart_type: ChartType::Scatter,
                title: "变量相关性分析".to_string(),
                x_axis: AxisConfig::value(Some("自变量 X".to_string())),
                y_axis: AxisConfig::value(Some("因变量 Y".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new_scatter(
                    "样本数据",
                    vec![
                        (10.0, 25.0),
                        (20.0, 38.0),
                        (30.0, 45.0),
                        (40.0, 58.0),
                        (50.0, 65.0),
                        (60.0, 72.0),
                        (70.0, 85.0),
                        (80.0, 92.0),
                    ],
                )],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_multi_factor_comparison_template() -> ChartTemplate {
        ChartTemplate::new(
            "multi-factor-comparison",
            "多因素对比",
            TemplateCategory::Academic,
            "对比多个样本在多个维度上的表现",
            "替换 indicators（维度）和 series 数据（各样本得分）",
            vec!["多因素".to_string(), "对比".to_string(), "雷达图".to_string()],
            ChartData {
                chart_type: ChartType::Radar,
                title: "多因素对比分析".to_string(),
                x_axis: AxisConfig::value(None),
                y_axis: AxisConfig::value(None),
                y_axis_secondary: None,
                series: vec![
                    Series::new("样本A", vec![85.0, 78.0, 92.0, 88.0, 80.0, 75.0]),
                    Series::new("样本B", vec![78.0, 85.0, 85.0, 82.0, 88.0, 80.0]),
                ],
                options: ChartOptions::default(),
                labels: None,
                indicators: Some(vec![
                    "因素1".to_string(),
                    "因素2".to_string(),
                    "因素3".to_string(),
                    "因素4".to_string(),
                    "因素5".to_string(),
                    "因素6".to_string(),
                ]),
                heatmap_data: None,
            },
        )
    }

    // ===== 数据探索模板 =====

    fn create_quick_preview_template() -> ChartTemplate {
        ChartTemplate::new(
            "quick-preview",
            "快速数据预览",
            TemplateCategory::Exploration,
            "快速查看数据的整体趋势",
            "替换 x 轴标签和 series 数据",
            vec!["预览".to_string(), "探索".to_string(), "折线图".to_string()],
            ChartData {
                chart_type: ChartType::Line,
                title: "数据快速预览".to_string(),
                x_axis: AxisConfig::category(vec![
                    "P1".to_string(),
                    "P2".to_string(),
                    "P3".to_string(),
                    "P4".to_string(),
                    "P5".to_string(),
                    "P6".to_string(),
                    "P7".to_string(),
                    "P8".to_string(),
                ]),
                y_axis: AxisConfig::value(Some("数值".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new("数据", vec![120.0, 132.0, 145.0, 138.0, 155.0, 170.0, 165.0, 180.0])],
                options: ChartOptions {
                    show_legend: false,
                    show_toolbox: true,
                    smooth: true,
                },
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }

    fn create_distribution_analysis_template() -> ChartTemplate {
        ChartTemplate::new(
            "distribution-analysis",
            "分布分析",
            TemplateCategory::Exploration,
            "分析数据的分布模式",
            "替换 series 中的 points 数据",
            vec!["分布".to_string(), "探索".to_string(), "散点图".to_string()],
            ChartData {
                chart_type: ChartType::Scatter,
                title: "数据分布分析".to_string(),
                x_axis: AxisConfig::value(Some("维度 X".to_string())),
                y_axis: AxisConfig::value(Some("维度 Y".to_string())),
                y_axis_secondary: None,
                series: vec![Series::new_scatter(
                    "数据点",
                    vec![
                        (15.0, 22.0),
                        (25.0, 35.0),
                        (32.0, 42.0),
                        (45.0, 55.0),
                        (52.0, 58.0),
                        (60.0, 68.0),
                        (68.0, 72.0),
                        (75.0, 85.0),
                        (82.0, 88.0),
                        (90.0, 95.0),
                    ],
                )],
                options: ChartOptions::default(),
                labels: None,
                indicators: None,
                heatmap_data: None,
            },
        )
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert_eq!(engine.all_templates().len(), 20);
    }

    #[test]
    fn test_find_template_by_id() {
        let engine = TemplateEngine::new();
        let template = engine.find_by_id("sales-trend");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "月度销售趋势");
    }

    #[test]
    fn test_filter_by_category() {
        let engine = TemplateEngine::new();
        let business_templates = engine.filter_by_category(TemplateCategory::Business);
        assert_eq!(business_templates.len(), 5);
    }

    #[test]
    fn test_search_templates() {
        let engine = TemplateEngine::new();
        let results = engine.search("销售");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_apply_template() {
        let engine = TemplateEngine::new();
        let template = engine.find_by_id("sales-trend").unwrap();
        let chart_data = template.apply();
        assert_eq!(chart_data.chart_type, ChartType::Line);
        assert_eq!(chart_data.title, "月度销售趋势");
    }
}
