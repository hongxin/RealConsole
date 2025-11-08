//! 步骤性质分析模块
//!
//! 基于易经六爻哲学，分析执行步骤的语义性质。
//!
//! # 核心理念
//!
//! 六爻位置不是简单的序列编号，而是代表不同的语义角色：
//! - 初爻（Chu）：准备阶段 - "潜龙勿用"，打基础
//! - 二爻（Er）：初始行动 - "见龙在田"，开始执行
//! - 三爻（San）：关键决策 - "终日乾乾"，关键判断
//! - 四爻（Si）：深度处理 - "或跃在渊"，深度操作
//! - 五爻（Wu）：接近完成 - "飞龙在天"，主要输出
//! - 上爻（Shang）：收尾清理 - "亢龙有悔"，善始善终
//!
//! # 设计原则
//!
//! 1. **语义分析优先**：根据步骤的真实功能判断性质，而非执行顺序
//! 2. **多维度推断**：综合工具名称、描述关键词、参数特征
//! 3. **阴阳平衡**：区分准备型（阴）和执行型（阳）操作

use crate::agent::decomposition::types::ExecutionStep;
use serde::{Deserialize, Serialize};

/// 步骤的语义性质
///
/// 代表步骤在整个执行流程中扮演的角色，而非其出现顺序
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepNature {
    /// 准备阶段（初爻）
    ///
    /// 特征：
    /// - 读取配置、加载数据、初始化环境
    /// - 检查前置条件、验证输入
    /// - 打开文件、建立连接
    ///
    /// 工具示例：read, load, init, open, connect, check_prerequisites
    Preparation,

    /// 初始执行（二爻）
    ///
    /// 特征：
    /// - 创建、写入、启动进程
    /// - 开始主要操作的第一步
    /// - 实际改变状态
    ///
    /// 工具示例：create, write, start, execute, run, insert
    Execution,

    /// 关键决策（三爻）
    ///
    /// 特征：
    /// - 搜索、查找、比对
    /// - 条件判断、分支选择
    /// - 关键信息获取
    ///
    /// 工具示例：search, find, match, check, if, choose, query
    Decision,

    /// 深度处理（四爻）
    ///
    /// 特征：
    /// - 转换、计算、排序
    /// - 数据处理、格式转换
    /// - 复杂业务逻辑
    ///
    /// 工具示例：transform, calculate, sort, filter, map, reduce, process
    Processing,

    /// 主要输出（五爻）
    ///
    /// 特征：
    /// - 显示结果、返回数据
    /// - 生成报告、输出文件
    /// - 完成核心任务
    ///
    /// 工具示例：display, show, print, return, output, generate, export
    Finalization,

    /// 收尾清理（上爻）
    ///
    /// 特征：
    /// - 关闭连接、释放资源
    /// - 保存状态、记录日志
    /// - 清理临时文件
    ///
    /// 工具示例：close, cleanup, save, log, release, delete_temp
    Cleanup,
}

impl StepNature {
    /// 获取性质的阴阳属性
    ///
    /// - 阴：准备、等待、观察型操作（Preparation, Decision）
    /// - 阳：执行、输出、改变型操作（Execution, Processing, Finalization, Cleanup）
    pub fn yin_yang(&self) -> YinYang {
        match self {
            StepNature::Preparation => YinYang::Yin,  // 准备观察
            StepNature::Execution => YinYang::Yang,   // 开始行动
            StepNature::Decision => YinYang::Yin,     // 判断选择
            StepNature::Processing => YinYang::Yang,  // 深度处理
            StepNature::Finalization => YinYang::Yang, // 输出结果
            StepNature::Cleanup => YinYang::Yang,     // 清理收尾
        }
    }

    /// 获取性质的中文描述
    pub fn chinese_name(&self) -> &'static str {
        match self {
            StepNature::Preparation => "准备阶段",
            StepNature::Execution => "初始执行",
            StepNature::Decision => "关键决策",
            StepNature::Processing => "深度处理",
            StepNature::Finalization => "主要输出",
            StepNature::Cleanup => "收尾清理",
        }
    }

    /// 获取对应的爻位特征描述
    pub fn yao_characteristic(&self) -> &'static str {
        match self {
            StepNature::Preparation => "潜龙勿用，打基础",
            StepNature::Execution => "见龙在田，初行动",
            StepNature::Decision => "终日乾乾，做决策",
            StepNature::Processing => "或跃在渊，深处理",
            StepNature::Finalization => "飞龙在天，出结果",
            StepNature::Cleanup => "亢龙有悔，善收尾",
        }
    }
}

