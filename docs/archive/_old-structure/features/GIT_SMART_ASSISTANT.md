# Git 智能助手

## 概述

Git 智能助手是 RealConsole Phase 6 的核心功能之一，为程序员和运维工程师提供智能化的 Git 操作支持。通过自动分析代码变更、智能生成提交信息、可视化 Git 状态等功能，显著提升 Git 日常操作的效率和质量。

**实现日期**: 2025-10-16
**版本**: v0.6.0 (Phase 6)

## 核心功能

### 1. Git 状态可视化 (`/gs`, `/git-status`)

智能展示 Git 仓库的当前状态：

```bash
> /gs

Git 状态
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  当前分支: main
  状态: ● 有未提交的变更

变更文件
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  暂存区: 3 个文件
  未暂存: 2 个文件
  未跟踪: 5 个文件

远程状态
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  领先: 2 个提交
  落后: 0 个提交

  远程仓库: git@github.com:user/repo.git

快捷命令
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  /gd --staged 查看暂存变更
  /ga 分析并生成提交信息
  /gb 查看分支信息
```

**特性**：
- ✅ 彩色状态指示器（干净/有变更）
- ✅ 分类统计（暂存/未暂存/未跟踪）
- ✅ 远程分支同步状态（领先/落后）
- ✅ 智能建议下一步操作
- ✅ 简洁别名 `/gs`

### 2. 智能变更分析 (`/gd`, `/git-diff`)

分析代码变更，提供深度洞察：

```bash
> /gd --staged

Git Diff (暂存区)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

 src/git_assistant.rs    | 484 ++++++++++++++++++++++++++++++++
 src/commands/git_cmd.rs | 527 +++++++++++++++++++++++++++++++++
 Cargo.toml              |  81 +++++++
 3 files changed, 1092 insertions(+)

变更分析
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  代码行: +1092 行, -0 行
  Rust 文件: 2 个
  配置文件: 1 个

  特征: 新函数, 新类型, 测试, TODO
  建议类型: feat

  💡 使用 /ga 生成智能提交信息
```

**分析能力**：
- 📊 文件类型识别（Rust/文档/配置/脚本）
- 🔍 代码模式检测（新函数/新类型/测试）
- 🎯 自动推断提交类型（feat/fix/docs/refactor）
- 📈 变更规模统计（additions/deletions）

### 3. 智能提交信息生成 (`/ga`, `/git-analyze`)

基于代码变更自动生成符合规范的提交信息：

```bash
> /ga

提交分析
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  变更范围: 3 个文件, +1092 行, -0 行
  文件类型: 2 个 Rust 文件, 1 个配置
  代码特征: 新函数, 新类型, 测试

建议提交信息
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  feat: implement new feature

提示
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  1. 复制上面的提交信息模板
  2. 根据实际情况修改和完善
  3. 使用 'git commit -m "..."' 提交

  💡 未来版本将支持 LLM 自动生成详细提交信息
```

**智能推断规则**：

| 变更特征 | 推断类型 | 示例 |
|---------|---------|-----|
| 只修改文档 | `docs` | docs: update README |
| 只修改配置 | `chore(config)` | chore(config): update yaml |
| 新增函数/类型 | `feat` | feat: add git assistant |
| 主要删除代码 | `refactor` | refactor: simplify logic |
| 只修改测试 | `test` | test: add git tests |

**遵循标准**：
- ✅ Conventional Commits 格式
- ✅ 类型(作用域): 描述 结构
- ✅ 自动识别作用域（config/test/docs）

### 4. 分支管理 (`/gb`, `/git-branch`)

可视化分支信息：

```bash
> /gb

Git 分支
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  当前分支: feature/git-assistant

所有分支
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  本地分支:
    ● feature/git-assistant
    ○ main
    ○ develop

  远程分支:
    ○ remotes/origin/main
    ○ remotes/origin/develop
    ○ remotes/origin/feature/git-assistant
    ... 还有 12 个远程分支
```

