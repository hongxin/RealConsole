# 文档重组说明

**日期**: 2025-10-27
**版本**: v1.8.0
**目标**: 简化 README，优化文档结构

## 📝 重组原则

采用**极简主义**理念，学习 [mlx](https://github.com/ml-explore/mlx) 项目风格：

1. **README 简洁化** - 只保留核心信息，详细内容移至 docs/
2. **双语策略** - README.md (英文) + README.cn.md (中文)
3. **开发流程** - 先更新中文版，大版本时更新英文版
4. **链接导航** - 通过链接连接 README 和详细文档

## 📂 新文档结构

```
RealConsole/
├── README.md              # 英文版（简洁，~170 行）
├── README.cn.md           # 中文版（简洁，~170 行）
├── CLAUDE.md              # 项目开发指南（给 AI 助手）
├── CHANGELOG.md           # 版本历史
├── LICENSE                # 许可证
│
└── docs/                  # 详细文档
    ├── README.md          # 文档中心导航
    │
    ├── 00-core/           # 核心理念
    │   ├── philosophy.md  # 一分为三哲学
    │   ├── vision.md      # 产品愿景
    │   └── roadmap.md     # 技术路线图
    │
    ├── 01-understanding/  # 设计理解
    │   ├── design/        # 架构设计文档
    │   └── analysis/      # 需求分析
    │
    ├── 02-practice/       # 实践指南
    │   ├── user/          # 用户手册
    │   │   ├── quickstart.md         # 快速开始（详细版）
    │   │   ├── user-guide.md         # 完整用户手册
    │   │   ├── llm-setup.md          # LLM 配置指南
    │   │   ├── tool-calling-guide.md # 工具调用指南
    │   │   └── ...
    │   │
    │   ├── developer/     # 开发者指南
    │   │   ├── developer-guide.md    # 开发总览
    │   │   ├── api-reference.md      # API 参考
    │   │   ├── architecture.md       # 架构详解
    │   │   └── ...
    │   │
    │   └── use-cases/     # 使用案例
    │
    ├── 03-evolution/      # 演化历程
    │   ├── version-history.md        # 版本历史
    │   └── archives/                 # 归档文档
    │
    └── 04-reports/        # 开发报告
        ├── phase-4.2-p1-completion.md
        ├── phase-4.2-p2.1-completion.md
        └── ...
```

## 🔄 迁移内容

### README.md/README.cn.md（简洁版）

**保留内容**：
- ✅ 项目简介（1-2段）
- ✅ 核心特性（简洁列表，6-8项）
- ✅ 快速开始（基础步骤）
- ✅ 使用示例（2-3个典型示例）
- ✅ 文档链接（快速导航）
- ✅ 架构概览（简图 + 1段说明）
- ✅ 免责声明
- ✅ License 和致谢

**移除内容**（移至详细文档）：
- ❌ 详细功能说明 → `docs/02-practice/user/user-guide.md`
- ❌ 完整命令列表 → `docs/02-practice/user/commands-reference.md`
- ❌ 配置详解 → `docs/02-practice/user/configuration.md`
- ❌ 开发指南 → `docs/02-practice/developer/`
- ❌ 项目结构详解 → `docs/02-practice/developer/project-structure.md`

### docs/ 详细文档

**用户文档** (`docs/02-practice/user/`):
- `quickstart.md` - 详细快速开始指南（从 README 扩展）
- `user-guide.md` - 完整功能手册（所有命令、配置）
- `llm-setup.md` - LLM 配置详解
- `tool-calling-guide.md` - 工具调用完整指南
- `configuration.md` - 配置文件详解
- `faq.md` - 常见问题

**开发者文档** (`docs/02-practice/developer/`):
- `developer-guide.md` - 开发总览
- `architecture.md` - 系统架构详解
- `api-reference.md` - API 参考
- `contributing.md` - 贡献指南
- `project-structure.md` - 项目结构详解

## 📊 对比

### 重组前
```
README.md: 817 行
├── 功能详解：~400 行
├── 配置说明：~150 行
├── 使用示例：~150 行
└── 其他：~117 行
```

### 重组后
```
README.md: ~170 行（英文）
README.cn.md: ~170 行（中文）
├── 项目简介：~30 行
├── 核心特性：~10 行
├── 快速开始：~40 行
├── 使用示例：~50 行
├── 文档链接：~20 行
└── 其他：~20 行

详细内容 → docs/ (~600+ 行，分散在多个文件)
```

## 🎯 设计目标

1. **降低门槛** - 新用户 1 分钟了解项目
2. **快速上手** - 5 分钟完成安装和首次运行
3. **分层文档** - 按需深入，避免信息过载
4. **国际化友好** - 英文版吸引国际用户
5. **维护性强** - 模块化文档，易于更新

## 📝 内容原则

### README（简洁版）
- ✅ "What" - 这是什么项目
- ✅ "Why" - 为什么使用它（核心价值）
- ✅ "How" - 如何快速开始
- ✅ "Where" - 去哪里找更多信息

### docs/（详细版）
- ✅ "Details" - 功能详解
- ✅ "Advanced" - 高级配置
- ✅ "Internals" - 内部实现
- ✅ "Examples" - 完整示例

## 🔗 链接策略

从 README 到 docs/ 的链接应该：

1. **精准定位** - 链接到具体章节，不是整个文档
2. **清晰标注** - 说明链接内容和预期时长
3. **分层导航** - 基础 → 进阶 → 高级

示例：
```markdown
- **[快速开始](docs/02-practice/user/quickstart.md)** - 5 分钟上手指南
- **[用户手册](docs/02-practice/user/user-guide.md)** - 完整功能说明
- **[开发者指南](docs/02-practice/developer/developer-guide.md)** - 架构与扩展
```

## ✅ 验收标准

- [ ] README.md（英文）< 200 行
- [ ] README.cn.md（中文）< 200 行
- [ ] 新用户能在 1 分钟内理解项目价值
- [ ] 新用户能在 5 分钟内完成安装运行
- [ ] 所有详细内容有对应的 docs/ 文档
- [ ] 所有链接有效且准确
- [ ] 双语版本内容一致

## 📅 更新流程

1. **日常开发** - 只更新 README.cn.md（中文版）
2. **功能迭代** - 同步更新详细文档（docs/）
3. **版本发布** - 发布前同步 README.md（英文版）
4. **大版本** - 重新审视文档结构，必要时重组

## 🔍 后续优化

- [ ] 创建 docs/README.md 文档中心导航
- [ ] 添加文档内容索引和标签
- [ ] 创建文档贡献指南
- [ ] 添加文档版本控制
- [ ] 考虑文档国际化（i18n）

---

**维护者**: RealConsole Team
**最后更新**: 2025-10-27
