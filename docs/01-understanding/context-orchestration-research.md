# 上下文编排系统研究

**创建时间**: 2025-10-22
**研究目标**: 为 Memory 2.0（上下文编排系统）收集先进技术和实现经验
**状态**: 研究中

---

## 研究背景

### 核心问题

**上下文限制是 LLM 应用的根本瓶颈**：
- 当前主流模型：128k tokens（约 40 万字符）
- 长对话、复杂任务、大代码库 → 远超限制
- 如何在有限空间内提供最有效的信息？

### RealConsole 的需求

**Memory 2.0 的定位**：
- ❌ 不是：简单的全量记录系统
- ✅ 而是：智能的上下文编排系统

**三大核心能力**：
1. **管理**：决策哪些信息进入上下文
2. **优化**：在 128k 限制下最大化信息价值
3. **可视化**：让用户理解和控制上下文组织

---

## 研究对象

### 1. Claude Code

**官方文档**：
- https://docs.claude.com/en/docs/claude-code
- 重点：上下文管理机制

**关键特性**（待研究）：
- [ ] 如何管理长对话历史？
- [ ] 如何处理大型代码库？
- [ ] 上下文压缩策略是什么？
- [ ] 用户可见性和控制度如何？

**研究方法**：
1. 阅读官方文档
2. 实际使用体验
3. 观察上下文行为
4. 总结设计模式

---

### 2. GitHub Copilot / Codex

**研究重点**：
- 代码上下文的智能选择
- 如何从大代码库中提取相关代码？
- 如何平衡文件数量和上下文长度？

**已知技术**：
- **Proximity-based context**: 基于当前编辑位置选择相关代码
- **Import graph analysis**: 分析依赖关系，包含相关导入
- **Semantic chunking**: 智能分割代码块

**待研究问题**：
- [ ] 具体的相关性算法是什么？
- [ ] 如何处理跨文件依赖？
- [ ] Token 预算如何分配？
- [ ] 是否有用户可调参数？

**参考资料**：
- GitHub Copilot 技术博客
- OpenAI Codex 论文
- 相关开源实现（如 fauxpilot）

---

### 3. Google Gemini CLI

**研究重点**：
- 长上下文窗口（1M+ tokens）的管理策略
- 即使有超长上下文，如何组织信息？
- 与短上下文模型的差异化设计

**待研究问题**：
- [ ] 超长上下文下的信息组织原则？
- [ ] 是否仍需要上下文压缩？
- [ ] 用户体验如何设计？
- [ ] 性能优化策略？

**参考资料**：
- Gemini 官方文档
- Google AI 博客
- 开发者社区实践

---

### 4. iFlow

**研究重点**：
- 工作流中的上下文传递
- 多步骤任务的状态管理
- 上下文在不同 Agent 间的共享

**待研究问题**：
- [ ] 如何在工作流节点间传递上下文？
- [ ] 上下文的增量更新机制？
- [ ] 分布式上下文管理？
- [ ] 可视化和调试工具？

**参考资料**：
- iFlow 官方文档
- 开源代码（如果有）
- 社区案例分享

---

### 5. 其他值得研究的系统

#### LangChain

**相关组件**：
- Memory 模块
- ConversationBufferMemory
- ConversationSummaryMemory
- VectorStoreRetrieverMemory

**研究价值**：
- 成熟的上下文管理抽象
- 多种策略的对比
- 与向量数据库集成

#### AutoGPT / BabyAGI

**研究重点**：
- 长期记忆的实现
- 任务上下文的持久化
- 自主 Agent 的上下文策略

#### Cursor

**研究重点**：
- 代码编辑器中的上下文管理
- 实时上下文更新
- 用户交互设计

---

## 核心技术方向

### 方向 1：基于相关性的选择（Relevance-based Selection）

**核心思想**：
```
给定当前任务/问题，从历史中选择最相关的信息
```

**关键技术**：
1. **向量相似度**
   - 使用 embedding 模型编码历史记录
   - 计算与当前任务的余弦相似度
   - 选择 Top-K 最相关的记录

2. **关键词匹配**
   - TF-IDF 加权
   - BM25 算法
   - 正则表达式匹配