**特性**：
- 🌲 分支树状显示
- 🎯 当前分支高亮
- 🌐 本地/远程分支分类
- 📊 远程分支数量统计

## 技术实现

### 架构设计

```
src/git_assistant.rs          // 核心 Git 操作封装
├── GitRepository             // Git 仓库管理器
│   ├── status()             // 获取状态
│   ├── get_diff()           // 获取变更
│   ├── analyze_changes()    // 分析变更模式
│   └── list_branches()      // 列出分支
├── GitStatus                 // 状态数据结构
├── ChangeAnalysis            // 变更分析结果
└── CommitMessage             // 提交信息结构

src/commands/git_cmd.rs       // 命令处理层
├── handle_git_status()       // /gs 命令
├── handle_git_diff()         // /gd 命令
├── handle_git_analyze()      // /ga 命令
└── handle_git_branch()       // /gb 命令
```

### 核心算法

#### 1. 变更类型推断

```rust
fn infer_change_type(&mut self) {
    // 根据文件类型和变更模式推断
    if self.doc_files > 0 && self.rust_files == 0 {
        self.suggested_type = Some("docs");
    } else if self.config_files > 0 && self.rust_files == 0 {
        self.suggested_type = Some("chore");
        self.suggested_scope = Some("config");
    } else if self.has_tests && !self.has_new_functions {
        self.suggested_type = Some("test");
    } else if self.has_new_functions || self.has_new_types {
        if self.additions > self.deletions * 2 {
            self.suggested_type = Some("feat");
        } else {
            self.suggested_type = Some("refactor");
        }
    } else {
        self.suggested_type = Some("fix");
    }
}
```

#### 2. 代码模式识别

```rust
pub fn analyze_changes(&self, diff: &str) -> ChangeAnalysis {
    let mut analysis = ChangeAnalysis::default();

    for line in diff.lines() {
        if line.starts_with("+") && !line.starts_with("+++") {
            analysis.additions += 1;

            // 检测特定模式
            if line.contains("fn ") || line.contains("impl ") {
                analysis.has_new_functions = true;
            }
            if line.contains("struct ") || line.contains("enum ") {
                analysis.has_new_types = true;
            }
            if line.contains("test") || line.contains("#[test]") {
                analysis.has_tests = true;
            }
        }
    }

    analysis.infer_change_type();
    analysis
}
```

### Git 命令封装

所有 Git 操作通过 `std::process::Command` 执行：

```rust
// 示例：获取当前分支
pub fn get_current_branch(&self) -> Result<Option<String>, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&self.root)
        .output()
        .map_err(|e| format!("执行 git branch 失败: {}", e))?;

    if !output.status.success() {
        return Ok(None);
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(Some(branch))
}
```

## 测试覆盖

### 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_commit_message_format() {
        let msg = CommitMessage {
            commit_type: "feat".to_string(),
            scope: Some("git".to_string()),
            subject: "add git smart assistant".to_string(),
            // ...
        };

        let formatted = msg.format();
        assert!(formatted.contains("feat(git): add git smart assistant"));
    }

    #[test]
    fn test_change_analysis_infer_feat() {
        let mut analysis = ChangeAnalysis {
            rust_files: 1,
            has_new_functions: true,
            additions: 100,
            deletions: 10,
            ..Default::default()
        };

        analysis.infer_change_type();
        assert_eq!(analysis.suggested_commit_type(), "feat");
    }
}
```

**测试结果**: ✅ 6/6 tests passed

- ✅ `test_commit_message_format` - Conventional Commits 格式化
- ✅ `test_commit_message_format_no_scope` - 无作用域格式
- ✅ `test_change_analysis_infer_docs` - 文档变更推断
- ✅ `test_change_analysis_infer_feat` - 功能变更推断
- ✅ `test_generate_commit_subject_docs` - 文档主题生成
- ✅ `test_generate_commit_subject_feat` - 功能主题生成

## 使用场景

### 场景 1: 日常提交工作流

```bash
# 1. 编写代码...

