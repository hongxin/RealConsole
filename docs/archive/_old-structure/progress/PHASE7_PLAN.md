# Phase 7 实施计划：LLM 驱动的 Pipeline 生成

**创建日期**: 2025-10-16
**状态**: 🚀 规划中
**预期耗时**: 2-3小时

---

## 🎯 目标

让 LLM 参与意图理解，动态生成结构化的 Pipeline 执行计划，实现真正的自然语言理解。

### 核心理念

> 不再预定义所有 Intent，而是让 LLM 理解用户意图，生成基础操作的组合。

**从**：
```
用户输入 → 规则匹配 → 固定模板 → 命令
```

**到**：
```
用户输入 → LLM 理解 → 结构化计划 → Pipeline DSL → 命令
```

---

## 📋 实施步骤

### Step 1: 设计结构化输出 Schema

**目标**: 定义 LLM 输出的 JSON 格式

**Schema 设计**:
```json
{
  "intent_type": "file_operations",
  "base_operation": {
    "type": "find_files",
    "parameters": {
      "path": ".",
      "pattern": "*.rs"
    }
  },
  "modifiers": [
    {
      "type": "sort",
      "field": "size",
      "direction": "ascending"
    },
    {
      "type": "limit",
      "count": 1
    }
  ],
  "explanation": "查找当前目录下所有.rs文件，按大小升序排列，显示最小的那个"
}
```

**Rust 数据结构**:
```rust
#[derive(Debug, Deserialize, Serialize)]
struct LlmIntent {
    intent_type: String,
    base_operation: BaseOpJson,
    modifiers: Vec<ModifierJson>,
    explanation: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct BaseOpJson {
    #[serde(rename = "type")]
    op_type: String,
    parameters: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModifierJson {
    #[serde(rename = "type")]
    op_type: String,
    #[serde(flatten)]
    parameters: HashMap<String, Value>,
}
```

---

### Step 2: 设计 LLM Prompt 模板

**目标**: 让 LLM 输出正确的 JSON 格式

**System Prompt**:
```
你是 RealConsole 的意图理解助手。你的任务是将用户的自然语言转换为结构化的文件操作计划。

## 可用的基础操作

### 1. find_files - 查找文件
参数：
- path (string): 搜索路径，默认 "."
- pattern (string): 文件名模式，如 "*.rs", "*.py", "*"

### 2. disk_usage - 检查磁盘使用
参数：
- path (string): 目录路径，默认 "."

### 3. list_files - 列出文件
参数：
- path (string): 目录路径，默认 "."

## 可用的修饰操作

### 1. sort - 排序
参数：
- field (string): "size" | "time" | "name" | "default"
- direction (string): "ascending" (升序/最小) | "descending" (降序/最大)

### 2. limit - 限制数量
参数：
- count (number): 显示前N个结果

### 3. filter - 过滤
参数：
- condition (string): 过滤条件

## 输出格式

必须输出有效的 JSON，格式如下：
{
  "intent_type": "file_operations",
  "base_operation": {
    "type": "基础操作类型",
    "parameters": { 参数字典 }
  },
  "modifiers": [
    {
      "type": "修饰操作类型",
      ...参数
    }
  ],
  "explanation": "简短的中文解释"
}

## 关键映射规则

1. "最大" / "最多" / "大于" → direction: "descending"
2. "最小" / "最少" / "小于" → direction: "ascending"
3. "最近" / "最新" → field: "time", direction: "descending"
4. "最旧" → field: "time", direction: "ascending"
5. 没有指定方向时，默认 "descending"

## 示例

用户输入: "显示当前目录下体积最小的rs文件"
输出:
{
  "intent_type": "file_operations",
  "base_operation": {
    "type": "find_files",
    "parameters": {
      "path": ".",
      "pattern": "*.rs"
    }
  },
  "modifiers": [
    {
      "type": "sort",
      "field": "size",
      "direction": "ascending"
    },
    {
      "type": "limit",
      "count": 1
    }
  ],
  "explanation": "查找.rs文件，按大小升序，取第1个（最小）"
}

现在请处理用户输入。只输出 JSON，不要其他内容。
```

