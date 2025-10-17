# 智能参数绑定机制设计

**日期**: 2025-10-15
**版本**: v0.5.2 计划
**问题来源**: 用户反馈 - Intent 参数提取刚性导致错误结果

---

## 🎯 问题描述

### 问题案例

**用户输入**: "查看子目录 doc"
**期望行为**: `ls -lh doc` 或 `ls -lh docs`
**实际行为**: `ls -lh .` (使用了默认值)

**根本原因**: 实体提取器 (EntityExtractor) 的路径正则表达式过于严格：
```rust
path_pattern: Regex::new(r"(\./[^\s]+|/[^\s]+|\.)").unwrap()
```

此正则仅匹配：
- `./something` (以 `./` 开头)
- `/something` (以 `/` 开头)
- `.` (当前目录)

**不匹配**: 简单的目录名如 "doc"、"src"、"tests" 等

---

## 📋 用户需求

用户明确提出两个改进方向：

1. **"匹配方面引入科学的评判，从而更加精准"**
   - 当前正则匹配过于刚性
   - 需要更智能的参数识别机制

2. **"对于匹配后的结果也要做事后的（可以考虑基于大模型的）评估，评估其匹配的合理性"**
   - 需要 LLM 参与验证
   - 确保生成的命令合理性

---

## 🎨 设计方案

### 架构概览

采用**三层渐进式参数绑定**机制：

```
用户输入 → Intent 匹配 → 参数提取 (3层) → 命令生成 → LLM 验证 → 执行
                          ↓
                      1. Regex 提取 (快速)
                      2. LLM 补充提取 (智能)
                      3. 事后验证 (安全)
```

---

## 🔧 技术实现

### Phase 1: 改进 Regex 提取 (快速路径)

#### 目标
- 扩展路径正则，支持简单目录名
- 保持快速响应 (无 LLM 调用)

#### 实现
```rust
// src/dsl/intent/extractor.rs

impl EntityExtractor {
    pub fn new() -> Self {
        Self {
            // ... 其他模式 ...

            // 改进的路径模式：支持简单目录名
            path_pattern: Regex::new(
                r"(?x)
                (\./[^\s]+|           # 相对路径: ./src
                 /[^\s]+|             # 绝对路径: /tmp
                 \.|                  # 当前目录: .
                 [a-zA-Z0-9_-]+/?)    # 简单目录名: doc, docs/, src
                "
            ).unwrap(),
        }
    }

    pub fn extract_path(&self, input: &str) -> Option<EntityType> {
        // 1. 尝试正则提取
        if let Some(captures) = self.path_pattern.captures(input) {
            if let Some(matched) = captures.get(1) {
                let path = matched.as_str().to_string();
                // 过滤掉明显不是路径的词（如命令关键字）
                if !self.is_command_keyword(&path) {
                    return Some(EntityType::Path(path));
                }
            }
        }

        // 2. 关键词检测
        if input.contains("当前目录") || input.contains("这里") {
            return Some(EntityType::Path(".".to_string()));
        }

        None
    }

    /// 检查是否为命令关键字（避免误提取）
    fn is_command_keyword(&self, word: &str) -> bool {
        matches!(
            word,
            "查看" | "显示" | "列出" | "检查" | "统计" |
            "ls" | "find" | "grep" | "check"
        )
    }
}
```

**改进点**:
- 新增 `[a-zA-Z0-9_-]+/?` 匹配简单目录名
- 添加关键字过滤，避免误提取命令词
- 支持带斜杠的目录名 (如 `docs/`)

---

### Phase 2: LLM 智能补充提取

#### 目标
- 当 Regex 提取失败时，使用 LLM 智能提取
- 理解上下文语义 (如 "子目录 doc" → "doc")

#### 实现

##### 2.1 新增 LLM 提取接口

