# Phase 5 设计计划：增强工具系统

**开始日期**: 2025-01-15
**目标**: 扩展工具生态，实现工具链编排
**状态**: ✅ Phase 5.1-5.2 已完成 (2025-10-15) | ⏳ Phase 5.3-5.4 计划中

---

## 📌 Phase 5.1 完成总结

- ✅ **9 个高级工具已实现**: HTTP (2) + JSON (2) + 文本 (3) + 系统 (2)
- ✅ **版本升级**: v0.1.0 → v0.5.0
- ✅ **工具总数**: 5 → 14 (+180%)
- ✅ **测试覆盖**: 新增 10 个单元测试，全部通过
- ✅ **集成验证**: 工具注册、命令更新、端到端测试完成

详见: [Phase 5.1 实施总结](PHASE5_IMPLEMENTATION.md)

## 📌 Phase 5.2 完成总结 (2025-10-15)

- ✅ **并行工具执行**: 使用 `futures::join_all` 实现并发调度
- ✅ **执行统计**: 添加 `duration_ms` 字段，记录每个工具的执行时间
- ✅ **执行模式**: 支持 `Parallel` 和 `Sequential` 两种模式
- ✅ **API 扩展**: `with_execution_mode()` 和 `execution_mode()` 方法
- ✅ **测试覆盖**: 新增 4 个单元测试，所有测试通过 (7/7)

详见: [Phase 5.2 实施总结](PHASE5.2_IMPLEMENTATION.md)

---

## 🎯 总体目标

在 Phase 4 的基础上，将工具系统从 5 个基础工具扩展到 15+ 个实用工具，并实现工具链编排能力，使 Agent 能够处理更复杂的任务。

### 设计原则
1. **实用优先** - 选择最常用的工具类型
2. **安全第一** - 每个工具都有严格的安全检查
3. **易扩展** - 保持简单的工具注册机制
4. **高性能** - 支持并行执行和结果缓存

---

## 📦 Phase 5.1: 新增实用工具（10 个）

### 1. HTTP 工具组（2 个）

#### `http_get` - HTTP GET 请求
```rust
// 功能：发送 HTTP GET 请求
// 参数：
//   - url: String (必需) - 目标 URL
//   - headers: Object (可选) - HTTP 头
//   - timeout: Number (可选) - 超时时间（秒），默认 30
// 返回：响应内容（字符串）
// 安全限制：
//   - 只允许 http/https 协议
//   - 超时限制（最多 60 秒）
//   - 响应大小限制（最多 10MB）
```

**使用场景**:
- 获取 API 数据
- 检查网站状态
- 下载文本内容

#### `http_post` - HTTP POST 请求
```rust
// 功能：发送 HTTP POST 请求
// 参数：
//   - url: String (必需) - 目标 URL
//   - body: String (必需) - 请求体
//   - headers: Object (可选) - HTTP 头
//   - timeout: Number (可选) - 超时时间（秒），默认 30
// 返回：响应内容（字符串）
// 安全限制：同 http_get
```

---

### 2. JSON 工具组（2 个）

#### `json_parse` - 解析 JSON
```rust
// 功能：解析 JSON 字符串并格式化
// 参数：
//   - json_str: String (必需) - JSON 字符串
//   - pretty: Boolean (可选) - 是否美化输出，默认 true
// 返回：格式化的 JSON 或错误信息
// 安全限制：
//   - JSON 大小限制（最多 1MB）
//   - 嵌套深度限制（最多 20 层）
```

**使用场景**:
- 验证 JSON 格式
- 美化 JSON 输出
- JSON 数据检查

#### `json_query` - 查询 JSON
```rust
// 功能：使用 JSONPath 查询 JSON 数据
// 参数：
//   - json_str: String (必需) - JSON 字符串
//   - path: String (必需) - JSONPath 表达式，如 "$.users[0].name"
// 返回：查询结果
// 安全限制：同 json_parse
```

**使用场景**:
- 提取 JSON 特定字段
- 过滤 JSON 数据
- JSON 数据转换

---

### 3. 文本处理工具组（3 个）

#### `text_search` - 文本搜索
```rust
// 功能：在文本中搜索匹配的行
// 参数：
//   - text: String (必需) - 要搜索的文本
//   - pattern: String (必需) - 搜索模式（支持正则）
//   - case_sensitive: Boolean (可选) - 是否区分大小写，默认 false
// 返回：匹配的行列表
```

