# 测试脚本索引

本目录包含 RealConsole 的各类测试脚本，用于验证功能、性能测试和功能演示。

---

## 📋 目录结构

```
scripts/test/
├── README.md                    # 本文件
├── trace/                       # Trace 功能测试
│   ├── test_trace.sh           # 基础 trace 功能测试
│   ├── test_trace_recent.sh    # /trace recent 命令测试
│   ├── test_trace_detail.sh    # /trace detail 命令测试（v1）
│   ├── test_trace_detail_v2.sh # /trace detail 命令测试（v2）
│   ├── test_trace_detail_final.sh # /trace detail 最终版本测试
│   ├── test_trace_detail_llm.sh   # LLM 调用链测试
│   ├── test_trace_session.sh   # 跨会话 trace 测试
│   └── test_llm_trace.sh       # LLM Span 追踪测试
├── dashboard/                   # Dashboard 功能测试
│   ├── test_dashboard.sh       # 基础 Dashboard 测试
│   ├── test_dashboard_anomaly.sh # 异常检测测试
│   ├── test_repeated_errors.sh # 重复错误检测测试
│   └── test_v1.6.0_demo.sh    # v1.6.0 完整演示
├── integration/                 # 集成测试
│   ├── test_context_fix.sh     # Context 修复测试
│   ├── test_detail.sh          # 详细功能测试
│   └── test_voice_default.sh   # 语音默认配置测试
└── legacy/                      # 废弃或历史测试脚本
```

---

## 🔍 测试脚本分类

### 1. Trace 功能测试 (trace/)

**基础功能**：
- `test_trace.sh` - 测试基础 trace 命令
- `test_trace_recent.sh` - 测试 `/trace recent` 显示最近的完整追踪

**详细追踪**：
- `test_trace_detail.sh` - 第一版 detail 命令测试
- `test_trace_detail_v2.sh` - 第二版优化测试
- `test_trace_detail_final.sh` - 最终版本测试
- `test_trace_detail_llm.sh` - 测试 LLM 调用链的详细展示

**高级功能**：
- `test_trace_session.sh` - 测试跨会话的 trace 功能
- `test_llm_trace.sh` - 测试 LLM Span 和 Tool Span 追踪

### 2. Dashboard 功能测试 (dashboard/)

**v1.6.0 新增**：
- `test_dashboard.sh` - 基础 Dashboard 渲染测试
  - 系统健康度评分
  - 四象分区视图
  - 智能建议系统

- `test_dashboard_anomaly.sh` - 异常检测功能测试
  - 高失败率检测
  - 异常提示展示

- `test_repeated_errors.sh` - 重复错误检测测试
  - 重复错误识别
  - 智能建议生成

- `test_v1.6.0_demo.sh` - v1.6.0 完整功能演示
  - 正常使用 → 健康 Dashboard
  - 制造异常 → 异常检测 → 智能建议
  - 综合展示所有功能

### 3. 集成测试 (integration/)

- `test_context_fix.sh` - Context 系统修复验证
- `test_detail.sh` - 详细功能集成测试
- `test_voice_default.sh` - 语音功能默认配置测试

---

## 🚀 快速开始

### 运行单个测试

```bash
# 从项目根目录运行
cd /path/to/RealConsole

# 测试 Dashboard
./scripts/test/dashboard/test_dashboard.sh

# 测试 Trace
./scripts/test/trace/test_trace_recent.sh
```

### 运行分类测试

```bash
# 运行所有 Dashboard 测试
for test in scripts/test/dashboard/*.sh; do
    echo "Running $test..."
    $test
done

# 运行所有 Trace 测试
for test in scripts/test/trace/*.sh; do
    echo "Running $test..."
    $test
done
```

### 运行所有测试

```bash
# 创建测试运行脚本
find scripts/test -name "*.sh" -not -path "*/legacy/*" -exec {} \;
```

---

## 📝 测试脚本编写规范

### 1. 命名规范

```bash
test_<feature>_<variant>.sh
```

- `<feature>`: 测试的功能名称（如 dashboard, trace, context）
- `<variant>`: 变体或版本（可选，如 v2, final, anomaly）

### 2. 脚本结构

```bash
#!/bin/bash
# <测试描述>

echo "=== <测试名称> ==="
echo ""

{
    # 1. 准备阶段
    echo "<命令1>"
    sleep <等待时间>

    # 2. 执行阶段
    echo "<命令2>"
    sleep <等待时间>

    # 3. 验证阶段
    echo "<查询命令>"
    sleep <等待时间>

    echo "exit"
} | ./target/debug/realconsole 2>&1

echo ""
echo "=== 测试完成 ==="
```

### 3. 注意事项

- 所有脚本必须有执行权限（`chmod +x`）
- 使用相对路径调用 realconsole（支持 debug 和 release）
- 适当的 sleep 时间确保命令执行完成
- 清晰的输出分隔和描述

---

## 🎯 测试覆盖情况

### v1.6.0 测试覆盖

| 功能模块 | 测试脚本 | 覆盖率 | 状态 |
|---------|---------|--------|------|
| Dashboard 基础 | test_dashboard.sh | 100% | ✅ |
| 异常检测 | test_dashboard_anomaly.sh | 80% | ✅ |
| 重复错误 | test_repeated_errors.sh | 100% | ✅ |
| 完整演示 | test_v1.6.0_demo.sh | 100% | ✅ |
| Trace Recent | test_trace_recent.sh | 100% | ✅ |
| Trace Detail | test_trace_detail*.sh | 100% | ✅ |
| LLM Span | test_llm_trace.sh | 100% | ✅ |

### v1.5.0 测试覆盖

| 功能模块 | 测试脚本 | 覆盖率 | 状态 |
|---------|---------|--------|------|
| 基础 Trace | test_trace.sh | 100% | ✅ |
| Context | test_context_fix.sh | 80% | ✅ |
| 集成功能 | test_detail.sh | 70% | ✅ |

---

## 🔧 维护指南

### 添加新测试脚本

1. 确定测试类型，选择合适的子目录
2. 按照命名规范创建脚本文件
3. 遵循脚本结构编写测试
4. 添加执行权限：`chmod +x <script>`
5. 更新本 README.md 文档

### 废弃旧测试脚本

1. 将脚本移动到 `legacy/` 目录
2. 在本 README 的废弃列表中记录
3. 说明废弃原因和替代方案

### 测试脚本重构

- 合并功能重复的测试
- 分离复杂测试为多个子测试
- 提取公共逻辑为独立函数

---

## 📚 相关文档

- [v1.6.0 Dashboard 完成报告](../../docs/04-reports/v1.6.0-dashboard-completion.md)
- [v1.6.0 优化报告](../../docs/04-reports/v1.6.0-optimization-report.md)
- [Trace 命令设计](../../docs/04-reports/trace-command-design.md)
- [四维哲学理论](../../docs/04-reports/four-dimensions-philosophy.md)

---

## 🐛 问题反馈

如果测试脚本执行失败或发现问题：

1. 检查 realconsole 是否已编译（`cargo build` 或 `cargo build --release`）
2. 确认脚本有执行权限
3. 查看脚本输出，定位失败原因
4. 提交 Issue 或联系开发团队

---

**最后更新**: 2025-10-23
**维护者**: RealConsole Contributors