3. **语义理解**
   - NLU 提取意图
   - 实体识别
   - 关系抽取

**示例伪代码**：
```rust
fn select_relevant_context(
    current_task: &str,
    history: &[HistoryEntry],
    max_tokens: usize
) -> Vec<HistoryEntry> {
    // 1. 编码当前任务
    let task_embedding = embed(current_task);

    // 2. 计算所有历史的相似度
    let mut scored_history: Vec<_> = history
        .iter()
        .map(|entry| {
            let relevance = cosine_similarity(
                task_embedding,
                embed(&entry.content)
            );
            (entry, relevance)
        })
        .collect();

    // 3. 按相关性排序
    scored_history.sort_by(|a, b|
        b.1.partial_cmp(&a.1).unwrap()
    );

    // 4. 贪心选择，直到达到 token 限制
    let mut selected = Vec::new();
    let mut total_tokens = 0;

    for (entry, _score) in scored_history {
        let entry_tokens = count_tokens(&entry.content);
        if total_tokens + entry_tokens <= max_tokens {
            selected.push(entry.clone());
            total_tokens += entry_tokens;
        }
    }

    selected
}
```

---

### 方向 2：基于摘要的压缩（Summary-based Compression）

**核心思想**：
```
保留全部历史，但对旧内容进行渐进式摘要
```

**策略**：
1. **滑动窗口摘要**
   ```
   最近 N 条 → 完整保留
   N-2N 条 → 摘要为一半
   2N-4N 条 → 摘要为 1/4
   更早 → 高度概括
   ```

2. **层次化摘要**
   ```
   Level 1: 原始记录（最近的）
   Level 2: 句子级摘要
   Level 3: 段落级摘要
   Level 4: 文档级摘要
   ```

3. **关键信息提取**
   ```
   保留：
   - 决策点
   - 错误和解决方案
   - 重要结论
   - 用户偏好
   ```

**示例伪代码**：
```rust
fn hierarchical_summary(
    history: &[HistoryEntry],
    max_tokens: usize
) -> String {
    let mut context = String::new();
    let mut used_tokens = 0;

    // 最近的完整保留
    for entry in history.iter().rev().take(5) {
        if used_tokens + entry.tokens() <= max_tokens {
            context.push_str(&entry.full_text());
            used_tokens += entry.tokens();
        }
    }

    // 中期的摘要
    if used_tokens < max_tokens {
        let mid_range = &history[..history.len()-5];
        let summary = summarize(mid_range, max_tokens - used_tokens);
        context.push_str(&summary);
    }

    context
}

fn summarize(entries: &[HistoryEntry], max_tokens: usize) -> String {
    // 使用 LLM 生成摘要
    llm.generate(&format!(
        "请将以下对话摘要为 {} tokens:\n{}",
        max_tokens,
        entries.iter().map(|e| e.text()).join("\n")
    ))
}
```

---

### 方向 3：基于结构的组织（Structure-based Organization）

**核心思想**：
```
将上下文组织为有层次的结构，而非平面列表
```

**结构类型**：

1. **树形结构**
   ```
   Root (当前任务)
   ├─ 直接相关的对话
   ├─ 依赖的代码片段
   │  ├─ 函数定义
   │  └─ 类型定义
   └─ 背景知识
      ├─ 项目文档
      └─ 外部参考
   ```

2. **图结构**
   ```
   节点：历史记录、代码、文档
   边：引用、依赖、相似性
   遍历：从当前任务开始的广度优先搜索
   ```

3. **时间线结构**
   ```
   按时间组织，但标注重要节点：
   ● 任务开始
   ◉ 关键决策
   ○ 中间结果
   ◉ 错误发生
   ○ 问题解决
   ● 任务完成
   ```