/// 阴阳属性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YinYang {
    /// 阴：准备、观察、等待
    Yin,
    /// 阳：执行、输出、改变
    Yang,
}

impl YinYang {
    /// 获取符号表示
    pub fn symbol(&self) -> &'static str {
        match self {
            YinYang::Yin => "⚋",  // 阴爻（断）
            YinYang::Yang => "⚊", // 阳爻（连）
        }
    }
}

/// 步骤性质分析器
pub struct StepAnalyzer;

impl StepAnalyzer {
    /// 分析步骤的语义性质
    ///
    /// 综合考虑：
    /// 1. 工具名称的语义
    /// 2. 描述文本的关键词
    /// 3. 参数特征（如是否有输出路径）
    ///
    /// 返回最可能的性质分类
    pub fn analyze_nature(step: &ExecutionStep) -> StepNature {
        // 优先级 1: 基于工具名称推断
        let tool_based = Self::infer_from_tool(&step.tool);

        // 优先级 2: 基于描述关键词推断
        let desc_based = Self::infer_from_description(&step.description);

        // 优先级 3: 基于参数特征推断
        let param_based = Self::infer_from_params(step);

        // 综合判断（工具名称权重最高）
        Self::combine_inferences(tool_based, desc_based, param_based)
    }

    /// 从工具名称推断性质
    fn infer_from_tool(tool: &str) -> Option<StepNature> {
        let tool_lower = tool.to_lowercase();

        // 优先检查清理类工具（避免 close_connection 被 connect 误匹配）
        if tool_lower.contains("close")
            || tool_lower.contains("cleanup")
            || tool_lower.contains("save")
            || tool_lower.contains("log")
            || tool_lower.contains("release")
            || tool_lower.contains("delete")
            || tool_lower.contains("remove")
            || tool_lower.contains("clear")
            || tool_lower.contains("shutdown")
        {
            return Some(StepNature::Cleanup);
        }

        // 准备类工具
        if tool_lower.contains("read")
            || tool_lower.contains("load")
            || tool_lower.contains("init")
            || tool_lower.contains("open")
            || tool_lower.contains("connect")
            || tool_lower.contains("check")
            || tool_lower.contains("verify")
            || tool_lower.contains("validate")
            || tool_lower.contains("list")  // 列出文件也是准备
            || tool_lower.contains("get_config")
        {
            return Some(StepNature::Preparation);
        }

        // 执行类工具
        if tool_lower.contains("create")
            || tool_lower.contains("write")
            || tool_lower.contains("start")
            || tool_lower.contains("execute")
            || tool_lower.contains("run")
            || tool_lower.contains("insert")
            || tool_lower.contains("add")
            || tool_lower.contains("append")
            || tool_lower.contains("mkdir")
        {
            return Some(StepNature::Execution);
        }

        // 决策类工具
        if tool_lower.contains("search")
            || tool_lower.contains("find")
            || tool_lower.contains("match")
            || tool_lower.contains("query")
            || tool_lower.contains("select")
            || tool_lower.contains("choose")
            || tool_lower.contains("if")
            || tool_lower.contains("grep")
            || tool_lower.contains("locate")
        {
            return Some(StepNature::Decision);
        }

        // 处理类工具
        if tool_lower.contains("transform")
            || tool_lower.contains("calculate")
            || tool_lower.contains("compute")
            || tool_lower.contains("sort")
            || tool_lower.contains("filter")
            || tool_lower.contains("map")
            || tool_lower.contains("reduce")
            || tool_lower.contains("process")
            || tool_lower.contains("convert")
            || tool_lower.contains("parse")
        {
            return Some(StepNature::Processing);
        }

        // 输出类工具
        if tool_lower.contains("display")
            || tool_lower.contains("show")
            || tool_lower.contains("print")
            || tool_lower.contains("output")
            || tool_lower.contains("return")
            || tool_lower.contains("generate")
            || tool_lower.contains("export")
            || tool_lower.contains("render")
            || tool_lower.contains("format_output")
        {
            return Some(StepNature::Finalization);
        }

        None
    }