---

### Step 3: 实现 JSON → ExecutionPlan 转换器

**文件**: `src/dsl/intent/llm_bridge.rs`

**核心功能**:
```rust
pub struct LlmToPipeline {
    llm_client: Arc<dyn LlmClient>,
}

impl LlmToPipeline {
    /// 使用 LLM 理解用户输入，生成 ExecutionPlan
    pub async fn understand_and_generate(
        &self,
        user_input: &str,
    ) -> Result<ExecutionPlan, String> {
        // 1. 调用 LLM
        let llm_response = self.call_llm(user_input).await?;

        // 2. 解析 JSON
        let llm_intent: LlmIntent = serde_json::from_str(&llm_response)?;

        // 3. 转换为 ExecutionPlan
        let plan = self.to_execution_plan(llm_intent)?;

        Ok(plan)
    }

    fn to_execution_plan(&self, intent: LlmIntent) -> Result<ExecutionPlan> {
        let mut plan = ExecutionPlan::new();

        // 添加基础操作
        match intent.base_operation.op_type.as_str() {
            "find_files" => {
                let path = intent.base_operation.parameters
                    .get("path").and_then(|v| v.as_str()).unwrap_or(".");
                let pattern = intent.base_operation.parameters
                    .get("pattern").and_then(|v| v.as_str()).unwrap_or("*");

                plan = plan.with_operation(BaseOperation::FindFiles {
                    path: path.to_string(),
                    pattern: pattern.to_string(),
                });
            }
            "disk_usage" => {
                let path = intent.base_operation.parameters
                    .get("path").and_then(|v| v.as_str()).unwrap_or(".");

                plan = plan.with_operation(BaseOperation::DiskUsage {
                    path: path.to_string(),
                });
            }
            _ => return Err("不支持的基础操作".to_string()),
        }

        // 添加修饰操作
        for modifier in intent.modifiers {
            match modifier.op_type.as_str() {
                "sort" => {
                    let field = modifier.parameters.get("field")
                        .and_then(|v| v.as_str())
                        .map(parse_field)
                        .unwrap_or(Field::Default);

                    let direction = modifier.parameters.get("direction")
                        .and_then(|v| v.as_str())
                        .map(parse_direction)
                        .unwrap_or(Direction::Descending);

                    plan = plan.with_operation(BaseOperation::SortFiles {
                        field,
                        direction,
                    });
                }
                "limit" => {
                    let count = modifier.parameters.get("count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(10) as usize;

                    plan = plan.with_operation(BaseOperation::LimitFiles {
                        count,
                    });
                }
                _ => {}
            }
        }

        Ok(plan)
    }
}

fn parse_field(s: &str) -> Field {
    match s {
        "size" => Field::Size,
        "time" => Field::Time,
        "name" => Field::Name,
        "default" => Field::Default,
        _ => Field::Default,
    }
}

fn parse_direction(s: &str) -> Direction {
    match s {
        "ascending" => Direction::Ascending,
        "descending" => Direction::Descending,
        _ => Direction::Descending,
    }
}
```

---

### Step 4: 集成到 Agent 流程

**修改 `src/agent.rs`**:

```rust
// 在 handle() 方法中添加 LLM 驱动路径

// 1. 优先尝试 LLM 驱动
if self.use_llm_intent {
    match self.llm_bridge.understand_and_generate(input).await {
        Ok(plan) => {
            let command = plan.to_shell_command();
            println!("🤖 LLM 生成: {}", command);
            return Ok(command);
        }
        Err(e) => {
            println!("⚠️ LLM 失败: {}, 降级到规则匹配", e);
        }
    }
}

// 2. Fallback: 规则匹配（现有流程）
let matches = self.matcher.match_intent(input);
// ...
```

**配置开关**:
```yaml
# realconsole.yaml
llm:
  intent_generation:
    enabled: true              # 是否启用 LLM 驱动
    fallback_to_rules: true    # 失败时降级到规则匹配
```