**示例：树形上下文**
```rust
struct ContextTree {
    root: ContextNode,
}

struct ContextNode {
    content: String,
    importance: f64,
    children: Vec<ContextNode>,
}

impl ContextTree {
    fn build(current_task: &str, history: &[Entry]) -> Self {
        let root = ContextNode {
            content: current_task.to_string(),
            importance: 1.0,
            children: vec![],
        };

        // 构建子节点
        let mut tree = ContextTree { root };
        tree.attach_related_conversations(history);
        tree.attach_code_dependencies(history);
        tree.attach_background_knowledge(history);

        tree
    }

    fn serialize(&self, max_tokens: usize) -> String {
        // 优先遍历高 importance 的节点
        // 广度优先，直到达到 token 限制
        self.bfs_serialize(max_tokens)
    }
}
```

---

### 方向 4：混合策略（Hybrid Approach）

**核心思想**：
```
组合多种策略，针对不同类型的信息使用不同方法
```

**分层策略**：

```
┌────────────────────────────────────────┐
│ Layer 1: 系统提示词（固定）             │  10% tokens
├────────────────────────────────────────┤
│ Layer 2: 项目上下文（结构化）           │  20% tokens
│  - 项目文档摘要                         │
│  - 关键API/函数签名                     │
│  - 代码结构概览                         │
├────────────────────────────────────────┤
│ Layer 3: 任务历史（摘要 + 原文）        │  40% tokens
│  - 最近 3 轮：完整对话                  │
│  - 之前 10 轮：摘要                     │
│  - 更早：高度概括                       │
├────────────────────────────────────────┤
│ Layer 4: 相关知识（向量检索）           │  20% tokens
│  - 类似问题的解决方案                   │
│  - 相关代码片段                         │
│  - 外部文档引用                         │
├────────────────────────────────────────┤
│ Layer 5: 当前任务（完整）               │  10% tokens
│  - 用户最新输入                         │
│  - 当前代码上下文                       │
└────────────────────────────────────────┘
```

**动态调整**：
- 根据任务类型调整各层比例
- 简单问题：减少历史，增加当前
- 复杂问题：增加相关知识
- 代码任务：增加项目上下文

---

## 可视化设计

### 用户需要看到什么？

1. **上下文组成**
   ```
   当前上下文（使用 45K/128K tokens）：
   ┌─────────────────────────────────────┐
   │ █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░  │ 35%
   └─────────────────────────────────────┘

   组成：
   • 系统提示: 2K (5%)
   • 项目上下文: 8K (18%)
   • 对话历史: 20K (44%)
   • 相关知识: 10K (22%)
   • 当前任务: 5K (11%)
   ```

2. **选择依据**
   ```
   最近 3 轮对话（完整）：
   ✓ [10:25] 用户: 如何优化这个函数？
     [10:26] AI: 可以使用缓存...
   ✓ [10:27] 用户: 缓存的实现细节？
     [10:28] AI: 使用 HashMap...
   ✓ [10:30] 用户: 如何处理过期？  ← 当前

   相关历史（向量检索）：
   ✓ [09:15] 相似度 0.85 - 讨论过 LRU 缓存
   ✓ [昨天] 相似度 0.72 - 实现过类似功能
   ✗ [上周] 相似度 0.45 - 不相关，已排除
   ```

3. **可调控制**
   ```
   上下文策略：
   • 最近对话数: [3] 条 (完整)
   • 历史摘要: [启用] [10] 条
   • 向量检索: [启用] Top-[5]
   • 代码上下文: [自动]
   • 项目文档: [包含摘要]

   [应用] [重置] [保存为预设]
   ```

---

## 实现路线图

### Phase 1: 基础研究（2-3周）

**目标**：深入理解先进系统的设计

**任务**：
- [ ] 研究 Claude Code 的上下文管理（1周）
  - 实际使用和观察
  - 文档阅读
  - 总结设计模式

- [ ] 研究 Copilot/Codex（1周）
  - 技术论文阅读
  - 开源实现分析
  - 算法复现实验

- [ ] 研究 Gemini/iFlow/其他（1周）
  - 对比分析
  - 提取共性
  - 识别差异

**产出**：
- 详细的技术调研报告
- 可借鉴的设计模式
- 算法原型代码

---

### Phase 2: 原型实现（2-3周）

**目标**：实现核心算法的原型

**任务**：
- [ ] 向量相似度检索（3天）
  - 集成 embedding 模型
  - 实现相似度计算
  - 性能测试

