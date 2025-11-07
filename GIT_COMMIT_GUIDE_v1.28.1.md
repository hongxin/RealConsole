# v1.28.1 Git 提交指南

本文档提供 v1.28.1 版本的 Git 提交操作步骤。

---

## 📋 提交前检查清单

- [x] 代码编译成功（`cargo build --release`）
- [x] 功能测试通过（回合模式 + 传统模式）
- [x] Bug 已修复（3 个关键 bug）
- [x] 版本号已更新（1.28.0 → 1.28.1）
- [x] 文档已完成（发布说明 + 深度复盘）
- [x] 提交信息已准备（`COMMIT_MESSAGE_v1.28.1.md`）

---

## 🔍 变更概览

### 代码变更
```
Cargo.toml           |   2 +-
src/web/session.rs   |   9 +++
src/web/websocket.rs |  80 +++++++++++++++++++
src/web/server.rs    |  30 +++++++
4 files changed, 120 insertions(+), 1 deletion(-)
```

### 新增文档
```
RELEASE_NOTES_v1.28.1.md                       (发布说明)
COMMIT_MESSAGE_v1.28.1.md                      (提交信息)
docs/04-reports/v1.28.1-retrospective.md       (深度复盘，7000+ 字)
```

---

## 📝 Git 操作步骤

### 步骤 1: 添加所有变更

```bash
# 添加修改的代码文件
git add Cargo.toml
git add src/web/session.rs
git add src/web/websocket.rs
git add src/web/server.rs

# 添加新增的文档
git add RELEASE_NOTES_v1.28.1.md
git add COMMIT_MESSAGE_v1.28.1.md
git add docs/04-reports/v1.28.1-retrospective.md
```

或者一次性添加所有变更：
```bash
git add Cargo.toml src/web/ RELEASE_NOTES_v1.28.1.md COMMIT_MESSAGE_v1.28.1.md docs/04-reports/v1.28.1-retrospective.md
```

### 步骤 2: 查看暂存的变更

```bash
git status
```

**预期输出**：
```
On branch main
Your branch is up to date with 'origin/main'.

Changes to be committed:
  (use "git restore --staged <file>..." to unstage)
	modified:   Cargo.toml
	new file:   COMMIT_MESSAGE_v1.28.1.md
	new file:   RELEASE_NOTES_v1.28.1.md
	new file:   docs/04-reports/v1.28.1-retrospective.md
	modified:   src/web/server.rs
	modified:   src/web/session.rs
	modified:   src/web/websocket.rs
```

### 步骤 3: 创建提交

**使用准备好的提交信息**：
```bash
git commit -F COMMIT_MESSAGE_v1.28.1.md
```

**或者手动输入**：
```bash
git commit -m "$(cat COMMIT_MESSAGE_v1.28.1.md)"
```

### 步骤 4: 验证提交

```bash
# 查看最新提交
git log -1

# 查看提交统计
git show --stat
```

### 步骤 5: 推送到远程（你来做）

```bash
# 推送到 main 分支
git push origin main

# 或者推送并创建标签
git tag v1.28.1
git push origin main --tags
```

---

## 🔖 创建版本标签（可选）

```bash
# 创建带注释的标签
git tag -a v1.28.1 -m "v1.28.1 - 统一回合系统完整实施

核心修复：
- Shell/System 命令回合化
- 双视图模式完整兼容
- 修复3个关键 bug

代码统计：
- +120 行代码
- 3 个文档（发布说明 + 深度复盘）
- 3 个 Bug 修复

详见：RELEASE_NOTES_v1.28.1.md"

# 查看标签
git tag -l -n9 v1.28.1

# 推送标签
git push origin v1.28.1
```

---

## 🧹 清理辅助文件（提交后）

提交完成后，可以选择清理这些辅助文件：

```bash
# 删除提交辅助文件（可选）
rm COMMIT_MESSAGE_v1.28.1.md
rm GIT_COMMIT_GUIDE_v1.28.1.md

# 保留发布说明和复盘文档（推荐）
# - RELEASE_NOTES_v1.28.1.md
# - docs/04-reports/v1.28.1-retrospective.md
```

---

## ⚠️ 注意事项

### 提交前最后检查

1. **确保代码能编译**:
   ```bash
   cargo build --release
   ```

2. **确保功能正常**:
   ```bash
   export DEEPSEEK_API_KEY="your-api-key"
   ./target/release/realconsole web
   # 测试回合模式和传统模式
   ```

3. **检查文档完整性**:
   ```bash
   ls -lh RELEASE_NOTES_v1.28.1.md COMMIT_MESSAGE_v1.28.1.md docs/04-reports/v1.28.1-retrospective.md
   ```

### 提交后操作

1. **验证远程仓库**:
   - 访问 GitHub 检查提交是否正确
   - 检查文件变更
   - 检查提交消息格式

2. **创建 GitHub Release**（可选）:
   - 访问仓库的 Releases 页面
   - 点击 "Create a new release"
   - 选择标签 v1.28.1
   - 复制 `RELEASE_NOTES_v1.28.1.md` 的内容作为 Release Notes

---

## 📊 提交统计摘要

### 代码
- **文件数**: 4 个
- **新增行**: +120
- **删除行**: -1
- **净增**: +119

### 文档
- **文件数**: 3 个
- **总行数**: ~8,000+
  - RELEASE_NOTES_v1.28.1.md (380 行)
  - COMMIT_MESSAGE_v1.28.1.md (60 行)
  - v1.28.1-retrospective.md (7,500+ 行，深度复盘)

### 功能
- **核心修复**: 3 个 Bug
- **架构改进**: 统一回合系统
- **设计原则**: 7 条

### 测试
- **手动测试**: 通过
- **测试场景**: 9 个

---

## 🎯 后续计划

提交完成后，可以开始规划 v1.29.0：

- 回合操作增强（删除、重执行、导出）
- 快捷键支持（Shift+Enter, Ctrl+/）
- 历史搜索功能
- 视图偏好持久化

详见：`docs/03-evolution/v1-to-v2-transition-plan.md`

---

**准备日期**: 2025-11-07
**版本**: v1.28.1
**准备者**: Claude Code AI Assistant