#### `text_replace` - 文本替换
```rust
// 功能：替换文本中的内容
// 参数：
//   - text: String (必需) - 原始文本
//   - pattern: String (必需) - 要替换的模式
//   - replacement: String (必需) - 替换内容
//   - all: Boolean (可选) - 是否替换所有匹配，默认 true
// 返回：替换后的文本
```

#### `text_split` - 文本分割
```rust
// 功能：按分隔符分割文本
// 参数：
//   - text: String (必需) - 原始文本
//   - delimiter: String (必需) - 分隔符
//   - max_split: Number (可选) - 最大分割次数，默认无限
// 返回：分割后的数组
```

---

### 4. 系统信息工具组（2 个）

#### `get_env` - 获取环境变量
```rust
// 功能：获取环境变量值
// 参数：
//   - name: String (必需) - 环境变量名
//   - default: String (可选) - 默认值
// 返回：环境变量值或默认值
// 安全限制：
//   - 禁止读取敏感环境变量（API_KEY, PASSWORD 等）
```

#### `get_system_info` - 获取系统信息
```rust
// 功能：获取系统信息
// 参数：
//   - info_type: String (必需) - 信息类型：os, arch, hostname, cpu_count
// 返回：请求的系统信息
```

**使用场景**:
- 检查运行环境
- 条件执行
- 调试信息

---

### 5. 文件操作增强（1 个）

#### `find_files` - 文件搜索
```rust
// 功能：在目录中搜索文件
// 参数：
//   - directory: String (必需) - 搜索目录
//   - pattern: String (必需) - 文件名模式（支持通配符）
//   - recursive: Boolean (可选) - 是否递归搜索，默认 false
//   - max_results: Number (可选) - 最大结果数，默认 100
// 返回：匹配的文件路径列表
// 安全限制：
//   - 禁止搜索系统目录
//   - 结果数量限制（最多 1000）
```

---

## 🔗 Phase 5.2: 工具链编排

### 1. 工具管道（Pipeline）

**概念**: 将多个工具的输出串联起来

**示例**:
```
用户: "从 https://api.github.com/repos/rust-lang/rust 获取数据，
      提取其中的 stargazers_count 字段"

执行链:
1. http_get(url="https://api.github.com/repos/rust-lang/rust")
   → 返回 JSON 字符串
2. json_query(json_str=<步骤1结果>, path="$.stargazers_count")
   → 返回 star 数量
```

**实现要点**:
- ToolExecutor 已支持迭代执行
- 需要改进结果传递机制
- 添加中间结果缓存

---

### 2. 并行工具执行

**概念**: 同时执行多个独立的工具调用

**示例**:
```
用户: "同时获取 Rust 和 Go 的 GitHub star 数"

并行执行:
1. http_get(url="https://api.github.com/repos/rust-lang/rust")
2. http_get(url="https://api.github.com/repos/golang/go")
→ 等待两个请求完成
→ 聚合结果
```

**实现要点**:
- 使用 `tokio::join!` 并行执行
- 最多并行 3 个工具（已有配置 max_tools_per_round）
- 错误处理：部分失败不影响其他工具

---

### 3. 工具依赖图

**概念**: 自动检测工具间的依赖关系

**示例**:
```
任务: "读取 data.json，解析后提取 name 字段"

依赖图:
read_file("data.json")
    ↓
json_query(json_str=<上一步>, path="$.name")
```

**实现**:
- 分析工具参数引用
- 构建 DAG（有向无环图）
- 拓扑排序执行

---

## 📊 Phase 5.3: 工具系统增强

### 1. 工具配置

```yaml
# realconsole.yaml
tools:
  # 工具开关
  enabled_tools:
    - calculator
    - read_file
    - write_file
    - http_get
    - json_parse

  # 工具限制
  http:
    max_timeout: 60
    max_response_size: 10485760  # 10MB
    allowed_domains:
      - "*.github.com"
      - "api.example.com"

  file:
    allowed_directories:
      - "/home/user/projects"
      - "/tmp"
    max_file_size: 1048576  # 1MB
```

---

### 2. 工具使用统计

**功能**: 记录每个工具的使用情况

