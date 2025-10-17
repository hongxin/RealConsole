# Intent 匹配的哲学改进方案

> **践行"一分为三"思想，从二元对立到多维演化**
> 完成日期：2025-10-15
> 版本：1.0

---

## 🎯 问题案例

### 失败案例

**用户输入**：
```
显示当前目录下体积最大的rs文件
```

**期望行为**：
- 查找当前目录下所有 `.rs` 文件
- 按体积排序（降序）
- 显示最大的那个

**实际行为**：
- ❌ 匹配到：`list_directory` (置信度: 1.00)
- ❌ 执行：`ls -lh .`
- ❌ 结果：列出所有文件，没有过滤、没有排序

### 根本原因

当前的 Intent 匹配采用**二分法思维**：

```
if 包含("显示") && 包含("目录") → list_directory
else if 包含("查找") && 包含("大文件") → find_large_files
```

**问题**：
1. ❌ **关键词过于简单**：只看单个词，不看组合语义
2. ❌ **缺少多维分析**：没有识别"动作 + 对象 + 条件 + 范围"的组合
3. ❌ **置信度虚高**：简单匹配就给 1.00，缺少细粒度评分

---

## 🌟 哲学理念的应用

### 从二分法到"一分为三"

**传统二分法**：
```
用户意图 ∈ {Intent A, Intent B}
```

**一分为三**：
```
用户意图 = f(动作维度, 对象维度, 条件维度, 范围维度, ...)
```

### 易经八卦的映射

用户意图不是单一的，而是**八个维度的组合**（对应易经八卦）：

| 卦象 | 维度 | 特征 | 示例关键词 |
|------|------|------|-----------|
| 乾☰ | **动作** | 执行、运行、启动 | 运行、执行、启动 |
| 坤☷ | **对象** | 文件、目录、服务 | 文件、目录、进程 |
| 震☳ | **变化** | 修改、更新、创建 | 创建、修改、删除 |
| 巽☴ | **过滤** | 筛选、排序、限制 | 最大、最新、前10个 |
| 坎☵ | **深度** | 递归、层级、嵌套 | 递归、所有子目录 |
| 离☲ | **展示** | 显示、列出、查看 | 显示、列出、查看 |
| 艮☶ | **范围** | 路径、目录、作用域 | 当前目录、/var/log |
| 兑☱ | **条件** | 大小、时间、类型 | 大于100MB、.rs文件 |

### 案例分析："显示当前目录下体积最大的rs文件"

**八卦解析**：

| 维度 | 提取结果 | 强度 |
|------|---------|------|
| 离（展示） | "显示" | 🔴 强 |
| 坤（对象） | "rs文件" | 🔴 强 |
| 巽（过滤） | "体积最大" | 🔴 强 |
| 兑（条件） | ".rs" 文件类型 | 🟡 中 |
| 艮（范围） | "当前目录" | 🟡 中 |
| 坎（深度） | 无递归 | 🟢 弱 |
| 震（变化） | 无变化 | 🟢 弱 |
| 乾（动作） | 无执行 | 🟢 弱 |

**Intent 对比**：

#### list_directory 的八卦特征

| 维度 | 支持度 | 说明 |
|------|-------|------|
| 离（展示） | 🔴 强 | ✅ "显示"、"列出" |
| 坤（对象） | 🟡 中 | ✅ "文件"、"目录" |
| 巽（过滤） | 🟢 弱 | ❌ 无排序、筛选 |
| 兑（条件） | 🟢 弱 | ❌ 无类型过滤 |

**匹配度**：强 + 中 + 弱 + 弱 = **50%**

#### find_large_files 的八卦特征

| 维度 | 支持度 | 说明 |
|------|-------|------|
| 离（展示） | 🟡 中 | ⚠️  "查找"不如"显示"直接 |
| 坤（对象） | 🔴 强 | ✅ "文件" |
| 巽（过滤） | 🔴 强 | ✅ 大小排序 |
| 兑（条件） | 🟡 中 | ⚠️  大小条件，但不是类型 |

**匹配度**：中 + 强 + 强 + 中 = **75%**

**结论**：应该匹配 `find_large_files`，但实际匹配了 `list_directory` → **匹配算法有缺陷**

---

## 🔧 解决方案设计

### 三层递进方案

#### 第一层：短期修复（Phase 6.2）

**目标**：修复当前案例，快速见效

**方案**：
1. **增强关键词权重**：
   - "体积最大"、"最大"、"最小" → 高权重否定词
   - 出现这些词时，降低 `list_directory` 的置信度

