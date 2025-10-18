//! 内置工作流模板
//!
//! 提供一组常用的工作流模板，基于成功案例固化的套路

use super::workflow::{CacheStrategy, TransformOperation, WorkflowIntent, WorkflowStep};
use super::types::{EntityType, Intent, IntentDomain};
use std::collections::HashMap;

/// 注册所有内置工作流模板
pub fn register_builtin_workflows() -> Vec<WorkflowIntent> {
    vec![
        create_crypto_analysis_workflow(),
        create_stock_analysis_workflow(),
        create_weather_analysis_workflow(),
        create_website_summary_workflow(),
    ]
}

/// 创建加密货币分析工作流
///
/// **基于成功案例**: BNB 投资分析
///
/// 工作流:
/// 1. 使用 http_get 获取网站数据
/// 2. 使用 LLM 分析数据并生成投资建议
///
/// **性能优化**:
/// - LLM 调用: 从 2-3 次减少到 1 次
/// - 工具选择: 跳过，直接调用 http_get
/// - 响应时间: 预计 5-8 秒（vs 10-15 秒）
fn create_crypto_analysis_workflow() -> WorkflowIntent {
    // 定义基础意图
    let base_intent = Intent::new(
        "crypto_analysis",
        IntentDomain::Custom("Financial".to_string()),
        vec![
            "分析".to_string(),
            "加密货币".to_string(),
            "币".to_string(),
            "走势".to_string(),
            "投资".to_string(),
        ],
        vec![
            r"分析.*(?P<symbol>\w+).*走势".to_string(),
            r"(?P<symbol>\w+).*投资策略".to_string(),
            r"访问.*非小号.*分析.*(?P<symbol>\w+)".to_string(),
        ],
        0.6,
    )
    .with_entity("symbol", EntityType::Custom("crypto_symbol".to_string(), "BTC".to_string()))
    .with_entity("source_url", EntityType::Custom("url".to_string(), "https://www.feixiaohao.co".to_string()));

    // 定义工作流步骤
    let workflow_steps = vec![
        // 步骤 1: 获取网站数据
        WorkflowStep::ToolCall {
            tool_name: "http_get".to_string(),
            args_template: {
                let mut args = HashMap::new();
                args.insert("url".to_string(), "{source_url}/currencies/{symbol}".to_string());
                args.insert("timeout".to_string(), "30".to_string());
                args
            },
            result_key: "website_data".to_string(),
        },

        // 步骤 2: LLM 分析数据
        WorkflowStep::LlmAnalyze {
            prompt_template: r#"基于以下从非小号网站获取的 {symbol} 数据，请进行全面的投资分析：

数据内容：
{website_data}

请按以下结构分析：

## {symbol} 当前走势分析

### 基本信息
- 当前价格和市值
- 24小时涨跌幅
- 市值排名

### 技术指标
- 价格波动情况
- 成交量分析
- 与 BTC 相关性

### 历史表现
- 历史最高/最低价
- 投资回报率

## 投资策略建议

### 短期策略 (1-3个月)
[具体建议]

### 中期策略 (3-12个月)
[具体建议]

### 长期策略 (1年以上)
[具体建议]

## 风险提示
[列出主要风险]

## 投资建议
[适合投资者类型、仓位建议、止损建议]

请注意，以上分析仅供参考，投资有风险，请根据自身情况谨慎决策。"#.to_string(),
            result_key: "analysis_result".to_string(),
        },
    ];

    WorkflowIntent::new(base_intent, workflow_steps)
        .with_cache_strategy(CacheStrategy::TimeBased { ttl: 300 }) // 缓存 5 分钟
        .with_description("分析指定加密货币的走势和投资策略")
}

/// 创建股票分析工作流
fn create_stock_analysis_workflow() -> WorkflowIntent {
    let base_intent = Intent::new(
        "stock_analysis",
        IntentDomain::Custom("Financial".to_string()),
        vec![
            "分析".to_string(),
            "股票".to_string(),
            "走势".to_string(),
        ],
        vec![
            r"分析.*(?P<symbol>\w+).*股票".to_string(),
            r"(?P<symbol>\w+).*投资价值".to_string(),
        ],
        0.6,
    )
    .with_entity("symbol", EntityType::Custom("stock_symbol".to_string(), "600519".to_string()))
    .with_entity("source_url", EntityType::Custom("url".to_string(), "https://quote.eastmoney.com".to_string()));

    let workflow_steps = vec![
        WorkflowStep::ToolCall {
            tool_name: "http_get".to_string(),
            args_template: {
                let mut args = HashMap::new();
                args.insert("url".to_string(), "{source_url}/stock/{symbol}.html".to_string());
                args.insert("timeout".to_string(), "30".to_string());
                args
            },
            result_key: "stock_data".to_string(),
        },

        WorkflowStep::LlmAnalyze {
            prompt_template: r#"基于以下股票数据，分析 {symbol} 的投资价值：

{stock_data}

请提供：
1. 基本面分析
2. 技术面分析
3. 估值分析
4. 投资建议"#.to_string(),
            result_key: "analysis_result".to_string(),
        },
    ];

    WorkflowIntent::new(base_intent, workflow_steps)
        .with_cache_strategy(CacheStrategy::TimeBased { ttl: 600 }) // 缓存 10 分钟
        .with_description("分析指定股票的投资价值")
}