```rust
// src/dsl/intent/extractor.rs

use crate::agent::Agent;

impl EntityExtractor {
    /// 使用 LLM 智能提取实体
    ///
    /// 当正则提取失败时的 fallback 机制
    pub async fn extract_with_llm(
        &self,
        input: &str,
        expected: &HashMap<String, EntityType>,
        agent: &Agent,
    ) -> HashMap<String, EntityType> {
        let mut extracted = HashMap::new();

        // 先尝试正则提取
        let regex_extracted = self.extract(input, expected);

        // 检查是否有缺失的实体
        let missing_entities: Vec<_> = expected
            .keys()
            .filter(|k| !regex_extracted.contains_key(*k))
            .collect();

        if missing_entities.is_empty() {
            return regex_extracted; // 正则已提取完整，无需 LLM
        }

        // 构造 LLM 提取 prompt
        let prompt = self.build_extraction_prompt(input, &missing_entities, expected);

        // 调用 LLM
        match agent.ask_llm(&prompt).await {
            Ok(response) => {
                // 解析 LLM 返回的 JSON
                if let Ok(llm_entities) = self.parse_llm_response(&response, expected) {
                    extracted.extend(llm_entities);
                }
            }
            Err(e) => {
                eprintln!("⚠ LLM 提取失败: {}", e);
            }
        }

        // 合并正则和 LLM 提取结果
        extracted.extend(regex_extracted);
        extracted
    }

    fn build_extraction_prompt(
        &self,
        input: &str,
        missing: &[&String],
        expected: &HashMap<String, EntityType>,
    ) -> String {
        let entity_descriptions: Vec<String> = missing
            .iter()
            .map(|name| {
                let entity_type = expected.get(*name).unwrap();
                format!("- {}: {}", name, self.describe_entity_type(entity_type))
            })
            .collect();

        format!(
            r#"从以下用户输入中提取指定的参数：

用户输入: "{}"

需要提取的参数:
{}

请以 JSON 格式返回提取结果，格式为:
{{
  "param_name": "value"
}}

如果无法提取，请返回 {{}}"#,
            input,
            entity_descriptions.join("\n")
        )
    }

    fn describe_entity_type(&self, entity_type: &EntityType) -> &str {
        match entity_type {
            EntityType::Path(_) => "文件路径或目录名",
            EntityType::FileType(_) => "文件类型 (如 py, rs, js)",
            EntityType::Number(_) => "数字",
            EntityType::Operation(_) => "操作名称",
            EntityType::Date(_) => "日期或时间",
            EntityType::Custom(name, _) => name,
        }
    }

    fn parse_llm_response(
        &self,
        response: &str,
        expected: &HashMap<String, EntityType>,
    ) -> Result<HashMap<String, EntityType>, String> {
        // 提取 JSON 块
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            return Err("未找到 JSON 响应".to_string());
        };

        // 解析 JSON
        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON 解析失败: {}", e))?;

        let mut entities = HashMap::new();

        // 将 JSON 转换为 EntityType
        for (name, entity_type) in expected {
            if let Some(value) = parsed.get(name) {
                if let Some(value_str) = value.as_str() {
                    let entity = match entity_type {
                        EntityType::Path(_) => EntityType::Path(value_str.to_string()),
                        EntityType::FileType(_) => EntityType::FileType(value_str.to_string()),
                        EntityType::Operation(_) => EntityType::Operation(value_str.to_string()),
                        EntityType::Date(_) => EntityType::Date(value_str.to_string()),
                        EntityType::Number(_) => {
                            if let Ok(n) = value_str.parse::<f64>() {
                                EntityType::Number(n)
                            } else {
                                continue;
                            }
                        }
                        EntityType::Custom(t, _) => EntityType::Custom(t.clone(), value_str.to_string()),
                    };
                    entities.insert(name.clone(), entity);
                }
            }
        }

        Ok(entities)
    }
}
```

##### 2.2 更新 IntentMatcher

```rust
// src/dsl/intent/matcher.rs

impl IntentMatcher {
    /// 匹配意图并智能提取实体
    pub async fn match_with_extraction(
        &self,
        input: &str,
        agent: Option<&Agent>,
    ) -> Option<IntentMatch> {
        // 1. 先用现有逻辑匹配意图
        let matches = self.match_intent(input);

        if matches.is_empty() {
            return None;
        }

        let mut best_match = matches[0].clone();

        // 2. 如果有缺失实体且提供了 agent，使用 LLM 补充提取
        if let Some(agent) = agent {
            let expected_count = best_match.intent.entities.len();
            let extracted_count = best_match.extracted_entities.len();

            if extracted_count < expected_count {
                // 有缺失实体，使用 LLM 补充
                let llm_entities = self.extractor
                    .extract_with_llm(input, &best_match.intent.entities, agent)
                    .await;

                best_match.extracted_entities.extend(llm_entities);
            }
        }

        Some(best_match)
    }
}
```