    /// 从描述关键词推断性质
    fn infer_from_description(desc: &str) -> Option<StepNature> {
        let desc_lower = desc.to_lowercase();

        // 准备阶段关键词
        if desc_lower.contains("读取")
            || desc_lower.contains("加载")
            || desc_lower.contains("初始化")
            || desc_lower.contains("打开")
            || desc_lower.contains("连接")
            || desc_lower.contains("检查")
            || desc_lower.contains("验证")
            || desc_lower.contains("准备")
            || desc_lower.contains("获取配置")
        {
            return Some(StepNature::Preparation);
        }

        // 执行阶段关键词
        if desc_lower.contains("创建")
            || desc_lower.contains("写入")
            || desc_lower.contains("启动")
            || desc_lower.contains("执行")
            || desc_lower.contains("运行")
            || desc_lower.contains("新建")
            || desc_lower.contains("开始")
        {
            return Some(StepNature::Execution);
        }

        // 决策阶段关键词
        if desc_lower.contains("搜索")
            || desc_lower.contains("查找")
            || desc_lower.contains("匹配")
            || desc_lower.contains("查询")
            || desc_lower.contains("选择")
            || desc_lower.contains("判断")
            || desc_lower.contains("决定")
        {
            return Some(StepNature::Decision);
        }

        // 处理阶段关键词
        if desc_lower.contains("转换")
            || desc_lower.contains("计算")
            || desc_lower.contains("排序")
            || desc_lower.contains("过滤")
            || desc_lower.contains("处理")
            || desc_lower.contains("解析")
            || desc_lower.contains("分析")
        {
            return Some(StepNature::Processing);
        }

        // 输出阶段关键词
        if desc_lower.contains("显示")
            || desc_lower.contains("输出")
            || desc_lower.contains("打印")
            || desc_lower.contains("返回")
            || desc_lower.contains("生成")
            || desc_lower.contains("导出")
            || desc_lower.contains("展示")
        {
            return Some(StepNature::Finalization);
        }

        // 清理阶段关键词
        if desc_lower.contains("关闭")
            || desc_lower.contains("清理")
            || desc_lower.contains("保存")
            || desc_lower.contains("释放")
            || desc_lower.contains("删除")
            || desc_lower.contains("记录")
            || desc_lower.contains("结束")
        {
            return Some(StepNature::Cleanup);
        }

        None
    }

    /// 从参数特征推断性质
    fn infer_from_params(step: &ExecutionStep) -> Option<StepNature> {
        if let Some(params) = &step.params {
            if let Some(obj) = params.as_object() {
                // 如果有 output_path / output_file，可能是输出步骤
                if obj.contains_key("output_path")
                    || obj.contains_key("output_file")
                    || obj.contains_key("export_to")
                {
                    return Some(StepNature::Finalization);
                }

                // 如果有 input_file / source，可能是准备步骤
                if obj.contains_key("input_file")
                    || obj.contains_key("source")
                    || obj.contains_key("config_file")
                {
                    return Some(StepNature::Preparation);
                }

                // 如果有 query / pattern，可能是决策步骤
                if obj.contains_key("query")
                    || obj.contains_key("pattern")
                    || obj.contains_key("search_term")
                {
                    return Some(StepNature::Decision);
                }
            }
        }

        None
    }

    /// 综合多个推断结果
    ///
    /// 优先级：tool > description > params
    fn combine_inferences(
        tool_based: Option<StepNature>,
        desc_based: Option<StepNature>,
        param_based: Option<StepNature>,
    ) -> StepNature {
        // 工具名称推断优先级最高
        if let Some(nature) = tool_based {
            return nature;
        }

        // 其次是描述关键词
        if let Some(nature) = desc_based {
            return nature;
        }

        // 最后是参数特征
        if let Some(nature) = param_based {
            return nature;
        }

        // 默认：如果无法推断，归类为执行步骤（最常见）
        StepNature::Execution
    }

