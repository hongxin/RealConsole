# v1.28.0 Git 提交指南

本文档提供 v1.28.0 版本的 Git 提交操作步骤。

---

## 📋 提交前检查清单

- [x] 代码编译成功（`cargo build --release`）
- [x] 功能测试通过
- [x] Bug 已修复
- [x] 版本号已更新（1.27.0 → 1.28.0）
- [x] 文档已完成（6 个文档，2,500+ 行）
- [x] 提交信息已准备（`COMMIT_MESSAGE.md`）

---

## 🔍 变更概览

### 代码变更
```
Cargo.toml           |   2 +-
src/web/server.rs    | 651 +++++++++++++++++++++++++++++++++++
src/web/session.rs   | 175 ++++++++++
src/web/websocket.rs |  82 +++++
4 files changed, 899 insertions(+), 11 deletions(-)
```

### 新增文档
```
docs/04-reports/v1.28.0-bugfix-report.md
docs/04-reports/v1.28.0-implementation-plan.md
docs/04-reports/v1.28.0-release-notes.md
docs/04-reports/v1.28.0-testing-guide.md
docs/04-reports/v1.28.0-ux-improvements.md
docs/04-reports/v1.28.0-view-mode-toggle.md
```

---

## 📝 Git 操作步骤

### 步骤 1: 添加所有变更

```bash
# 添加修改的代码文件
git add Cargo.toml
git add src/web/server.rs
git add src/web/session.rs
git add src/web/websocket.rs

# 添加新增的文档
git add docs/04-reports/v1.28.0-*.md
```

或者一次性添加所有变更：
```bash
git add Cargo.toml src/web/ docs/04-reports/v1.28.0-*.md
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
	new file:   docs/04-reports/v1.28.0-bugfix-report.md
	new file:   docs/04-reports/v1.28.0-implementation-plan.md
	new file:   docs/04-reports/v1.28.0-release-notes.md
	new file:   docs/04-reports/v1.28.0-testing-guide.md
	new file:   docs/04-reports/v1.28.0-ux-improvements.md
	new file:   docs/04-reports/v1.28.0-view-mode-toggle.md
	modified:   src/web/server.rs
	modified:   src/web/session.rs
	modified:   src/web/websocket.rs
```

### 步骤 3: 创建提交

**使用准备好的提交信息**：
```bash
git commit -F COMMIT_MESSAGE.md
```

**或者手动输入**：
```bash
git commit -m "$(cat COMMIT_MESSAGE.md)"
```

**或者使用编辑器**：
```bash
git commit
# 然后粘贴 COMMIT_MESSAGE.md 的内容
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
git tag v1.28.0
git push origin main --tags
```

---

## 📦 完整的一键提交脚本

如果你想快速提交，可以运行：

```bash
#!/bin/bash
# v1.28.0-commit.sh

# 添加所有变更
git add Cargo.toml src/web/ docs/04-reports/v1.28.0-*.md

# 查看状态
echo "=== Git Status ==="
git status

# 确认提交
read -p "继续提交？(y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # 提交
    git commit -F COMMIT_MESSAGE.md

    # 查看提交
    echo ""
    echo "=== 提交完成 ==="
    git log -1 --stat

    echo ""
    echo "下一步：运行 'git push origin main' 推送到远程"
else
    echo "提交已取消"
fi
```

---

## 🔖 创建版本标签（可选）

```bash
# 创建带注释的标签
git tag -a v1.28.0 -m "v1.28.0 - 对话回合可视化

核心功能：
- Jupyter-like 对话回合卡片
- 双视图模式切换（回合/传统）
- 执行反馈优化（飞轮动画）

代码统计：
- +899 行代码
- 6 个文档（2,500+ 行）
- 4 个 Bug 修复

详见：docs/04-reports/v1.28.0-release-notes.md"

# 查看标签
git tag -l -n9 v1.28.0

# 推送标签
git push origin v1.28.0
```

---

## 🧹 清理辅助文件（提交后）

提交完成后，可以清理这些辅助文件：

```bash
# 删除提交信息和指南（可选）
rm COMMIT_MESSAGE.md GIT_COMMIT_GUIDE.md
```

---

## ⚠️ 注意事项

### 提交前最后检查

1. **确保代码能编译**:
   ```bash
   cargo build --release
   ```

2. **确保测试通过**:
   ```bash
   cargo test
   ```

3. **确保功能正常**:
   ```bash
   export DEEPSEEK_API_KEY="your-api-key"
   ./target/release/realconsole web
   # 在浏览器中测试
   ```

4. **检查文档完整性**:
   ```bash
   ls -lh docs/04-reports/v1.28.0-*.md
   ```

### 提交后操作

1. **验证远程仓库**:
   - 访问 GitHub 检查提交是否正确
   - 检查文件变更
   - 检查提交消息格式

2. **创建 GitHub Release**（可选）:
   - 访问仓库的 Releases 页面
   - 点击 "Create a new release"
   - 选择标签 v1.28.0
   - 复制 `docs/04-reports/v1.28.0-release-notes.md` 的内容作为 Release Notes

3. **通知团队**:
   - 更新 README.md（如需要）
   - 发送版本发布通知

---

## 📊 提交统计摘要

### 代码
- **文件数**: 4 个
- **新增行**: +899
- **删除行**: -11
- **净增**: +888

### 文档
- **文件数**: 6 个
- **总行数**: ~2,500

### 功能
- **核心功能**: 3 个
- **Bug 修复**: 4 个
- **性能优化**: 2 个

### 测试
- **手动测试**: 通过
- **自动化测试**: 待添加（v1.29.0）

---

## 🎯 下一步计划

提交完成后，可以开始规划 v1.29.0：
- 回合操作增强（删除、重执行、导出）
- 快捷键支持（Shift+Enter, Ctrl+/）
- 历史搜索功能
- 视图偏好持久化

详见：`docs/03-evolution/v1-to-v2-transition-plan.md`

---

**准备日期**: 2025-11-07
**版本**: v1.28.0
**准备者**: Claude Code AI Assistant