- [ ] 上下文压缩（3天）
  - 摘要生成
  - 关键信息提取
  - 质量评估

- [ ] 混合策略（1周）
  - 多层上下文组织
  - Token 预算管理
  - 动态调整逻辑

- [ ] 可视化原型（3天）
  - 上下文组成展示
  - 选择依据说明
  - 交互控制界面

**产出**：
- 可运行的原型系统
- 性能和质量测试报告
- 用户体验评估

---

### Phase 3: 集成到 RealConsole（1-2周）

**目标**：替换 Memory 为新的上下文编排系统

**任务**：
- [ ] API 设计（2天）
  - 定义接口
  - 向后兼容
  - 文档编写

- [ ] 集成实现（5天）
  - 替换旧 Memory
  - 与 Context 集成
  - 与 Agent 集成

- [ ] 测试和优化（3天）
  - 单元测试
  - 集成测试
  - 性能优化

**产出**：
- Memory 2.0 正式版
- 完整测试覆盖
- 用户文档

---

### Phase 4: 迭代和改进（持续）

**目标**：根据实际使用持续优化

**任务**：
- 收集用户反馈
- 监控性能指标
- 调整算法参数
- 增加新策略

---

## 研究记录

### 研究笔记模板

每个系统研究完成后，使用以下模板记录：

```markdown
## [系统名称] 研究笔记

**研究日期**: YYYY-MM-DD
**研究者**: [姓名]

### 核心发现

1. **上下文管理策略**
   - 描述...
   - 优点...
   - 缺点...

2. **关键技术**
   - 算法1: 描述 + 伪代码
   - 算法2: 描述 + 伪代码

3. **用户体验**
   - 可见性...
   - 可控性...
   - 学习曲线...

### 可借鉴之处

- [ ] 技术点1: 应用场景 + 实现难度
- [ ] 技术点2: ...

### 不适用之处

- 原因1: ...
- 原因2: ...

### 代码示例

```rust
// 关键算法的 Rust 实现
```

### 参考资料

- [链接1](url)
- [链接2](url)
```

---

## 关键指标

### 评估维度

**准确性**：
- 选择的上下文与当前任务的相关度
- 测量：用户满意度调查、A/B 测试

**完整性**：
- 是否包含了完成任务所需的所有信息
- 测量：任务成功率、需要追问的次数

**效率**：
- Token 利用率（有效信息 / 总 tokens）
- 测量：信息熵、冗余度分析

**性能**：
- 上下文构建的延迟
- 目标：< 100ms

**可理解性**：
- 用户能否理解上下文的组织逻辑
- 测量：用户调研、可用性测试

---

## 开放问题

### 技术问题

1. **Embedding 模型选择**
   - 使用哪个模型？本地 vs API？
   - 性能 vs 准确性的权衡？

2. **向量存储**
   - 使用哪个向量数据库？
   - 如何处理增量更新？

3. **摘要质量**
   - 如何评估摘要的质量？
   - 是否需要人工校验？

### 设计问题

1. **自动 vs 手动**
   - 默认完全自动，还是提供手动控制？
   - 如何平衡智能化和可控性？

2. **性能 vs 质量**
   - 实时计算 vs 预计算？
   - 准确性 vs 延迟的权衡？

3. **通用 vs 专用**
   - 一套策略适用所有场景？
   - 还是针对不同任务类型定制？

---

## 下一步行动

### 立即开始（本周）

- [ ] 开始使用 Claude Code，观察其上下文管理行为
- [ ] 阅读 Copilot 相关技术博客和论文
- [ ] 收集 Gemini CLI 的实践案例

### 近期（2周内）

- [ ] 完成 Claude Code 的深度研究
- [ ] 实现向量相似度检索的原型
- [ ] 编写第一版技术调研报告

### 中期（1月内）

- [ ] 完成所有系统的研究
- [ ] 实现混合策略原型
- [ ] 完成可视化设计

---

**相关文档**:
- `memory-system-redesign.md` - Memory 整体设计
- `four-dimensions-philosophy.md` - 哲学基础

**变更历史**:
| 版本 | 日期 | 说明 |
|------|------|------|
| v1.0 | 2025-10-22 | 初始研究框架 |
