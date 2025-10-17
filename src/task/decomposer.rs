//! 任务分解器 (TaskDecomposer)
//!
//! Phase 10: 任务分解与规划系统
//!
//! 负责将用户的高层次目标分解为可执行的子任务序列

use super::error::{TaskError, TaskResult};
use super::types::{ExecutionContext, SubTask, TaskType};
use crate::llm::LlmClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// LLM 响应格式
#[derive(Debug, Serialize, Deserialize)]
struct TaskListResponse {
    tasks: Vec<SubTaskJson>,
}

/// 子任务 JSON 格式（用于 LLM 响应解析）
#[derive(Debug, Serialize, Deserialize)]
struct SubTaskJson {
    id: String,
    name: String,
    description: String,
    command: String,
    estimated_time: u32,
    #[serde(default)]
    depends_on: Vec<String>,
    task_type: String,
    #[serde(default)]
    skippable: bool,
}

impl From<SubTaskJson> for SubTask {
    fn from(json: SubTaskJson) -> Self {
        let task_type = match json.task_type.to_lowercase().as_str() {
            "shell" => TaskType::Shell,
            "fileoperation" => TaskType::FileOperation,
            "network" => TaskType::Network,
            "validation" => TaskType::Validation,
            "userinput" => TaskType::UserInput,
            _ => TaskType::Shell, // 默认为 Shell
        };

        SubTask {
            id: json.id,
            name: json.name,
            description: json.description,
            command: json.command,
            estimated_time: json.estimated_time,
            depends_on: json.depends_on,
            task_type,
            skippable: json.skippable,
            retry_policy: None,
        }
    }
}

/// 分解记录（用于学习和优化）
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DecompositionRecord {
    goal: String,
    context: ExecutionContext,
    subtasks: Vec<SubTask>,
    timestamp: chrono::DateTime<chrono::Utc>,
    success: bool,
}

/// 任务分解器
///
/// 使用 LLM 将复杂任务分解为可执行的子任务序列
pub struct TaskDecomposer {
    /// LLM 客户端
    llm: Arc<dyn LlmClient>,

    /// 历史分解记录（用于学习）
    history: Arc<RwLock<Vec<DecompositionRecord>>>,

    /// 最大子任务数量限制
    max_subtasks: usize,
}