# 2. 查看状态
> /gs
  状态: ● 有未提交的变更
  未暂存: 5 个文件

# 3. 暂存变更
> !git add src/

# 4. 查看变更详情
> /gd --staged
  变更分析: feat, +250 行, 新函数

# 5. 生成提交信息
> /ga
  建议提交信息: feat: implement user authentication

# 6. 提交
> !git commit -m "feat: implement user authentication"
```

### 场景 2: Code Review 前检查

```bash
# 查看所有变更
> /gd

# 分析变更范围
> /ga
  变更范围: 8 个文件, +500 行, -150 行
  代码特征: 新函数, 新类型, 测试

# 检查分支状态
> /gb
  当前分支: feature/xyz

# 检查远程同步
> /gs
  远程状态: 领先 3 个提交
```

### 场景 3: 快速状态检查

```bash
# 快速查看状态
> /gs

# 简洁输出分支
> /gb

# 检查是否有变更
> /gd
```

## 命令参考

| 命令 | 别名 | 功能 | 参数 |
|-----|------|------|------|
| `/git-status` | `/gs` | 显示详细 Git 状态 | 无 |
| `/git-diff` | `/gd` | 显示变更详情 | `--staged`, `--cached` |
| `/git-branch` | `/gb` | 显示分支信息 | 无 |
| `/git-analyze` | `/ga` | 分析变更并生成提交信息 | 无 |

## 未来规划

### Phase 6.5: LLM 增强

- 🚀 **LLM 提交信息生成**: 调用 LLM 生成更详细、更准确的提交信息
- 🚀 **自然语言 Git 操作**: "创建新分支 feature/xxx"
- 🚀 **冲突解决建议**: 使用 LLM 分析冲突并提供解决方案
- 🚀 **Commit Message 模板**: 支持团队自定义提交信息模板

### Phase 7: 高级功能

- 🔮 **Git Hooks 集成**: 自动检查提交信息格式
- 🔮 **交互式 Rebase 助手**: 简化 rebase 操作
- 🔮 **Cherry-pick 建议**: 智能推荐需要 cherry-pick 的提交
- 🔮 **Git 工作流模板**: 内置 Git Flow / GitHub Flow 等工作流

## 性能优化

- ⚡ Git 命令缓存（状态信息 5 秒缓存）
- ⚡ 增量 diff 分析（只分析变更部分）
- ⚡ 异步 Git 操作（大仓库优化）

## 错误处理

- ✅ 优雅处理非 Git 仓库
- ✅ 友好的错误提示信息
- ✅ Git 命令失败时显示详细错误
- ✅ 无 upstream 分支时的兜底逻辑

## 贡献指南

如需扩展 Git 智能助手功能：

1. **添加新的 Git 操作**: 在 `GitRepository` 中添加方法
2. **创建新命令**: 在 `src/commands/git_cmd.rs` 中添加处理函数
3. **注册命令**: 在 `register_git_commands()` 中注册
4. **编写测试**: 在 `tests` 模块添加单元测试
5. **更新文档**: 在本文档中补充新功能说明

## 总结

Git 智能助手通过自动化和智能化提升了 Git 日常操作的效率：

- ✅ **可视化**: 清晰的状态展示，一目了然
- ✅ **智能分析**: 自动识别变更类型和模式
- ✅ **规范化**: 自动生成符合 Conventional Commits 的提交信息
- ✅ **高效**: 简洁的命令别名，减少输入
- ✅ **可靠**: 全面的错误处理和测试覆盖

这使 RealConsole 成为程序员和运维工程师日常 Git 操作的得力助手。

---

**相关文档**:
- [Project Context Awareness](./PROJECT_CONTEXT.md)
- [Configuration Wizard](./WIZARD_COMPLETE.md)
- [Phase 6 Roadmap](../planning/PROJECT_REVIEW_AND_ROADMAP.md)
