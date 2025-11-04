# 04-reports - 开发报告与技术文档

本目录记录 RealConsole 项目的开发历程、技术决策和实施细节。

## 📁 目录结构

```
04-reports/
├── archives/          # 历史版本归档 (v1.6.0 - v1.20.0)
├── current/           # 当前版本开发 (v1.21.0+)
├── philosophy/        # 设计理念与架构思考
├── features/          # 功能模块开发记录
├── technical/         # 技术改进与修复
└── [重要文档]         # 根目录战略规划文档
```

## 📚 分类导航

### 🗄️ archives/ - 历史归档

**v1.6.0 - v1.20.0** 的所有开发文档（55 个文件）

包含早期版本的开发计划、完成报告、Phase 报告等历史文档。

### 📌 current/ - 当前版本

活跃开发中的版本文档（按版本号组织）：

- **v1.21.0/** - 任务编排系统优化
- **v1.22.0/** - 任务持久化与命令增强
- **v1.22.1/** - 任务命令重构
- **v1.24.0/** - 全面国际化支持（最新）

### 🧠 philosophy/ - 设计理念

核心设计哲学与架构思考（13 个文件）：

- 八卦记成 (`bagua-integration-*.md`)
- 两仪系统 (`liangyyi-*.md`)
- 四维哲学 (`four-dimensions-philosophy.md`)
- 架构反思 (`architecture-reflection-*.md`)

### 🚀 features/ - 功能开发

按模块组织的功能开发记录：

- **trace/** - 统一追踪系统 (Trace 命令)
- **web/** - Web 终端功能
- **memory/** - 记忆系统
- **system-prompt/** - 系统提示词配置

### 🔧 technical/ - 技术改进

技术优化与修复文档：

- **bugfixes/** - Bug 修复记录
- **optimizations/** - 性能优化
- **refactoring/** - 代码重构

## 🎯 根目录重要文档

- `roadmap-to-v2.0.md` - 产品路线图
- `next-phase-strategic-analysis.md` - 战略分析
- `development-lessons-learned.md` - 开发经验总结
- `v1.x-technical-debt-inventory.md` - 技术债务清单
- `v1.x-stabilization-and-release-plan.md` - 稳定与发布计划

---

## 📖 使用说明

### 查找历史版本文档

```bash
# 查看某个版本的所有文档
ls archives/v1.16.0-*.md

# 搜索特定主题
grep -r "LLM" archives/
```

### 查看当前开发进展

```bash
# 最新版本文档
ls current/v1.24.0/

# 所有活跃版本
ls current/*/
```

### 了解设计理念

```bash
# 哲学思想文档
ls philosophy/

# 功能设计文档
ls features/*/
```

---

**组织原则**:
- 按用途分类（时间/哲学/功能/技术）
- 活跃文档置顶（current/）
- 历史文档归档（archives/）
- 清晰的逻辑层次

**最后更新**: 2025-01-04