2. **正则优先级调整**：
   ```rust
   // list_directory: 排除"最大"、"最小"等排序词
   r"(?i)(查看|显示|列出).*(目录|文件夹|当前)(?!.*(最大|最小|排序|最新))"

   // find_large_files: 扩展支持"最大"
   r"(?i)(查找|显示).*(最大|最小|大于|小于|体积)"
   ```

3. **创建新 Intent**：`find_largest_files_by_type`
   - 专门处理"最大的XX文件"
   - 支持文件类型过滤（.rs, .py, .md）
   - 模板：`find {path} -name '*.{ext}' -type f -exec ls -lh {} + | sort -k5 -hr | head -n 1`

**优点**：快速实现，立即见效
**缺点**：仍然是关键词匹配，治标不治本

#### 第二层：多维匹配（Phase 6.3）

**目标**：实现八卦维度的多维分析

**方案**：

```rust
/// 八维意图特征（对应易经八卦）
#[derive(Debug, Clone)]
pub struct IntentFeatureVector {
    /// 乾：动作维度（execute, run, start）
    pub action_strength: f64,

    /// 坤：对象维度（file, directory, process）
    pub target_strength: f64,

    /// 震：变化维度（create, modify, delete）
    pub change_strength: f64,

    /// 巽：过滤维度（sort, filter, limit）
    pub filter_strength: f64,

    /// 坎：深度维度（recursive, nested）
    pub depth_strength: f64,

    /// 离：展示维度（show, list, display）
    pub display_strength: f64,

    /// 艮：范围维度（path, scope）
    pub scope_strength: f64,

    /// 兑：条件维度（size, type, time）
    pub condition_strength: f64,
}

impl IntentFeatureVector {
    /// 计算与另一个向量的相似度（余弦距离）
    pub fn similarity(&self, other: &Self) -> f64 {
        let dot_product =
            self.action_strength * other.action_strength +
            self.target_strength * other.target_strength +
            self.change_strength * other.change_strength +
            self.filter_strength * other.filter_strength +
            self.depth_strength * other.depth_strength +
            self.display_strength * other.display_strength +
            self.scope_strength * other.scope_strength +
            self.condition_strength * other.condition_strength;

        let self_norm = (
            self.action_strength.powi(2) +
            self.target_strength.powi(2) +
            self.change_strength.powi(2) +
            self.filter_strength.powi(2) +
            self.depth_strength.powi(2) +
            self.display_strength.powi(2) +
            self.scope_strength.powi(2) +
            self.condition_strength.powi(2)
        ).sqrt();

        let other_norm = (
            other.action_strength.powi(2) +
            other.target_strength.powi(2) +
            other.change_strength.powi(2) +
            other.filter_strength.powi(2) +
            other.depth_strength.powi(2) +
            other.display_strength.powi(2) +
            other.scope_strength.powi(2) +
            other.condition_strength.powi(2)
        ).sqrt();

        if self_norm == 0.0 || other_norm == 0.0 {
            return 0.0;
        }

        dot_product / (self_norm * other_norm)
    }
}

/// 从用户输入提取特征向量
pub fn extract_features(input: &str) -> IntentFeatureVector {
    let input_lower = input.to_lowercase();

    IntentFeatureVector {
        action_strength: calculate_action_strength(&input_lower),
        target_strength: calculate_target_strength(&input_lower),
        change_strength: calculate_change_strength(&input_lower),
        filter_strength: calculate_filter_strength(&input_lower),
        depth_strength: calculate_depth_strength(&input_lower),
        display_strength: calculate_display_strength(&input_lower),
        scope_strength: calculate_scope_strength(&input_lower),
        condition_strength: calculate_condition_strength(&input_lower),
    }
}

/// 示例：计算过滤维度强度
fn calculate_filter_strength(input: &str) -> f64 {
    let mut strength = 0.0;

    // 排序关键词
    if input.contains("最大") { strength += 0.9; }
    if input.contains("最小") { strength += 0.9; }
    if input.contains("最新") { strength += 0.8; }
    if input.contains("排序") { strength += 0.7; }
    if input.contains("前") && input.contains("个") { strength += 0.6; }
    if input.contains("top") { strength += 0.7; }

    strength.min(1.0)  // 限制在 [0, 1]
}
```

**Intent 定义增强**：

