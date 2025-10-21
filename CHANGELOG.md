# Changelog

All notable changes to RealConsole will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
