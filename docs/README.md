# RealConsole 文档中心

**版本**: v1.1.0 "International"
**最后更新**: 2025-10-17

欢迎来到 RealConsole 文档中心！本文档遵循"极简主义"设计理念，只保留对未来开发和用户使用有价值的核心文档。

## 📖 文档导航

### 00-core - 核心理念（3个文档）

**定位**：道/核心哲学，指导整个项目的设计思想

- [philosophy.md](00-core/philosophy.md) - 一分为三哲学思想
- [vision.md](00-core/vision.md) - 产品愿景和定位
- [roadmap.md](00-core/roadmap.md) - 技术路线图

### 01-understanding - 理解态（6个文档）

**定位**：设计文档与技术分析，供开发者理解系统架构

#### 核心设计
- [overview.md](01-understanding/overview.md) - 架构总览
- [design/architecture.md](01-understanding/design/architecture.md) - 系统架构设计
- [design/error-handling.md](01-understanding/design/error-handling.md) - 错误处理系统
- [design/security.md](01-understanding/design/security.md) - 安全设计
- [design/phase10-task-system-architecture.md](01-understanding/design/phase10-task-system-architecture.md) - 任务系统架构

#### 技术分析
- [analysis/technical-debt.md](01-understanding/analysis/technical-debt.md) - 技术债务追踪

### 02-practice - 实践态（15个文档）

**定位**：实用指南和示例，供用户和开发者日常使用

#### 用户指南（8个）
- [user/quickstart.md](02-practice/user/quickstart.md) - 5分钟快速开始
- [user/user-guide.md](02-practice/user/user-guide.md) - 完整用户手册
- [user/env-config.md](02-practice/user/env-config.md) - 环境变量配置
- [user/llm-setup.md](02-practice/user/llm-setup.md) - LLM 配置指南
- [user/intent-dsl-guide.md](02-practice/user/intent-dsl-guide.md) - Intent DSL 使用指南
- [user/tool-calling-guide.md](02-practice/user/tool-calling-guide.md) - 工具调用指南
- [user/conversation-guide.md](02-practice/user/conversation-guide.md) - 多轮对话指南
- [user/history-feature-guide.md](02-practice/user/history-feature-guide.md) - 历史记录功能

#### 开发者指南（4个）
- [developer/developer-guide.md](02-practice/developer/developer-guide.md) - 开发者指南
- [developer/tool-development.md](02-practice/developer/tool-development.md) - 工具开发指南
- [developer/i18n-guide.md](02-practice/developer/i18n-guide.md) - 国际化指南
- [developer/api-reference.md](02-practice/developer/api-reference.md) - API 参考

#### 用例示例（2个）
- [use-cases/README.md](02-practice/use-cases/README.md) - 用例索引
- [use-cases/selected-cases.md](02-practice/use-cases/selected-cases.md) - 精选使用案例

### 03-evolution - 演化态（2个文档）

**定位**：版本发布信息，记录重要里程碑

- [RELEASE-v1.0.0.md](03-evolution/RELEASE-v1.0.0.md) - v1.0.0 发布说明
- [README.md](03-evolution/README.md) - 演化历程索引

### archive - 归档（230个文档）

**定位**：历史开发记录，仅供追溯参考

包含所有历史的开发日志、会话记录、旧版文档等。这些文档已不再维护，但保留以供历史追溯。

## 📊 文档统计

**活跃文档数量**：
- 00-core: 3个
- 01-understanding: 6个
- 02-practice: 15个
- 03-evolution: 2个
- **总计**: 26个核心文档（从82个精简而来）

**归档文档**：230个（不计入活跃文档）

## 🌐 多语言支持计划

RealConsole v1.1.0 开始支持多语言界面。文档多语言化计划：

1. **中文优先**：所有核心文档默认中文
2. **英文同步**：v2.0.0 大版本时同步英文版本
3. **文档命名**：英文版本使用 `.en.md` 后缀

示例：
```
philosophy.md      # 中文版（默认）
philosophy.en.md   # 英文版（计划中）
```

## 🔍 快速查找

### 新手入门
1. 阅读 [quickstart.md](02-practice/user/quickstart.md) - 5分钟上手
2. 阅读 [user-guide.md](02-practice/user/user-guide.md) - 完整功能了解
3. 参考 [selected-cases.md](02-practice/use-cases/selected-cases.md) - 实战案例

### 开发贡献
1. 阅读 [philosophy.md](00-core/philosophy.md) - 理解设计哲学
2. 阅读 [architecture.md](01-understanding/design/architecture.md) - 了解系统架构
3. 阅读 [developer-guide.md](02-practice/developer/developer-guide.md) - 开发规范
4. 参考 [tool-development.md](02-practice/developer/tool-development.md) - 创建工具

### 问题排查
1. 查看 [technical-debt.md](01-understanding/analysis/technical-debt.md) - 已知问题
2. 查看 [CHANGELOG.md](CHANGELOG.md) - 版本历史
3. 提交 Issue: https://github.com/hongxin/realconsole/issues

## 📝 文档维护原则

遵循"极简主义"设计理念：

1. **保留核心**：只保留对未来有价值的文档
2. **删除冗余**：历史开发记录移至 archive/
3. **清晰分类**：文档按功能明确分类
4. **易于导航**：README 提供完整索引
5. **持续优化**：定期审查文档相关性

## 🔗 相关资源

- **项目主页**: https://github.com/hongxin/realconsole
- **完整 CHANGELOG**: [CHANGELOG.md](CHANGELOG.md)
- **项目指南**: [CLAUDE.md](../CLAUDE.md)
- **发布说明**: [RELEASE-v1.0.0.md](03-evolution/RELEASE-v1.0.0.md)

---

**文档体系版本**: v2.0 (极简主义)
**文档精简**: 82个 → 26个（保留核心 + 230个归档）
**最后清理**: 2025-10-17
