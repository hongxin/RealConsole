# Changelog

All notable changes to RealConsole will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.6.0] - 2025-10-23

### Added

- **[Feature] 系统 Dashboard (`/trace dashboard`)**
  - 基于易经"四象"哲学的统一系统健康度视图
  - **系统健康度评分**（0-100 分）
    - 命令成功率（40% 权重）
    - LLM 响应质量（20% 权重）
    - 系统活跃度（20% 权重）
    - 异常程度（20% 权重，反向）
    - 5 级健康等级：优秀(90-100)、良好(75-89)、一般(60-74)、较差(40-59)、危险(0-39)
  - **四象分区视图**
    - ☰ 太阳（Statistics）- 命令频率、使用模式
    - ☷ 太阴（Memory）- 对话上下文、知识积累
    - ☲ 少阳（Coordination）- 执行追踪、协同流程
    - ☵ 少阴（BlackBox）- LLM 调用、智能黑盒
  - **异常检测系统**
    - 高失败率检测（>20% 触发）
    - 重复错误检测（>=3 次触发）
    - 严重程度分级（1-5）
    - 数据关联（失败率、失败次数、错误详情等）
  - **智能建议系统**
    - 基于健康度的建议
    - 基于成功率的建议
    - 基于异常的建议（高失败率、重复错误）
    - 基于活跃度的建议
    - 优先级排序（1-5）
    - 可执行命令链接

- **核心模块 `tracer/dashboard.rs`**
  - `Dashboard` - Dashboard 生成器
  - `DashboardConfig` - 可配置选项
  - `HealthScore` - 系统健康度评分
  - `Anomaly` / `AnomalyType` - 异常检测
  - `Suggestion` - 智能建议
  - 总代码量：~620 行

- **UnifiedTracer 增强**
  - 新增 `get_failed_logs()` 方法，用于异常检测和错误分析
  - 支持获取最近的失败执行日志

### Changed

- **`/trace` 命令增强**
  - 新增 `/trace dashboard` 子命令（别名：`dash`）
  - 更新帮助文本，添加 Dashboard 说明

### Philosophy

- **离卦（☲）- 向外照明**
  - Dashboard 通过可视化展示，将系统内部状态"照明"给用户
  - 健康度评分条形图、四象分区百分比、彩色状态标识、清晰的建议列表

- **坎卦（☵）- 向内深入**
  - Dashboard 通过算法分析，深入理解系统规律
  - 多维度健康评分计算、异常模式检测、智能建议生成

- **四象理论**
  - Dashboard 完美体现了"四象"的分类思想
  - 将系统观测维度抽象为四个互补视角

### Performance

- Dashboard 渲染时间：< 100ms
- 异常检测额外开销：~10-20ms
- 编译时间：~5-7s（增量编译）
- 性能影响：可忽略不计

### Documentation

- 新增 `docs/04-reports/v1.6.0-dashboard-completion.md` - Dashboard 完成报告
- 新增 `docs/04-reports/v1.6.0-optimization-report.md` - 优化完成报告

## [1.5.0] - 2025-10-23

### Added

- **[Feature] 统一追踪系统 (`/trace` 命令)**
  - 全新的四维观测体系，聚合 History、log、llm-log、Context 四个数据源
  - 提供统一的查询接口，降低用户认知负担
  - 8个子命令支持多种查询方式：
    - `/trace` - 显示最近 20 条记录（四维聚合）
    - `/trace all [n]` - 显示最近 N 条记录
    - `/trace history [n]` - 仅显示 History 维度（统计）
    - `/trace log [n]` - 仅显示 log 维度（协同）
    - `/trace llm [n]` - 仅显示 llm-log 维度（黑盒）
    - `/trace context [n]` - 仅显示 Context 维度（记忆）
    - `/trace search <关键词>` - 关键词搜索
    - `/trace stats` - 显示统计信息
  - 快捷别名支持：`trace → t`, `history → h`, `log → l`, `context → c`, `search → s`
  - 基于"四象"哲学理论的设计（详见 `docs/04-reports/four-dimensions-philosophy.md`）

- **核心模块 `tracer/`**
  - `types.rs` - 核心类型定义（Dimension, EntryType, Status）
  - `entry.rs` - 统一追踪条目（TraceEntry）
  - `unified_tracer.rs` - 统一追踪器（UnifiedTracer）
  - `benchmarks.rs` - 性能基准测试
  - 总代码量：~2000 行

- **高性能设计**
  - 并行查询四个数据源（使用 `tokio::join!`）
  - 智能去重算法（内容哈希 + 时间窗口）
  - 性能超预期 40-65 倍（详见测试报告）
  - 所有操作 < 15ms（目标 10-250ms）

### Changed

- **[Freeze] Memory 系统冻结**
  - 停止记录新的对话内容（Phase 1 of Memory 重新设计）
  - `/memory` 命令添加冻结警告横幅
  - 保留所有查询功能（status/search/clear/important 等）
  - 未来 Memory 2.0 将专注于智能上下文编排
  - 详见：`docs/04-reports/memory-system-redesign.md`