---

### Phase 3: LLM 事后验证

#### 目标
- 在命令生成后，使用 LLM 验证命令的合理性
- 检测明显错误 (如路径不存在、参数不合理等)

#### 实现

```rust
// src/dsl/intent/validator.rs (新文件)

use crate::agent::Agent;
use crate::dsl::intent::ExecutionPlan;

pub struct CommandValidator;

impl CommandValidator {
    /// 使用 LLM 验证生成的命令
    pub async fn validate(
        &self,
        user_input: &str,
        plan: &ExecutionPlan,
        agent: &Agent,
    ) -> Result<ValidationResult, String> {
        let prompt = format!(
            r#"请评估以下命令生成是否合理：

用户意图: "{}"
生成的命令: {}

请从以下角度评估:
1. 命令是否正确理解了用户意图？
2. 参数是否合理（如路径是否存在、文件类型是否匹配等）？
3. 是否存在明显错误？

请以 JSON 格式返回评估结果:
{{
  "is_valid": true/false,
  "confidence": 0.0-1.0,
  "reason": "评估理由",
  "suggestions": ["改进建议1", "改进建议2"]
}}
"#,
            user_input,
            plan.commands.join(" && ")
        );

        let response = agent.ask_llm(&prompt).await?;
        self.parse_validation_response(&response)
    }

    fn parse_validation_response(&self, response: &str) -> Result<ValidationResult, String> {
        // 提取 JSON
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            return Err("未找到 JSON 响应".to_string());
        };

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON 解析失败: {}", e))?;

        Ok(ValidationResult {
            is_valid: parsed["is_valid"].as_bool().unwrap_or(true),
            confidence: parsed["confidence"].as_f64().unwrap_or(1.0),
            reason: parsed["reason"].as_str().unwrap_or("").to_string(),
            suggestions: parsed["suggestions"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub confidence: f64,
    pub reason: String,
    pub suggestions: Vec<String>,
}
```

##### 3.2 集成到 Agent

```rust
// src/agent.rs

impl Agent {
    pub async fn execute_with_validation(&mut self, input: &str) -> Result<String, String> {
        // 1. Intent 匹配 + LLM 提取
        let intent_match = self.intent_matcher
            .match_with_extraction(input, Some(self))
            .await;

        if let Some(intent_match) = intent_match {
            // 2. 生成命令
            let plan = self.template_engine
                .generate_from_intent(&intent_match)?;

            // 3. LLM 验证
            let validator = CommandValidator;
            let validation = validator.validate(input, &plan, self).await?;

            if !validation.is_valid || validation.confidence < 0.7 {
                // 验证失败，警告用户
                eprintln!("⚠ 命令验证警告:");
                eprintln!("  置信度: {:.2}", validation.confidence);
                eprintln!("  原因: {}", validation.reason);

                if !validation.suggestions.is_empty() {
                    eprintln!("  建议:");
                    for suggestion in &validation.suggestions {
                        eprintln!("    - {}", suggestion);
                    }
                }

                // 询问用户是否继续
                print!("是否继续执行? [y/N]: ");
                // ... 用户确认逻辑 ...
            }

            // 4. 执行命令
            self.execute_plan(&plan).await
        } else {
            // Fallback to normal LLM
            self.ask_llm(input).await
        }
    }
}
```

---

## 📊 性能优化

### 策略

1. **快速路径优先** (无 LLM 调用)
   - 如果 Regex 提取成功 → 直接使用
   - 跳过 LLM 提取步骤

2. **按需验证** (可配置)
   - 配置选项: `intent.llm_validation: bool`
   - 默认 `false` (高性能模式)
   - 仅在用户启用时才使用 LLM 验证

3. **缓存机制**
   - 缓存常见输入的提取结果
   - TTL: 1 小时

```yaml
# realconsole.yaml
intent:
  llm_extraction: true    # 启用 LLM 智能提取
  llm_validation: false   # 是否启用命令验证 (性能影响)
  validation_threshold: 0.7  # 验证置信度阈值
```

