# RealConsole 文档中心

> **融合东方哲学智慧的智能 CLI Agent**
>
> 基于"一分为三"设计哲学，遵循道德经"道生一，一生二，二生三，三生万物"的演化智慧

---

## 📚 文档架构

本文档系统采用**五态架构**，体现从认知到实践再到演化的完整路径：

```
道（核心理念）
  ↓
理解态 → 实践态 → 演化态 → 协同报告
  ↓       ↓       ↓         ↓
 认知    应用    升华      记录
  ↓       ↓       ↓         ↓
        归档（历史文档）
```

---

## 🎯 快速导航

### 【道】00-core/ - 核心理念层

> "执古之道，以御今之有" - 把握核心，统领全局

核心哲学和产品愿景，指导整个项目的发展方向。

- [philosophy.md](00-core/philosophy.md) - 一分为三哲学思想
- [vision.md](00-core/vision.md) - 产品愿景和定位
- [roadmap.md](00-core/roadmap.md) - 技术路线图

**适合**: 想要深入理解项目理念的所有人

---

### 【理解态】01-understanding/ - 认知和设计

> "知其然，知其所以然" - 理解系统设计的本质

系统架构、设计文档、分析报告和思考笔记。

- [overview.md](01-understanding/overview.md) - 架构总览
- [design/](01-understanding/design/) - 设计文档集
  - intent-philosophy.md - Intent系统设计哲学
  - error-handling.md - 错误处理设计
  - security.md - 安全设计
- [analysis/](01-understanding/analysis/) - 分析文档
  - technical-debt.md - 技术债务清单
  - python-rust-gap.md - Python/Rust对比
- [thinking/](01-understanding/thinking/) - 思考笔记
  - dsl-philosophy.md - DSL设计哲学

**适合**: 架构师、系统设计者、技术决策者

---

### 【实践态】02-practice/ - 使用和开发

> "纸上得来终觉浅，绝知此事要躬行" - 从实践中掌握

用户指南、开发者指南、示例代码和使用场景。

#### 👥 [user/](02-practice/user/) - 用户实践

快速上手和日常使用指南：

- [quickstart.md](02-practice/user/quickstart.md) - 5分钟快速开始
- [user-guide.md](02-practice/user/user-guide.md) - 完整用户手册
- [tool-calling-guide.md](02-practice/user/tool-calling-guide.md) - 工具调用指南
- [intent-dsl-guide.md](02-practice/user/intent-dsl-guide.md) - Intent DSL使用
- [llm-setup.md](02-practice/user/llm-setup.md) - LLM配置指南
- [env-config.md](02-practice/user/env-config.md) - 环境变量配置

#### 💻 [developer/](02-practice/developer/) - 开发者实践

扩展和定制RealConsole：

- [developer-guide.md](02-practice/developer/developer-guide.md) - 开发者指南
- [tool-development.md](02-practice/developer/tool-development.md) - 创建自定义工具
- [api-reference.md](02-practice/developer/api-reference.md) - API参考

#### 📦 [use-cases/](02-practice/use-cases/) - 使用场景

- basic-10-cases.md / basic-20-cases.md - 基础场景
- advanced-30-cases.md - 进阶场景
- expert-50-cases.md - 专家场景
- selected-cases.md - 精选案例

**适合**: 所有用户和开发者

---

### 【演化态】03-evolution/ - 进展和历史

> "穷则变，变则通，通则久" - 持续演化，生生不息

项目的发展历程、功能实现记录和未来规划。

- [phases/](03-evolution/phases/) - 阶段总结
  - phase3-intent-dsl.md - Phase 3完整记录
  - phase5-pipeline.md - Phase 5 Pipeline DSL
  - phase7-polish.md - Phase 7 最终打磨
  - phase-9-v0.9.0-release.md - Phase 9 统计可视化
  - phase9.1-week2-error-auto-fixing.md - Phase 9.1 错误修复
  - phase-10-summary.md - Phase 10 任务编排系统 ⭐
- [features/](03-evolution/features/) - 功能实现
  - shell-execution.md - Shell执行系统
  - streaming.md - 流式输出实现
  - git-assistant.md - Git智能助手
  - log-analyzer.md - 日志分析器
  - system-monitor.md - 系统监控
  - summary.md - 功能总览
- [RELEASE-v1.0.0.md](03-evolution/RELEASE-v1.0.0.md) - **v1.0.0 正式发布** 🎉

**适合**: 贡献者、项目维护者

---

### 【协同报告】04-reports/ - 工作成果

> "温故而知新" - 记录决策，追溯思考

协同工作过程中产生的各类分析、总结和决策报告。

- [reorganization-2025-10-16.md](04-reports/reorganization-2025-10-16.md) - 文档重组报告

**适合**: 所有参与者，便于了解项目决策过程

---

## 🗂️ 归档文档

- [archive/](archive/) - 历史文档归档
  - old-progress/ - 详细开发记录（80+文档）
  - old-designs/ - 过时的设计文档

---

## 🧭 推荐阅读路径

### 路径 1: 新用户入门（5分钟）
```
quickstart.md → llm-setup.md → user-guide.md
```

### 路径 2: 理解设计哲学（15分钟）
```
philosophy.md → vision.md → overview.md → dsl-philosophy.md
```

### 路径 3: 开发者深度学习（30分钟）
```
overview.md → developer-guide.md → tool-development.md → api-reference.md
```

### 路径 4: 贡献者全面了解（60分钟）
```
philosophy.md → overview.md → roadmap.md → phases/ → technical-debt.md
```

---

## 📊 文档统计

- **核心文档**: 3个
- **理解态**: 12个（设计5 + 分析4 + 思考3）
- **实践态**: 20个（用户6 + 开发者3 + 用例5 + 示例）
- **演化态**: 14个（阶段4 + 功能8 + 里程碑2）
- **协同报告**: 持续增长（当前1个）
- **归档**: 226个历史文档

**总计**: 约50个活跃文档，清晰分类，易于导航

---

## 💡 文档设计原则

1. **三态分离** - 理解、实践、演化各有其位
2. **数字编排** - 00-03前缀体现演化路径
3. **统一命名** - 全小写连字符，简洁语义化
4. **清晰导航** - 每层都有README索引
5. **精简高效** - 保留活跃文档，归档历史记录

---

## 📝 最新更新

**2025-10-17** - v1.0.0 正式发布 🎉
- ✅ Phase 10 任务编排系统完成
- ✅ 更新所有文档到 v1.0.0
- ✅ 创建正式发布说明（RELEASE-v1.0.0.md）
- ✅ 更新技术路线图和里程碑
- ✅ 完整文档一致性验证

**2025-10-16** - 文档体系完善
- ✅ 实施"一分为三"五态架构（00-04 + archive）
- ✅ 精简活跃文档至50个
- ✅ 归档226个历史文档
- ✅ 统一命名规范（全小写连字符）
- ✅ 完善导航系统（7个README索引）
- ✅ 新增04-reports/协同报告目录
- ✅ 文档构建规则写入CLAUDE.md

---

## 🔗 外部链接

- **项目主页**: [../README.md](../README.md)
- **项目说明**: [../CLAUDE.md](../CLAUDE.md)
- **完整变更日志**: [CHANGELOG.md](CHANGELOG.md)
- **配置文件**: [../realconsole.yaml](../realconsole.yaml)
- **GitHub**: https://github.com/hongxin/realconsole

---

**RealConsole 文档中心** | 道生一，一生二，二生三，三生万物 ✨