```rust
impl Intent {
    /// 定义该 Intent 的八卦特征向量
    pub fn with_feature_vector(mut self, vector: IntentFeatureVector) -> Self {
        self.feature_vector = Some(vector);
        self
    }
}

// 示例：list_directory 的特征向量
fn list_directory_features() -> IntentFeatureVector {
    IntentFeatureVector {
        action_strength: 0.1,    // 弱：不执行复杂操作
        target_strength: 0.7,    // 强：文件/目录
        change_strength: 0.0,    // 无：只读操作
        filter_strength: 0.0,    // 弱：无过滤排序
        depth_strength: 0.2,     // 弱：单层目录
        display_strength: 1.0,   // 强：主要是展示
        scope_strength: 0.8,     // 强：需要路径
        condition_strength: 0.0, // 弱：无条件
    }
}

// 示例：find_large_files 的特征向量
fn find_large_files_features() -> IntentFeatureVector {
    IntentFeatureVector {
        action_strength: 0.3,    // 中：查找操作
        target_strength: 0.9,    // 强：文件
        change_strength: 0.0,    // 无：只读
        filter_strength: 0.9,    // 强：大小排序
        depth_strength: 0.6,     // 中：可递归
        display_strength: 0.7,   // 强：展示结果
        scope_strength: 0.8,     // 强：需要路径
        condition_strength: 0.9, // 强：大小条件
    }
}
```

**匹配算法**：

```rust
impl IntentMatcher {
    /// 多维匹配：结合关键词和特征向量
    pub fn match_intent_multidim(&self, input: &str) -> Vec<IntentMatch> {
        // 1. 提取用户输入的特征向量
        let input_features = extract_features(input);

        // 2. 关键词匹配（传统方式）
        let keyword_matches = self.match_intent(input);

        // 3. 特征向量匹配
        let mut multidim_matches = Vec::new();
        for intent in &self.intents {
            if let Some(intent_features) = &intent.feature_vector {
                let similarity = input_features.similarity(intent_features);

                // 查找对应的关键词匹配
                let keyword_score = keyword_matches
                    .iter()
                    .find(|m| m.intent.name == intent.name)
                    .map(|m| m.confidence)
                    .unwrap_or(0.0);

                // 综合评分：关键词 40% + 特征向量 60%
                let final_score = keyword_score * 0.4 + similarity * 0.6;

                if final_score > 0.3 {
                    multidim_matches.push(IntentMatch {
                        intent: intent.clone(),
                        confidence: final_score,
                        extracted_entities: HashMap::new(),
                    });
                }
            }
        }

        // 4. 按置信度排序
        multidim_matches.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap()
        });

        multidim_matches
    }
}
```

**优点**：科学、全面、可扩展
**缺点**：实现复杂，需要为每个 Intent 定义特征向量

#### 第三层：自适应学习（Phase 7）

**目标**：系统自动学习和优化

**方案**：
1. **记录用户反馈**：
   - 用户确认/拒绝意图匹配
   - 记录实际执行的命令

2. **动态调整权重**：
   - 根据历史数据优化八维权重
   - 自适应调整关键词权重

3. **发现新模式**：
   - 聚类分析用户输入
   - 自动提议新 Intent

**优点**：长期优化，越用越智能
**缺点**：需要大量数据和时间

---

## 🎯 实施计划

### Phase 6.2（本次实施）

**时间**：2-3小时

**任务**：
1. ✅ 创建本设计文档
2. 🔄 修复 `list_directory` 的正则（排除排序词）
3. 🔄 增强 `find_large_files` 的灵活性
4. 🔄 创建新 Intent：`find_largest_files_by_type`
5. 🔄 测试用例验证

### Phase 6.3（下一阶段）

**时间**：1-2天

**任务**：
1. 实现 `IntentFeatureVector` 结构
2. 实现八维特征提取函数
3. 为所有 Intent 定义特征向量
4. 实现多维匹配算法
5. 性能测试和优化

### Phase 7（未来规划）

**时间**：1周+

**任务**：
1. 实现用户反馈记录
2. 实现自适应学习算法
3. 实现模式发现和聚类
4. A/B 测试和效果验证

---

## 📚 参考文献

- **易经**：八卦、六十四卦的变化智慧
- **道德经第二十二章**："少则得，多则惑"
- **PHILOSOPHY.md**：一分为三的基础思想
- **PHILOSOPHY_ADVANCED.md**：变化的变化，状态演化系统

---

**文档版本**: 1.0
**完成日期**: 2025-10-15
**维护者**: RealConsole Team

**核心理念**：
> 意图不是离散的选项，而是多维向量空间中的一个点。
> 匹配不是简单的if-else，而是多重视角的综合评判。
> 系统应该能学习、能演化、能升华。✨