---

### Step 5: 安全验证

**目标**: 确保 LLM 生成的命令安全

**验证规则**:
1. **操作白名单**: 只允许预定义的基础操作
2. **参数验证**: 路径不能包含 `..`，不能是根目录
3. **命令长度限制**: 生成的命令不能超过 1000 字符
4. **黑名单检查**: 不能包含 `rm -rf /`, `:(){ :|:& };:` 等危险命令

```rust
impl ExecutionPlan {
    pub fn validate_safety(&self) -> Result<(), String> {
        for op in &self.operations {
            match op {
                BaseOperation::FindFiles { path, pattern } => {
                    if path.contains("..") {
                        return Err("路径包含非法字符 ..".to_string());
                    }
                    if path == "/" {
                        return Err("不允许搜索根目录".to_string());
                    }
                }
                BaseOperation::DiskUsage { path } => {
                    if path.contains("..") {
                        return Err("路径包含非法字符 ..".to_string());
                    }
                }
                _ => {}
            }
        }

        let command = self.to_shell_command();
        if command.len() > 1000 {
            return Err("生成的命令过长".to_string());
        }

        Ok(())
    }
}
```

---

### Step 6: 测试场景

**基础场景**:
1. ✅ "显示当前目录下体积最小的rs文件" → ascending
2. ✅ "查找最近修改的py文件" → time + descending
3. ✅ "检查src目录磁盘使用" → disk_usage

**复杂场景**:
4. "显示体积在100KB到1MB之间的文件"（需要 filter）
5. "查找倒数第三大的文件"（需要反向 + offset）
6. "统计每种文件类型的数量"（需要 group_by）

**边界场景**:
7. 恶意输入："删除所有文件"
8. 无效输入："帮我写个Python程序"
9. 模糊输入："找点东西"

---

## 🎯 交付标准

### 代码
- [  ] `src/dsl/intent/llm_bridge.rs` - LLM 桥接模块
- [  ] `src/dsl/intent/mod.rs` - 导出 LlmToPipeline
- [  ] `src/agent.rs` - 集成 LLM 驱动流程
- [  ] `realconsole.yaml` - 配置开关

### 测试
- [  ] 单元测试：JSON 解析
- [  ] 单元测试：ExecutionPlan 转换
- [  ] 集成测试：LLM → ExecutionPlan → Shell
- [  ] 真实场景测试（6个基础场景）

### 文档
- [  ] Phase 7 实施计划（本文档）
- [  ] LLM Prompt 设计文档
- [  ] Phase 7 完成总结

---

## 📊 预期效果

**Before (Phase 6.3)**:
```
» 显示当前目录下体积最小的rs文件
❌ Intent: find_files_by_size
→ 错误：找到最大的文件（规则无法理解"最小"）
```

**After (Phase 7)**:
```
» 显示当前目录下体积最小的rs文件
✅ LLM 理解: 按 size ascending
→ 执行: find . -name '*.rs' -type f -exec ls -lh {} + | sort -k5 -h | head -n 1
→ 结果: 正确显示最小的文件
```

---

## 🚀 实施顺序

1. **创建数据结构** (30分钟)
   - LlmIntent, BaseOpJson, ModifierJson
   - 解析器和转换器

2. **设计 Prompt** (30分钟)
   - System prompt
   - 示例 few-shot

3. **实现 llm_bridge.rs** (1小时)
   - LlmToPipeline 结构
   - understand_and_generate 方法
   - to_execution_plan 转换

4. **集成到 Agent** (30分钟)
   - 添加配置开关
   - 实现 fallback 逻辑

5. **测试和调优** (30分钟)
   - 真实场景测试
   - Prompt 调优

---

**开始时间**: 2025-10-16
**预计完成**: 2025-10-16
**负责人**: Claude Code

---

**核心理念**:
> LLM 不是"事后检查器"，而是"核心理解器"。
> 让 LLM 参与意图理解，生成结构化计划，而非直接生成 Shell 命令。
> 结构化输出 + 安全验证 = 可控的智能生成。✨