---

## 🎯 实施计划

### Phase 1: 改进 Regex (高优先级)
- [ ] 更新 `path_pattern` 支持简单目录名
- [ ] 添加关键字过滤逻辑
- [ ] 更新单元测试
- [ ] 验证 "查看子目录 doc" 案例

**预计时间**: 30 分钟
**预期效果**: 解决 80% 的简单路径提取问题

---

### Phase 2: LLM 智能提取 (中优先级)
- [ ] 实现 `extract_with_llm()` 方法
- [ ] 更新 `IntentMatcher` 集成 LLM 提取
- [ ] 添加配置选项
- [ ] 编写集成测试

**预计时间**: 1-2 小时
**预期效果**: 解决复杂语义理解问题

---

### Phase 3: LLM 事后验证 (低优先级)
- [ ] 实现 `CommandValidator`
- [ ] 集成到 Agent 执行流程
- [ ] 添加用户确认交互
- [ ] 性能测试和优化

**预计时间**: 1-2 小时
**预期效果**: 提供安全网，防止明显错误

---

## 🧪 测试用例

### Regex 改进测试

```rust
#[test]
fn test_extract_simple_directory_name() {
    let extractor = EntityExtractor::new();

    // 简单目录名
    if let Some(EntityType::Path(path)) = extractor.extract_path("查看子目录 doc") {
        assert_eq!(path, "doc");
    } else {
        panic!("Expected to extract 'doc'");
    }

    // 带斜杠的目录名
    if let Some(EntityType::Path(path)) = extractor.extract_path("列出 docs/ 的内容") {
        assert_eq!(path, "docs/");
    }
}

#[test]
fn test_filter_command_keywords() {
    let extractor = EntityExtractor::new();

    // 不应该提取命令关键字
    let result = extractor.extract_path("查看当前目录");
    // 应该返回 "." 而不是 "查看"
    if let Some(EntityType::Path(path)) = result {
        assert_eq!(path, ".");
    }
}
```

### LLM 提取测试

```rust
#[tokio::test]
async fn test_llm_extraction_fallback() {
    let agent = create_test_agent();
    let extractor = EntityExtractor::new();

    let mut expected = HashMap::new();
    expected.insert("path".to_string(), EntityType::Path(String::new()));

    let input = "查看子目录 doc 的内容";
    let entities = extractor.extract_with_llm(input, &expected, &agent).await;

    assert!(entities.contains_key("path"));
    if let Some(EntityType::Path(path)) = entities.get("path") {
        assert!(path == "doc" || path == "docs");
    }
}
```

---

## 📝 文档更新

需要更新以下文档：

1. **docs/design/INTENT_DSL_DESIGN.md**
   - 添加 "智能参数绑定" 章节
   - 说明三层提取机制

2. **README.md**
   - 功能特性添加 "LLM 增强的参数提取"

3. **CHANGELOG.md**
   - v0.5.2: 智能参数绑定机制

---

## 🎉 预期成果

### 解决的问题

1. ✅ **路径提取刚性** → 支持简单目录名
2. ✅ **语义理解不足** → LLM 智能补充
3. ✅ **错误结果离谱** → LLM 事后验证

### 性能指标

- **快速路径 (Regex only)**: < 1ms
- **LLM 补充提取**: 100-500ms (按需)
- **LLM 验证**: 200-800ms (可选)

### 用户体验

**Before**:
```
» 查看子目录 doc
✨ Intent: list_directory (置信度: 1.00)
→ 执行: ls -lh .  ❌ 错误
```

**After (Phase 1)**:
```
» 查看子目录 doc
✨ Intent: list_directory (置信度: 1.00)
→ 执行: ls -lh doc  ✅ 正确
```

**After (Phase 2+3)**:
```
» 查看子目录 documentation
✨ Intent: list_directory (置信度: 1.00)
🤖 使用 LLM 提取参数...
→ 执行: ls -lh documentation  ✅ 智能
✓ 命令验证通过 (置信度: 0.95)
```

---

**设计完成日期**: 2025-10-15
**设计人**: Claude Code + User
**版本**: v0.5.2 计划