/// 创建天气分析工作流
fn create_weather_analysis_workflow() -> WorkflowIntent {
    let base_intent = Intent::new(
        "weather_analysis",
        IntentDomain::Custom("Weather".to_string()),
        vec![
            "天气".to_string(),
            "预报".to_string(),
            "分析".to_string(),
        ],
        vec![
            r"分析.*(?P<city>\w+).*天气".to_string(),
            r"(?P<city>\w+).*未来.*天气".to_string(),
        ],
        0.6,
    )
    .with_entity("city", EntityType::Custom("city".to_string(), "北京".to_string()));

    let workflow_steps = vec![
        WorkflowStep::ToolCall {
            tool_name: "http_get".to_string(),
            args_template: {
                let mut args = HashMap::new();
                args.insert("url".to_string(), "http://www.weather.com.cn/weather/{city}.shtml".to_string());
                args.insert("timeout".to_string(), "20".to_string());
                args
            },
            result_key: "weather_data".to_string(),
        },

        WorkflowStep::LlmAnalyze {
            prompt_template: r#"基于以下天气数据，分析 {city} 未来一周的天气趋势：

{weather_data}

请提供：
1. 天气概况
2. 温度变化趋势
3. 降水可能性
4. 生活建议"#.to_string(),
            result_key: "analysis_result".to_string(),
        },
    ];

    WorkflowIntent::new(base_intent, workflow_steps)
        .with_cache_strategy(CacheStrategy::TimeBased { ttl: 1800 }) // 缓存 30 分钟
        .with_description("分析指定城市的天气趋势")
}

/// 创建网站内容摘要工作流
fn create_website_summary_workflow() -> WorkflowIntent {
    let base_intent = Intent::new(
        "website_summary",
        IntentDomain::Custom("Web".to_string()),
        vec![
            "总结".to_string(),
            "摘要".to_string(),
            "网站".to_string(),
        ],
        vec![
            r"总结.*网站.*内容".to_string(),
            r"访问.*(?P<url>https?://\S+).*摘要".to_string(),
        ],
        0.5,
    )
    .with_entity("url", EntityType::Custom("url".to_string(), "https://example.com".to_string()));

    let workflow_steps = vec![
        WorkflowStep::ToolCall {
            tool_name: "http_get".to_string(),
            args_template: {
                let mut args = HashMap::new();
                args.insert("url".to_string(), "{url}".to_string());
                args.insert("timeout".to_string(), "30".to_string());
                args
            },
            result_key: "website_content".to_string(),
        },

        // 数据转换：截断过长的内容
        WorkflowStep::Transform {
            operation: TransformOperation::Truncate { max_length: 5000 },
            input_key: "website_content".to_string(),
            result_key: "truncated_content".to_string(),
        },

        WorkflowStep::LlmAnalyze {
            prompt_template: r#"请总结以下网站内容的核心要点：

网址：{url}

内容：
{truncated_content}

请提供：
1. 主题摘要（100字以内）
2. 核心要点（3-5 条）
3. 适合人群"#.to_string(),
            result_key: "summary_result".to_string(),
        },
    ];

    WorkflowIntent::new(base_intent, workflow_steps)
        .with_cache_strategy(CacheStrategy::ParameterBased) // 基于参数缓存
        .with_description("总结网站内容的核心要点")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_builtin_workflows() {
        let workflows = register_builtin_workflows();
        assert!(workflows.len() >= 4);

        // 验证加密货币分析模板
        let crypto_workflow = workflows.iter()
            .find(|w| w.base_intent.name == "crypto_analysis")
            .expect("crypto_analysis workflow should exist");

        assert_eq!(crypto_workflow.workflow_steps.len(), 2);
        assert!(crypto_workflow.base_intent.keywords.contains(&"分析".to_string()));
    }

    #[test]
    fn test_crypto_workflow_structure() {
        let workflow = create_crypto_analysis_workflow();

        // 验证基础信息
        assert_eq!(workflow.base_intent.name, "crypto_analysis");
        assert_eq!(workflow.workflow_steps.len(), 2);

        // 验证第一步是工具调用
        match &workflow.workflow_steps[0] {
            WorkflowStep::ToolCall { tool_name, .. } => {
                assert_eq!(tool_name, "http_get");
            }
            _ => panic!("First step should be ToolCall"),
        }

        // 验证第二步是 LLM 分析
        match &workflow.workflow_steps[1] {
            WorkflowStep::LlmAnalyze { .. } => {
                // OK
            }
            _ => panic!("Second step should be LlmAnalyze"),
        }
    }
}
