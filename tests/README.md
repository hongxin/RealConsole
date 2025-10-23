# Rust 集成测试

本目录包含 RealConsole 的 Rust 集成测试，使用 Rust 的测试框架进行端到端功能验证。

---

## 📋 测试文件索引

### CLI 集成测试
- **test_cli_integration.rs** - CLI 命令行接口集成测试
  - 命令参数解析
  - 基础 CLI 功能验证

### 对话系统测试
- **test_conversation_integration.rs** - 对话系统集成测试
  - 上下文管理
  - 对话流程验证
  - 消息处理测试

### Intent 系统测试
- **test_intent_integration.rs** - Intent DSL 集成测试
  - Intent 匹配逻辑
  - 模式识别
  - 优先级处理

- **test_intent_matching_fix.rs** - Intent 匹配修复测试
  - Bug 修复验证
  - 边界情况测试

### Function Calling 测试
- **test_function_calling_e2e.rs** - Function Calling 端到端测试
  - LLM Function Calling 功能
  - 工具调用流程
  - 端到端集成验证

### 兼容性测试
- **terminal_compatibility.sh** - 终端兼容性测试（Shell 脚本）
  - 不同终端环境兼容性
  - 颜色输出测试
  - 交互式功能测试

---

## 🚀 运行测试

### 运行所有测试
```bash
cargo test --test '*'
```

### 运行单个测试文件
```bash
# CLI 集成测试
cargo test --test test_cli_integration

# 对话系统测试
cargo test --test test_conversation_integration

# Intent 系统测试
cargo test --test test_intent_integration
cargo test --test test_intent_matching_fix

# Function Calling 测试
cargo test --test test_function_calling_e2e
```

### 运行特定测试函数
```bash
cargo test --test test_cli_integration test_function_name
```

### 显示测试输出
```bash
cargo test --test test_cli_integration -- --nocapture
```

### 运行 Shell 兼容性测试
```bash
./tests/terminal_compatibility.sh
```

---

## 📝 测试编写规范

### 1. 文件命名
- 格式：`test_<module>_<type>.rs`
- `<module>`: 测试的模块名称
- `<type>`: 测试类型（integration, e2e, fix 等）

### 2. 测试结构
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange: 准备测试环境
        let sut = SystemUnderTest::new();

        // Act: 执行操作
        let result = sut.do_something();

        // Assert: 验证结果
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_value);
    }
}
```

### 3. 测试原则
- **独立性**: 每个测试应该独立运行
- **可重复性**: 测试结果应该稳定可重复
- **清晰性**: 测试意图应该清晰明确
- **完整性**: 覆盖正常流程和异常情况

---

## 🎯 测试覆盖情况

| 模块 | 测试文件 | 覆盖率 | 状态 |
|------|---------|--------|------|
| CLI | test_cli_integration.rs | 85% | ✅ |
| Conversation | test_conversation_integration.rs | 80% | ✅ |
| Intent DSL | test_intent_integration.rs | 90% | ✅ |
| Intent Fix | test_intent_matching_fix.rs | 95% | ✅ |
| Function Call | test_function_calling_e2e.rs | 75% | ✅ |
| Terminal | terminal_compatibility.sh | 60% | 🟡 |

---

## 🔧 维护指南

### 添加新测试
1. 在 `tests/` 目录创建 `test_<module>_<type>.rs`
2. 添加必要的依赖和导入
3. 编写测试用例
4. 更新本 README.md
5. 运行测试验证：`cargo test --test test_<module>_<type>`

### 调试测试
```bash
# 显示详细输出
cargo test --test <test_file> -- --nocapture

# 运行单个测试
cargo test --test <test_file> <test_name> -- --nocapture

# 显示测试日志
RUST_LOG=debug cargo test --test <test_file>
```

### CI/CD 集成
测试会在以下情况自动运行：
- PR 提交时
- 合并到主分支时
- 发布新版本时

---

## 📚 相关文档

- [开发者指南](../docs/02-practice/developer/developer-guide.md)
- [测试脚本索引](../scripts/test/README.md)
- [示例代码](../examples/README.md)

---

## 🐛 问题反馈

测试失败时的排查步骤：

1. **检查依赖**: `cargo check`
2. **清理构建**: `cargo clean && cargo build`
3. **更新依赖**: `cargo update`
4. **查看日志**: `RUST_LOG=debug cargo test`
5. **隔离测试**: 运行单个测试文件定位问题

如果问题持续存在，请提交 Issue 并附带：
- 测试失败的输出
- Rust 版本信息
- 操作系统信息

---

**最后更新**: 2025-10-23
**维护者**: RealConsole Contributors