**统计指标**:
- 调用次数
- 成功率
- 平均执行时间
- 最近使用时间

**查看方式**:
```bash
> /tools stats
工具使用统计:
  calculator - 调用 45 次，成功率 100%，平均 12ms
  read_file - 调用 23 次，成功率 95.7%，平均 34ms
  http_get - 调用 12 次，成功率 83.3%，平均 456ms
```

---

### 3. 工具权限控制

**目的**: 细粒度控制工具的使用权限

**实现**:
```rust
pub struct ToolPermission {
    pub tool_name: String,
    pub allowed: bool,
    pub rate_limit: Option<RateLimit>,  // 速率限制
    pub require_confirmation: bool,      // 是否需要确认
}

// 危险操作需要确认
write_file → 需要用户确认
http_post → 需要用户确认
```

---

## 🏗️ 实现路线图

### 阶段 1: 基础工具实现（第 1-3 天） ✅ 已完成
- ✅ 设计 Phase 5 计划
- ✅ 实现 HTTP 工具组（http_get, http_post）
- ✅ 实现 JSON 工具组（json_parse, json_query）
- ✅ 实现文本处理工具组（text_search, text_replace, text_split）

### 阶段 2: 系统工具实现（第 4 天） ✅ 已完成
- ✅ 实现系统信息工具组（get_env, get_system_info）
- ⏳ 实现文件搜索工具（find_files） - 可选，Phase 5.2
- ✅ 编写所有新工具的单元测试（10 个测试）

### 阶段 3: 工具链编排（第 5-6 天） ✅ Phase 5.2 已完成
- ✅ 改进 ToolExecutor 结果传递（添加 duration_ms 统计）
- ✅ 实现并行工具执行（ExecutionMode::Parallel）
- ⏳ 实现工具依赖分析（DAG） - 留待 Phase 5.3+

### 阶段 4: 系统增强（第 7 天）
- ⏳ 实现工具配置系统
- ⏳ 实现工具使用统计
- ⏳ 实现工具权限控制（可选）

### 阶段 5: 测试与文档（第 8 天）
- ⏳ 端到端测试
- ⏳ 性能测试
- ⏳ 编写用户文档
- ⏳ 更新开发者文档

---

## 📊 目标指标

| 指标 | 当前 (Phase 4) | 目标 (Phase 5) | 提升 |
|-----|---------------|---------------|------|
| **内置工具数** | 5 | 15+ | +200% |
| **工具分类** | 3 类 | 6 类 | +100% |
| **并行执行** | ❌ | ✅ | NEW |
| **工具配置** | ❌ | ✅ | NEW |
| **使用统计** | ❌ | ✅ | NEW |
| **测试覆盖** | 37 tests | 80+ tests | +116% |

---

## 🎯 成功标准

### 必须完成（P0）
- ✅ 至少新增 8 个实用工具
- ✅ HTTP、JSON、文本处理三大类工具完整
- ✅ 所有新工具有安全限制
- ✅ 所有新工具有单元测试
- ✅ 工具链串行执行优化

### 应该完成（P1）
- ⏳ 新增 10 个工具
- ⏳ 并行工具执行
- ⏳ 工具配置系统
- ⏳ E2E 测试覆盖

### 可选完成（P2）
- ⏳ 工具依赖分析
- ⏳ 工具使用统计
- ⏳ 工具权限控制
- ⏳ 工具热加载

---

## 🔧 技术栈

- **HTTP 客户端**: `reqwest` (已有)
- **JSON 处理**: `serde_json` (已有)
- **正则表达式**: `regex` (已有)
- **系统信息**: `std::env` (内置)
- **并行执行**: `tokio::join!` (已有)
- **JSON 查询**: `jsonpath_lib` (新增依赖)

---

## 📚 相关文档

- [工具调用用户指南](../guides/TOOL_CALLING_USER_GUIDE.md)
- [工具调用开发者指南](../guides/TOOL_CALLING_DEVELOPER_GUIDE.md)
- [Phase 4 总结](PHASE3_SUMMARY.md)

---

## 🚀 开始实施

**第一步**: 实现 HTTP 工具组
- 从最常用的 `http_get` 开始
- 添加完整的错误处理
- 编写单元测试

**下一步**: 逐步实现其他工具组

---

**Phase 5 计划制定完成！** 准备开始实施 🎉