### Fixed

- **[Critical] UTF-8 字符串切片 Panic**
  - 修复 `TraceEntry::preview()` 中的不安全字符串切片
  - 添加 UTF-8 字符边界检查，避免切割多字节字符（如中文）
  - 影响文件：`src/tracer/entry.rs:207`
  - 问题：切片可能落在多字节字符内部导致 panic
  - 修复：使用 `is_char_boundary()` 安全检查

### Performance

- **性能测试结果**（实际 vs 目标）：
  - query_all(100): 0.97ms < 50ms ✅ (51倍优于目标)
  - query_by_dimension(100): 0.46ms < 30ms ✅ (65倍优于目标)
  - search: 4.29ms < 100ms ✅ (23倍优于目标)
  - deduplicate(300): 0.25ms < 10ms ✅ (40倍优于目标)
  - stats: 9.23ms < 200ms ✅ (21倍优于目标)
  - 4 parallel queries: 13.86ms < 250ms ✅ (18倍优于目标)

### Testing

- **测试覆盖**：
  - 单元测试：10个（UnifiedTracer 核心功能）
  - 基准测试：6个（性能验证）
  - 边缘测试：5个（空数据、UTF-8、大数据集、边界、特殊字符）
  - 全套测试：831 passed, 0 failed, 20 ignored
  - Clippy：tracer 模块零警告 ✅

- **边缘情况覆盖**：
  - ✅ 空数据源不崩溃
  - ✅ UTF-8 多语言支持（中文/日文/俄文/emoji）
  - ✅ 5000条大数据集处理
  - ✅ limit 边界测试（0/1/999999）
  - ✅ 特殊字符搜索（空字符串/特殊符号/Unicode）

### Documentation

- **新增文档**：
  - `docs/04-reports/trace-command-design.md` - 详细设计文档
  - `docs/04-reports/trace-implementation-plan.md` - 实施计划与进度
  - `docs/04-reports/four-dimensions-philosophy.md` - 四维哲学理论
  - `docs/04-reports/phase-5-testing-completion.md` - 测试完成报告

- **更新文档**：
  - `CHANGELOG.md` - 添加 v1.5.0 更新日志
  - 更多文档更新详见 Phase 6

### Development

- **实施时间**：2天（预计 8-12 天）
- **开发模式**：一气呵成（Phase 1-5 连续完成）
- **代码质量**：零警告、高覆盖、超性能

### Migration Notes

- Memory 系统已冻结，不再记录新内容
- 现有 Memory 数据仍可查询，不受影响
- 使用 `/trace` 命令获取完整的四维观测视图
- Memory 2.0 将在未来版本中实现智能上下文编排

---

## [1.3.7] - 2025-10-22

### Added
- **Memory 优化**
  - 添加 `/memory stats` 命令显示统计分析（类型分布、时间跨度、可视化进度条）
  - 实现记忆重要性标记功能（Normal/Important/Critical 三级）
  - 新增 `/memory mark <索引> <级别>` 命令标记重要性
  - 新增 `/memory important [级别]` 命令查看指定重要性的记忆
  - 记忆条目显示增加重要性标记符号（⭐ / ⭐⭐）
  - 相对时间展示（"刚刚"、"5分钟前"、"3小时前"等）

- **语音播报系统（v1.3.7 新特性）**
  - 创建完整的 `voice` 模块，支持跨平台 TTS（Text-to-Speech）
  - macOS: 使用 `say` 命令（支持中文语音如 Ting-Ting）
  - Linux: 支持 `espeak` 或 `festival`
  - Windows: 支持 PowerShell TTS
  - 异步语音播报队列，不阻塞主线程
  - 新增 `/voice` 命令系列：
    - `/voice on` - 启用语音播报
    - `/voice off` - 禁用语音播报
    - `/voice status` - 显示状态
    - `/voice test [文本]` - 测试语音播报
  - 配置文件支持：`voice.enabled`、`voice.voice`、`voice.max_queue_size`

### Changed
- `MemoryEntry` 增加 `importance` 字段，向后兼容（使用 `#[serde(default)]`）
- `EntryType` 添加 `Hash` trait 支持，用于统计分析
- 更新配置文件示例，添加 voice 配置说明

### Fixed
- Memory 预览功能使用 `chars()` 按字符数截断，避免 UTF-8 边界问题

### Documentation
- 更新主配置文件 `realconsole.yaml` 添加语音配置示例
- 更新 `config/minimal.yaml` 添加语音配置注释
- 完善 voice 命令帮助文档

### Testing
- 添加 memory stats 功能测试
- 添加 memory importance 功能测试
- 添加 voice 模块完整测试覆盖
- 添加 voice commands 测试（7个测试用例）
- 所有测试通过（multi-thread runtime）

---

## [1.3.6] - Previous Release

详见之前的版本记录...

---

## Future Releases

查看 [ROADMAP.md](docs/00-core/roadmap.md) 了解未来规划。
