# 对话上下文 Phase 7 - Bug 修复

**日期**: 2025-10-21
**版本**: v1.3.6
**类型**: Bug Fix

---

## 修复内容

### 1. UTF-8 字符边界 Panic

**问题**: `/context show` 截断中文字符导致 panic

**修复**:
- 新增 `truncate_str()` 函数，按字符数（非字节数）安全截取
- 位置: `src/commands/context_cmd.rs:188-247`

### 2. 工具模式缺少上下文

**问题**: 工具调用模式虽记录对话，但从不使用历史上下文

**修复**:
- `ToolExecutor::execute_iterative`: 支持多轮消息输入
- `LlmService`: 添加 `messages` 字段和 `with_tools_and_context()` 方法
- `Agent::handle_text_with_tools`: 集成上下文检查和构建逻辑
- 位置: `src/tool_executor.rs`, `src/services/llm_service.rs`, `src/agent.rs`

### 3. 触发词列表不完整

**问题**: 缺少"现在"、"那么"、"所以"等常见承接词

**修复**: 扩充 20+ 触发词
- 回顾: 刚才、之前、上面、前面、earlier、previous、above、before
- 承接（新增）: 现在、那么、所以、因此、这样、那、now、then、so、thus、therefore
- 位置: `src/conversation/context_manager.rs:117-122`

---

## 测试

- ✓ 所有单元测试通过 (703/703)
- ✓ UTF-8 字符截断测试
- ✓ 工具模式上下文集成测试
- ✓ 触发词检测测试

---

## 影响

**用户体验**:
- ✓ 对话记忆正常工作（工具模式）
- ✓ `/context show` 不再 panic
- ✓ 更多自然对话被识别

**性能**: 无影响

---

**完成标志**: 🟢 已发布 v1.3.6