    /// 分析整个计划的阴阳平衡情况
    ///
    /// 返回 (阴爻数, 阳爻数)
    pub fn analyze_yin_yang_balance(steps: &[ExecutionStep]) -> (usize, usize) {
        let mut yin_count = 0;
        let mut yang_count = 0;

        for step in steps {
            let nature = Self::analyze_nature(step);
            match nature.yin_yang() {
                YinYang::Yin => yin_count += 1,
                YinYang::Yang => yang_count += 1,
            }
        }

        (yin_count, yang_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_preparation_step() {
        let step = ExecutionStep::new(
            "读取配置文件".to_string(),
            "read_config".to_string(),
            0.5,
        );

        let nature = StepAnalyzer::analyze_nature(&step);
        assert_eq!(nature, StepNature::Preparation);
        assert_eq!(nature.yin_yang(), YinYang::Yin);
    }

    #[test]
    fn test_analyze_execution_step() {
        let step = ExecutionStep::new(
            "创建新文件".to_string(),
            "create_file".to_string(),
            1.0,
        );

        let nature = StepAnalyzer::analyze_nature(&step);
        assert_eq!(nature, StepNature::Execution);
        assert_eq!(nature.yin_yang(), YinYang::Yang);
    }

    #[test]
    fn test_analyze_decision_step() {
        let step = ExecutionStep::new(
            "搜索匹配项".to_string(),
            "search_pattern".to_string(),
            1.5,
        );

        let nature = StepAnalyzer::analyze_nature(&step);
        assert_eq!(nature, StepNature::Decision);
        assert_eq!(nature.yin_yang(), YinYang::Yin);
    }

    #[test]
    fn test_analyze_processing_step() {
        let step = ExecutionStep::new(
            "转换数据格式".to_string(),
            "transform_data".to_string(),
            2.0,
        );

        let nature = StepAnalyzer::analyze_nature(&step);
        assert_eq!(nature, StepNature::Processing);
        assert_eq!(nature.yin_yang(), YinYang::Yang);
    }

    #[test]
    fn test_analyze_finalization_step() {
        let step = ExecutionStep::new(
            "输出结果".to_string(),
            "display_result".to_string(),
            0.5,
        );

        let nature = StepAnalyzer::analyze_nature(&step);
        assert_eq!(nature, StepNature::Finalization);
        assert_eq!(nature.yin_yang(), YinYang::Yang);
    }

    #[test]
    fn test_analyze_cleanup_step() {
        let step = ExecutionStep::new(
            "关闭连接".to_string(),
            "close_connection".to_string(),
            0.3,
        );

        let nature = StepAnalyzer::analyze_nature(&step);
        assert_eq!(nature, StepNature::Cleanup);
        assert_eq!(nature.yin_yang(), YinYang::Yang);
    }

    #[test]
    fn test_tool_name_priority() {
        // 工具名称明确指示 "create"，即使描述是"查找"
        let step = ExecutionStep::new(
            "查找并创建".to_string(),
            "create_if_not_exists".to_string(),
            1.0,
        );

        let nature = StepAnalyzer::analyze_nature(&step);
        // 应该识别为 Execution（因为工具名有 create）
        assert_eq!(nature, StepNature::Execution);
    }

    #[test]
    fn test_description_fallback() {
        // 工具名称不明确，依靠描述
        let step = ExecutionStep::new(
            "读取用户输入".to_string(),
            "handle_input".to_string(),  // 不明确的工具名
            1.0,
        );

        let nature = StepAnalyzer::analyze_nature(&step);
        // 应该识别为 Preparation（描述有"读取"）
        assert_eq!(nature, StepNature::Preparation);
    }

    #[test]
    fn test_params_hint() {
        let step = ExecutionStep::new(
            "处理数据".to_string(),
            "process".to_string(),
            1.0,
        ).with_params(serde_json::json!({
            "output_file": "/tmp/result.txt"
        }));

        let nature = StepAnalyzer::analyze_nature(&step);
        // 工具名是 process（Processing），但有 output_file 参数（Finalization）
        // 工具名优先，应该是 Processing
        assert_eq!(nature, StepNature::Processing);
    }

    #[test]
    fn test_yin_yang_balance() {
        let steps = vec![
            ExecutionStep::new("读取".to_string(), "read".to_string(), 1.0),      // Yin
            ExecutionStep::new("创建".to_string(), "create".to_string(), 1.0),    // Yang
            ExecutionStep::new("搜索".to_string(), "search".to_string(), 1.0),    // Yin
            ExecutionStep::new("输出".to_string(), "output".to_string(), 1.0),    // Yang
        ];

        let (yin, yang) = StepAnalyzer::analyze_yin_yang_balance(&steps);
        assert_eq!(yin, 2);
        assert_eq!(yang, 2);
    }

    #[test]
    fn test_nature_descriptions() {
        let nature = StepNature::Preparation;
        assert_eq!(nature.chinese_name(), "准备阶段");
        assert_eq!(nature.yao_characteristic(), "潜龙勿用，打基础");

        let nature = StepNature::Finalization;
        assert_eq!(nature.chinese_name(), "主要输出");
        assert_eq!(nature.yao_characteristic(), "飞龙在天，出结果");
    }

    #[test]
    fn test_yin_yang_symbols() {
        assert_eq!(YinYang::Yin.symbol(), "⚋");
        assert_eq!(YinYang::Yang.symbol(), "⚊");
    }
}
