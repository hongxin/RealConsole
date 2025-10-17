# Pipeline DSL 示例 - 从"枚举"到"组合"

> **Phase 6.3 原型验证**
> 日期：2025-10-15
> 状态：✅ 概念验证成功

---

## 🎯 核心理念

### 问题：刚性枚举的局限

**传统方式**（Phase 6.2 之前）：
```rust
// ❌ 需要为每个变体创建新 Intent
find_largest_files   // 最大
find_smallest_files  // 最小 ← 需要新增
find_newest_files    // 最新
find_oldest_files    // 最旧 ← 需要新增
...                  // 无穷无尽
```

**问题**：
- 组合爆炸：N 个字段 × M 个方向 = N×M 种 Intent
- 维护困难：每次都要写完整的Intent+Template
- 无法泛化：用户说"第二小"、"倒数第三大"怎么办？

### 解决：组合式Pipeline

**Pipeline DSL 方式**：
```rust
// ✅ 基础操作（象）+ 参数（爻）= 无穷变化（卦）
ExecutionPlan {
    operations: vec![
        BaseOperation::FindFiles { ... },      // 象1：查找
        BaseOperation::SortFiles { field, direction },  // 象2：排序
        BaseOperation::LimitFiles { count },    // 象3：限制
    ]
}
```

**优势**：
- ✅ 无限扩展：改变参数即可
- ✅ 易于维护：操作复用
- ✅ 符合哲学："象"不变，"爻"变化

---

## 📚 基础概念

### 三层架构

```
┌─────────────────────────────────────────┐
│  BaseOperation (象 - 不变的基础操作)    │
│  ├─ FindFiles                           │
│  ├─ SortFiles                           │
│  └─ LimitFiles                          │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  Parameters (爻 - 变化的维度)           │
│  ├─ Field: Size/Time/Name              │
│  ├─ Direction: Ascending/Descending    │
│  └─ Count: 1/10/100                    │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  ExecutionPlan (卦 - 操作的组合)        │
│  operations: Vec<BaseOperation>         │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  Shell Command (命令生成)               │
│  find ... | sort ... | head ...        │
└─────────────────────────────────────────┘
```

---

## 🚀 示例代码

### 示例 1：查找最大的 rs 文件

```rust
use realconsole::dsl::pipeline::{ExecutionPlan, BaseOperation, Field, Direction};

// 构建执行计划
let plan = ExecutionPlan::new()
    .with_operation(BaseOperation::FindFiles {
        path: ".".to_string(),
        pattern: "*.rs".to_string(),
    })
    .with_operation(BaseOperation::SortFiles {
        field: Field::Size,
        direction: Direction::Descending,  // 最大 = 降序
    })
    .with_operation(BaseOperation::LimitFiles {
        count: 1,
    });

// 生成命令
let command = plan.to_shell_command();
println!("{}", command);
// → find . -name '*.rs' -type f -exec ls -lh {} + | sort -k5 -hr | head -n 1
```

**输出示例**：
```
-rw-r--r--  1 user  group   47K Oct 15 21:41 ./src/dsl/intent/matcher.rs
```

---

### 示例 2：查找最小的 rs 文件

**核心洞察**：只需改变一个参数！

```rust
use realconsole::dsl::pipeline::{ExecutionPlan, BaseOperation, Field, Direction};

// 与示例1完全相同的结构，只有Direction不同
let plan = ExecutionPlan::new()
    .with_operation(BaseOperation::FindFiles {
        path: ".".to_string(),
        pattern: "*.rs".to_string(),
    })
    .with_operation(BaseOperation::SortFiles {
        field: Field::Size,
        direction: Direction::Ascending,  // 最小 = 升序 ← 唯一的区别！
    })
    .with_operation(BaseOperation::LimitFiles {
        count: 1,
    });

let command = plan.to_shell_command();
println!("{}", command);
// → find . -name '*.rs' -type f -exec ls -lh {} + | sort -k5 -h | head -n 1
```

**对比**：
```diff
  最大: sort -k5 -hr  (降序)
  最小: sort -k5 -h   (升序)
         ↑ 只有一个字母的差异！
```

---

### 示例 3：查找最新修改的文件

**扩展**：改变排序字段

```rust
let plan = ExecutionPlan::new()
    .with_operation(BaseOperation::FindFiles {
        path: ".".to_string(),
        pattern: "*.md".to_string(),
    })
    .with_operation(BaseOperation::SortFiles {
        field: Field::Time,  // 改变字段：Size → Time
        direction: Direction::Descending,
    })
    .with_operation(BaseOperation::LimitFiles {
        count: 5,
    });

// → find . -name '*.md' -type f -exec ls -lh {} + | sort -k6 -hr | head -n 5
```

---

### 示例 4：查找前10个最大的 Python 文件

**扩展**：改变文件类型和数量

```rust
let plan = ExecutionPlan::new()
    .with_operation(BaseOperation::FindFiles {
        path: "./src".to_string(),
        pattern: "*.py".to_string(),  // 改变类型
    })
    .with_operation(BaseOperation::SortFiles {
        field: Field::Size,
        direction: Direction::Descending,
    })
    .with_operation(BaseOperation::LimitFiles {
        count: 10,  // 改变数量
    });
```

---

### 示例 5：简单列出目录

**最简**：只有一个操作

```rust
let plan = ExecutionPlan::new()
    .with_operation(BaseOperation::ListFiles {
        path: ".".to_string(),
    });

// → ls -lh .
```

---

## 🧪 测试验证

### 单元测试