impl TaskDecomposer {
    /// 创建新的任务分解器
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
        Self {
            llm,
            history: Arc::new(RwLock::new(Vec::new())),
            max_subtasks: 20,
        }
    }

    /// 设置最大子任务数量
    pub fn with_max_subtasks(mut self, max: usize) -> Self {
        self.max_subtasks = max;
        self
    }

    /// 分解任务
    ///
    /// # Arguments
    /// * `goal` - 用户目标描述
    /// * `context` - 当前执行上下文
    ///
    /// # Returns
    /// * `Vec<SubTask>` - 子任务列表
    pub async fn decompose(
        &self,
        goal: &str,
        context: &ExecutionContext,
    ) -> TaskResult<Vec<SubTask>> {
        // 1. 使用 LLM 分解任务
        let subtasks = self.decompose_with_llm(goal, context).await?;

        // 2. 验证任务合理性
        let validated_tasks = self.validate_tasks(subtasks)?;

        // 3. 记录历史
        self.record_decomposition(goal, context, &validated_tasks, true)
            .await;

        Ok(validated_tasks)
    }

    /// 使用 LLM 分解任务
    async fn decompose_with_llm(
        &self,
        goal: &str,
        context: &ExecutionContext,
    ) -> TaskResult<Vec<SubTask>> {
        let prompt = self.build_decomposition_prompt(goal, context);

        // 调用 LLM
        let messages = vec![crate::llm::Message::user(prompt)];
        let response = self
            .llm
            .chat(messages)
            .await
            .map_err(|e| TaskError::LlmError(e.to_string()))?;

        // 解析 JSON 响应
        self.parse_llm_response(&response)
    }

    /// 构建分解提示词
    fn build_decomposition_prompt(&self, goal: &str, context: &ExecutionContext) -> String {
        format!(
            r#"你是一个任务分解专家。请将以下目标分解为可执行的子任务序列。

目标: {}

当前上下文:
- 工作目录: {}
- 系统: {}
- Shell: {}
- 用户: {}

请按以下 JSON 格式输出任务列表:
{{
  "tasks": [
    {{
      "id": "task1",
      "name": "任务名称",
      "description": "详细描述这个任务要做什么",
      "command": "要执行的具体命令",
      "estimated_time": 30,
      "depends_on": [],
      "task_type": "Shell",
      "skippable": false
    }},
    {{
      "id": "task2",
      "name": "下一个任务",
      "description": "任务描述",
      "command": "命令",
      "estimated_time": 20,
      "depends_on": ["task1"],
      "task_type": "Shell",
      "skippable": false
    }}
  ]
}}

要求:
1. 任务应该具体可执行，避免过于抽象
2. 命令应该是有效的 shell 命令（适用于 {} 系统）
3. 正确标识任务间的依赖关系（depends_on 数组包含前置任务的 id）
4. 提供合理的时间估计（单位：秒）
5. 按执行顺序排列任务
6. task_type 只能是: Shell, FileOperation, Network, Validation, UserInput
7. 如果任务失败可以跳过，设置 skippable 为 true
8. 任务数量不超过 {} 个
9. 每个任务的 id 必须唯一
10. 输出必须是有效的 JSON 格式

注意：
- 只输出 JSON，不要有其他解释文字
- 使用双引号，不要使用单引号
- 命令字符串中的特殊字符需要转义

【重要】Shell 命令执行规则：
- 每个命令在独立的 shell 进程中执行
- cd 命令不会影响后续命令的工作目录
- 如需在特定目录执行命令，必须使用以下方式之一：
  1. 使用绝对路径：mkdir /path/to/dir && /path/to/dir/script.sh
  2. 使用 cd && 连接符：cd target_dir && gcc hello.c -o hello
  3. 使用子shell：(cd target_dir && make)
- 不要生成单独的 cd 命令作为独立任务
- 涉及目录切换的操作应合并到一个命令中
"#,
            goal,
            context.working_dir,
            context.os,
            context.shell,
            context.user,
            context.os,
            self.max_subtasks
        )
    }

    /// 解析 LLM 响应
    fn parse_llm_response(&self, response: &str) -> TaskResult<Vec<SubTask>> {
        // 尝试从响应中提取 JSON
        let json_str = self.extract_json(response)?;

        // 解析 JSON
        let task_list: TaskListResponse = serde_json::from_str(&json_str)
            .map_err(|e| TaskError::ParseError(format!("JSON 解析失败: {}", e)))?;

        // 转换为 SubTask
        let subtasks: Vec<SubTask> = task_list.tasks.into_iter().map(SubTask::from).collect();

        Ok(subtasks)
    }

    /// 从响应中提取 JSON
    ///
    /// LLM 可能返回的格式：
    /// 1. 纯 JSON
    /// 2. ```json ... ```
    /// 3. 带解释的 JSON
    fn extract_json(&self, response: &str) -> TaskResult<String> {
        let trimmed = response.trim();

        // 尝试1: 直接解析
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            return Ok(trimmed.to_string());
        }

        // 尝试2: 查找 ```json ... ``` 代码块
        if let Some(start) = trimmed.find("```json") {
            let after_start = &trimmed[start + 7..];
            if let Some(end) = after_start.find("```") {
                let json = &after_start[..end].trim();
                return Ok(json.to_string());
            }
        }

        // 尝试3: 查找第一个 { 到最后一个 }
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                if end > start {
                    return Ok(trimmed[start..=end].to_string());
                }
            }
        }

        Err(TaskError::ParseError(
            "无法从 LLM 响应中提取有效的 JSON".to_string(),
        ))
    }

    /// 验证任务合理性
    fn validate_tasks(&self, mut tasks: Vec<SubTask>) -> TaskResult<Vec<SubTask>> {
        // 1. 检查任务数量
        if tasks.is_empty() {
            return Err(TaskError::ParseError("任务列表为空".to_string()));
        }

        if tasks.len() > self.max_subtasks {
            // 截断到最大数量
            tasks.truncate(self.max_subtasks);
        }

        // 2. 检查 ID 唯一性
        let mut ids = std::collections::HashSet::new();
        for task in &tasks {
            if !ids.insert(task.id.clone()) {
                return Err(TaskError::ParseError(format!(
                    "任务 ID 重复: {}",
                    task.id
                )));
            }
        }

        // 3. 检查依赖有效性
        for task in &tasks {
            for dep_id in &task.depends_on {
                if !ids.contains(dep_id) {
                    return Err(TaskError::ParseError(format!(
                        "任务 {} 依赖的任务 {} 不存在",
                        task.id, dep_id
                    )));
                }
            }
        }

        // 4. 检查命令非空
        for task in &tasks {
            if task.command.trim().is_empty() {
                return Err(TaskError::ParseError(format!(
                    "任务 {} 的命令为空",
                    task.id
                )));
            }
        }

        Ok(tasks)
    }

    /// 记录分解历史
    async fn record_decomposition(
        &self,
        goal: &str,
        context: &ExecutionContext,
        subtasks: &[SubTask],
        success: bool,
    ) {
        let record = DecompositionRecord {
            goal: goal.to_string(),
            context: context.clone(),
            subtasks: subtasks.to_vec(),
            timestamp: chrono::Utc::now(),
            success,
        };

        let mut history = self.history.write().await;
        history.push(record);

        // 限制历史记录数量（保留最近 100 条）
        if history.len() > 100 {
            let drain_count = history.len() - 100;
            history.drain(0..drain_count);
        }
    }

    /// 获取历史分解记录数量
    pub async fn history_count(&self) -> usize {
        self.history.read().await.len()
    }

    /// 清空历史记录
    pub async fn clear_history(&self) {
        self.history.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ClientStats, LlmError, Message};
    use async_trait::async_trait;

    // Mock LLM 客户端
    struct MockLlmClient {
        response: String,
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn chat(&self, _messages: Vec<Message>) -> Result<String, LlmError> {
            Ok(self.response.clone())
        }

        fn model(&self) -> &str {
            "mock"
        }

        fn stats(&self) -> ClientStats {
            ClientStats::new()
        }

        async fn diagnose(&self) -> String {
            "Mock LLM Client".to_string()
        }
    }

    #[tokio::test]
    async fn test_decompose_simple_task() {
        let response = r#"{
            "tasks": [
                {
                    "id": "task1",
                    "name": "Run tests",
                    "description": "Execute unit tests",
                    "command": "npm test",
                    "estimated_time": 30,
                    "depends_on": [],
                    "task_type": "Shell",
                    "skippable": false
                }
            ]
        }"#;

        let llm = Arc::new(MockLlmClient {
            response: response.to_string(),
        });
        let decomposer = TaskDecomposer::new(llm);
        let context = ExecutionContext::current();

        let tasks = decomposer
            .decompose("run tests", &context)
            .await
            .unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "task1");
        assert_eq!(tasks[0].name, "Run tests");
        assert_eq!(tasks[0].command, "npm test");
    }

    #[tokio::test]
    async fn test_decompose_with_dependencies() {
        let response = r#"{
            "tasks": [
                {
                    "id": "t1",
                    "name": "Install",
                    "description": "Install dependencies",
                    "command": "npm install",
                    "estimated_time": 30,
                    "depends_on": [],
                    "task_type": "Shell",
                    "skippable": false
                },
                {
                    "id": "t2",
                    "name": "Test",
                    "description": "Run tests",
                    "command": "npm test",
                    "estimated_time": 20,
                    "depends_on": ["t1"],
                    "task_type": "Shell",
                    "skippable": false
                }
            ]
        }"#;

        let llm = Arc::new(MockLlmClient {
            response: response.to_string(),
        });
        let decomposer = TaskDecomposer::new(llm);
        let context = ExecutionContext::current();

        let tasks = decomposer.decompose("test", &context).await.unwrap();

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[1].depends_on, vec!["t1"]);
    }

    #[test]
    fn test_extract_json_pure() {
        let decomposer = TaskDecomposer::new(Arc::new(MockLlmClient {
            response: String::new(),
        }));

        let json = r#"{"tasks": []}"#;
        let result = decomposer.extract_json(json).unwrap();
        assert_eq!(result, json);
    }

    #[test]
    fn test_extract_json_with_code_block() {
        let decomposer = TaskDecomposer::new(Arc::new(MockLlmClient {
            response: String::new(),
        }));

        let response = r#"Here is the task list:
```json
{"tasks": []}
```
"#;
        let result = decomposer.extract_json(response).unwrap();
        assert_eq!(result, r#"{"tasks": []}"#);
    }

    #[test]
    fn test_extract_json_with_text() {
        let decomposer = TaskDecomposer::new(Arc::new(MockLlmClient {
            response: String::new(),
        }));

        let response = r#"Some explanation text {"tasks": []} more text"#;
        let result = decomposer.extract_json(response).unwrap();
        assert_eq!(result, r#"{"tasks": []}"#);
    }

    #[test]
    fn test_validate_tasks_empty() {
        let decomposer = TaskDecomposer::new(Arc::new(MockLlmClient {
            response: String::new(),
        }));

        let result = decomposer.validate_tasks(vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tasks_duplicate_id() {
        let decomposer = TaskDecomposer::new(Arc::new(MockLlmClient {
            response: String::new(),
        }));

        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t1", "Task 2", "cmd2"), // Duplicate ID
        ];

        let result = decomposer.validate_tasks(tasks);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tasks_invalid_dependency() {
        let decomposer = TaskDecomposer::new(Arc::new(MockLlmClient {
            response: String::new(),
        }));

        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t2", "Task 2", "cmd2").with_dependency("t3"), // t3 doesn't exist
        ];

        let result = decomposer.validate_tasks(tasks);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tasks_empty_command() {
        let decomposer = TaskDecomposer::new(Arc::new(MockLlmClient {
            response: String::new(),
        }));

        let mut task = SubTask::new("t1", "Task 1", "");
        task.command = "   ".to_string(); // Empty command

        let result = decomposer.validate_tasks(vec![task]);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_history_recording() {
        let llm = Arc::new(MockLlmClient {
            response: r#"{"tasks": [{"id":"t1","name":"Test","description":"desc","command":"cmd","estimated_time":10,"task_type":"Shell"}]}"#.to_string(),
        });

        let decomposer = TaskDecomposer::new(llm);
        let context = ExecutionContext::current();

        assert_eq!(decomposer.history_count().await, 0);

        let _ = decomposer.decompose("test goal", &context).await.unwrap();

        assert_eq!(decomposer.history_count().await, 1);

        decomposer.clear_history().await;
        assert_eq!(decomposer.history_count().await, 0);
    }
}
