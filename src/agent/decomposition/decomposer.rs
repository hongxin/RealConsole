//! 意图拆解器

use super::types::{ExecutionPlan, ExecutionStep};
use crate::llm::LlmClient;
use anyhow::{anyhow, Result};
use std::sync::Arc;

/// 意图拆解器
///
/// 使用 LLM 将自然语言意图拆解为可执行的步骤计划
pub struct IntentDecomposer {
    llm: Arc<dyn LlmClient>,
}

impl IntentDecomposer {
    /// 创建新的意图拆解器
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
        Self { llm }
    }

    /// 将自然语言意图拆解为执行计划
    ///
    /// # 参数
    ///
    /// - `input`: 用户的自然语言输入
    ///
    /// # 返回
    ///
    /// 解析后的执行计划
    ///
    /// # 示例
    ///
    /// ```no_run
    /// let plan = decomposer.decompose("加载 data.csv 并显示前 10 行").await?;
    /// println!("理解: {}", plan.understanding);
    /// println!("步骤数: {}", plan.step_count());
    /// ```
    pub async fn decompose(&self, input: &str) -> Result<ExecutionPlan> {
        // 1. 构建拆解 prompt
        let prompt = self.build_decomposition_prompt(input)?;

        // 2. 调用 LLM 进行拆解
        use crate::llm::Message;
        let messages = vec![Message::user(prompt)];
        let response = self.llm.chat(messages).await?;

        // 3. 解析 JSON 响应
        let plan = self.parse_response(&response)?;

        // 4. 验证计划合法性
        self.validate_plan(&plan)?;

        Ok(plan)
    }

    /// 构建拆解 prompt
    fn build_decomposition_prompt(&self, input: &str) -> Result<String> {
        // TODO: 添加可用工具列表
        // TODO: 添加上下文信息
        let available_tools = self.get_available_tools();

        let prompt = format!(
            r#"你是一个任务拆解专家。请将用户的自然语言请求拆解为具体的执行步骤。

用户输入：{}

可用工具：
{}

请以 JSON 格式返回执行计划：
{{
  "understanding": "对用户意图的理解（1-2 句话）",
  "steps": [
    {{
      "description": "步骤描述",
      "tool": "工具名称",
      "params": {{"参数名": "参数值"}},
      "estimated_time": 1.2
    }}
  ]
}}

要求：
1. 步骤描述要具体、可执行
2. tool 必须是可用工具列表中的工具名称（如 read_file, shell_execute）
3. params 必须是 JSON 对象，包含工具所需的参数（参考上面的参数示例）
4. 优先使用专用工具（如 read_file），而非 shell_execute
5. 时间估计要合理（秒）
6. 步骤之间要有逻辑顺序
7. 步骤数量控制在 3-10 步之间
8. 只返回 JSON，不要有其他说明文字

示例：
用户输入：读取 config.yaml 文件
正确输出：
{{
  "understanding": "用户需要读取 config.yaml 文件的内容",
  "steps": [
    {{
      "description": "读取配置文件",
      "tool": "read_file",
      "params": {{"path": "config.yaml"}},
      "estimated_time": 0.5
    }}
  ]
}}
"#,
            input, available_tools
        );

        Ok(prompt)
    }

    /// 解析 LLM 响应
    fn parse_response(&self, response: &str) -> Result<ExecutionPlan> {
        // 提取 JSON 部分（LLM 可能返回额外文字）
        let json_str = self.extract_json(response)?;

        // 解析 JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| anyhow!("JSON 解析失败: {}", e))?;

        // 提取字段
        let understanding = parsed["understanding"]
            .as_str()
            .ok_or_else(|| anyhow!("缺少 understanding 字段"))?
            .to_string();

        let steps_array = parsed["steps"]
            .as_array()
            .ok_or_else(|| anyhow!("缺少 steps 数组"))?;

        // 解析步骤
        let mut steps = Vec::new();
        for (index, step_value) in steps_array.iter().enumerate() {
            let description = step_value["description"]
                .as_str()
                .ok_or_else(|| anyhow!("步骤 {} 缺少 description", index))?
                .to_string();

            let tool = step_value["tool"]
                .as_str()
                .ok_or_else(|| anyhow!("步骤 {} 缺少 tool", index))?
                .to_string();

            let estimated_time = step_value["estimated_time"]
                .as_f64()
                .unwrap_or(1.0);

            // v1.30.0: 解析 params 字段
            let params = step_value.get("params").cloned();

            let mut step = ExecutionStep::new(description, tool, estimated_time);
            step.params = params;
            steps.push(step);
        }

        Ok(ExecutionPlan::new(understanding, steps))
    }

    /// 从响应中提取 JSON（处理 LLM 可能的额外输出）
    fn extract_json(&self, response: &str) -> Result<String> {
        // 尝试直接解析
        if response.trim().starts_with('{') {
            return Ok(response.trim().to_string());
        }

        // 查找 JSON 代码块
        if let Some(start) = response.find("```json") {
            let json_start = start + 7; // "```json".len()
            // 从 json_start 之后查找结束的 ```
            if let Some(end_offset) = response[json_start..].find("```") {
                let json_end = json_start + end_offset;
                return Ok(response[json_start..json_end].trim().to_string());
            }
        }

        // 查找花括号
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                return Ok(response[start..=end].to_string());
            }
        }

        Err(anyhow!("无法从响应中提取 JSON: {}", response))
    }

    /// 验证计划合法性
    fn validate_plan(&self, plan: &ExecutionPlan) -> Result<()> {
        // 1. 检查步骤数量
        if plan.steps.is_empty() {
            return Err(anyhow!("执行计划不能为空"));
        }

        if plan.steps.len() > 20 {
            return Err(anyhow!("执行步骤过多（最多 20 步）"));
        }

        // 2. 验证每个步骤
        for (index, step) in plan.steps.iter().enumerate() {
            if step.description.is_empty() {
                return Err(anyhow!("步骤 {} 描述为空", index));
            }

            if step.tool.is_empty() {
                return Err(anyhow!("步骤 {} 工具名称为空", index));
            }

            if step.estimated_time <= 0.0 {
                return Err(anyhow!("步骤 {} 预计时间必须大于 0", index));
            }
        }

        // 3. 验证总时间
        if plan.total_estimated_time <= 0.0 {
            return Err(anyhow!("总执行时间必须大于 0"));
        }

        Ok(())
    }

    /// 获取可用工具列表（v1.30.0: 真实工具清单）
    fn get_available_tools(&self) -> String {
        r#"1. read_file - 读取文件内容
   参数: {"path": "文件路径"}
   示例: {"path": "config.yaml"}

2. write_file - 写入文件内容
   参数: {"path": "文件路径", "content": "内容"}
   示例: {"path": "output.txt", "content": "Hello World"}

3. list_dir - 列出目录内容
   参数: {"path": "目录路径"}（可选，默认当前目录）
   示例: {"path": "."} 或 {"path": "src"}

4. shell_execute - 执行 Shell 命令
   参数: {"command": "命令"}
   示例: {"command": "ls -la"}
   注意: 禁止危险命令（rm, sudo, chmod 等）

5. calculator - 数学计算
   参数: {"expression": "表达式"}
   示例: {"expression": "2 + 3 * 4"}

6. get_datetime - 获取当前日期时间
   参数: {}（无需参数）

7. count_code_lines - 统计代码行数
   参数: {"path": "目录或文件", "extensions": ["扩展名列表"]}
   示例: {"path": "src", "extensions": ["rs", "toml"]}

8. lunar_date - 农历日期转换
   参数: {"date": "日期"}（可选）
   示例: {}"#
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json() {
        let decomposer = create_test_decomposer();

        // 纯 JSON
        let json1 = r#"{"key": "value"}"#;
        assert!(decomposer.extract_json(json1).is_ok());

        // 带代码块
        let json2 = r#"```json
{"key": "value"}
```"#;
        assert!(decomposer.extract_json(json2).is_ok());

        // 带额外文字
        let json3 = r#"这是一个 JSON: {"key": "value"} 结束"#;
        assert!(decomposer.extract_json(json3).is_ok());
    }

    #[test]
    fn test_parse_response() {
        let decomposer = create_test_decomposer();

        let response = r#"{
  "understanding": "加载并显示数据",
  "steps": [
    {
      "description": "读取文件",
      "tool": "file_read",
      "estimated_time": 1.0
    },
    {
      "description": "显示数据",
      "tool": "display",
      "estimated_time": 0.5
    }
  ]
}"#;

        let plan = decomposer.parse_response(response).unwrap();
        assert_eq!(plan.understanding, "加载并显示数据");
        assert_eq!(plan.step_count(), 2);
        assert_eq!(plan.steps[0].tool, "file_read");
    }

    // 简单的 Mock LLM 用于测试
    struct MockLlm;

    #[async_trait::async_trait]
    impl crate::llm::LlmClient for MockLlm {
        async fn chat(&self, _messages: Vec<crate::llm::Message>) -> Result<String, crate::llm::LlmError> {
            Ok("Mock response".to_string())
        }

        async fn chat_with_tools(
            &self,
            _messages: Vec<crate::llm::Message>,
            _tools: Vec<serde_json::Value>,
        ) -> Result<crate::llm::ChatResponse, crate::llm::LlmError> {
            unimplemented!()
        }

        fn model(&self) -> &str {
            "mock-model"
        }

        fn stats(&self) -> crate::llm::ClientStats {
            crate::llm::ClientStats::default()
        }

        async fn diagnose(&self) -> String {
            "Mock LLM: OK".to_string()
        }
    }

    fn create_test_decomposer() -> IntentDecomposer {
        IntentDecomposer::new(Arc::new(MockLlm))
    }
}