```rust
#[test]
fn test_smallest_vs_largest() {
    // 最大
    let largest = ExecutionPlan::new()
        .with_operation(BaseOperation::FindFiles {
            path: ".".to_string(),
            pattern: "*.rs".to_string(),
        })
        .with_operation(BaseOperation::SortFiles {
            field: Field::Size,
            direction: Direction::Descending,
        })
        .with_operation(BaseOperation::LimitFiles { count: 1 });

    // 最小
    let smallest = ExecutionPlan::new()
        .with_operation(BaseOperation::FindFiles {
            path: ".".to_string(),
            pattern: "*.rs".to_string(),
        })
        .with_operation(BaseOperation::SortFiles {
            field: Field::Size,
            direction: Direction::Ascending,  // 唯一区别
        })
        .with_operation(BaseOperation::LimitFiles { count: 1 });

    // 验证：结构相同，参数不同
    assert_eq!(largest.len(), smallest.len());
    assert_eq!(largest.len(), 3);

    // 但生成的命令不同
    let cmd_largest = largest.to_shell_command();
    let cmd_smallest = smallest.to_shell_command();

    assert!(cmd_largest.contains("-hr"));  // 降序
    assert!(cmd_smallest.contains("-h"));  // 升序
    assert!(!cmd_smallest.contains("-hr"));
}
```

**运行结果**：
```bash
$ cargo test --lib dsl::pipeline
test result: ok. 17 passed; 0 failed; 0 ignored
```

✅ 所有测试通过！

---

## 🌟 哲学体现

### 易经映射

| 易经概念 | Pipeline DSL | 示例 |
|----------|-------------|------|
| **象**（不变） | BaseOperation | FindFiles, SortFiles |
| **爻**（变化） | Parameters | Direction, Field, Count |
| **卦**（组合） | ExecutionPlan | operations 的顺序组合 |
| **变化规律** | to_shell_command() | 参数→命令的转换规则 |

### 从"数"到"变"

**错误思维**（数）：
```
Intent1, Intent2, Intent3, ..., IntentN
(枚举所有状态)
```

**正确思维**（变）：
```
BaseOperation × Parameters = ∞ 种组合
(定义变化规律，动态生成)
```

### 核心价值

1. **象不变，爻可变**
   - 查找文件这个操作不变
   - 排序方向可以从"降序"变为"升序"

2. **操作可组合**
   - Find + Sort = 有序查找
   - Find + Sort + Limit = 前N个
   - 组合产生新的语义

3. **规律可学习**
   - 当前：手工定义规则
   - 未来：LLM 学习规律，生成 ExecutionPlan

---

## 📊 性能对比

### 代码量对比

**传统方式**（枚举）：
- 支持 4 种查询：最大、最小、最新、最旧
- 代码量：~400 行（4×100 行/Intent）

**Pipeline DSL**（组合）：
- 支持无限组合
- 代码量：~200 行（基础操作定义）
- **节省 50% 代码**

### 可扩展性对比

| 需求 | 传统方式 | Pipeline DSL |
|------|---------|-------------|
| 添加"最小" | 新增完整 Intent | 改变1个参数 |
| 添加"按名称排序" | 新增完整 Intent | 改变1个 Field |
| 添加"前20个" | 修改模板 | 改变1个 Count |
| 组合查询 | ❌ 无法实现 | ✅ 自由组合 |

---

## 🔮 未来方向

### Phase 6.3 完整实现

当前原型已验证可行，下一步：

1. **集成到 Intent 匹配**
   ```rust
   fn match_intent(input: &str) -> ExecutionPlan {
       // 用户输入 → 解析 → ExecutionPlan
   }
   ```

2. **更多基础操作**
   - FilterFiles: 按条件过滤
   - TransformFiles: 文件转换
   - AggregateFiles: 聚合统计

3. **验证和安全**
   - Plan 验证（validate方法已实现）
   - 黑名单检查
   - 权限控制

### Phase 7: LLM 驱动

**终极目标**：
```
用户输入 → LLM 理解 → 生成 ExecutionPlan JSON → 验证 → 执行
```

**示例**：
```
用户：显示当前目录下体积最小的rs文件

LLM 输出：
{
  "operations": [
    {"FindFiles": {"path": ".", "pattern": "*.rs"}},
    {"SortFiles": {"field": "Size", "direction": "Ascending"}},
    {"LimitFiles": {"count": 1}}
  ]
}

系统：解析 → ExecutionPlan → to_shell_command() → 执行
```

---

## ✅ 成功验证

### 原型目标

- ✅ 证明组合式架构可行
- ✅ 解决"最大/最小"问题
- ✅ 体现"象+爻"哲学
- ✅ 所有测试通过（17/17）

### 核心收获

1. **架构清晰**：BaseOperation + ExecutionPlan 模式简洁有效
2. **易于扩展**：添加新操作只需定义一次
3. **哲学落地**："一分为三"不是空谈，在代码中可实践
4. **测试充分**：17个测试覆盖核心场景

### 下一步

- ✅ **A**：Pipeline DSL 原型验证 - 完成
- 🔄 **B**：参数化模板（Phase 6.2.1）- 进行中
- 📅 **C**：LLM 驱动（Phase 7）- 未来

---

**文档版本**: 1.0
**创建日期**: 2025-10-15
**维护者**: RealConsole Team

**核心理念**：
> 不是枚举所有状态，而是定义变化规律。
> 不是"数"（counting），而是"变"（transformation）。
> 象不变，爻可变，卦可组合。✨
