# RealConsole 发布目录

此目录包含 RealConsole v1.0.0 的公开发布版本。

## 📦 目录结构

```
publish/
├── src/                    # 源代码
├── tests/                  # 测试文件
├── benches/                # 性能测试
├── docs/                   # 完整文档
├── examples/               # 使用示例
├── config/                 # 配置示例
├── scripts/                # 实用脚本
├── Cargo.toml              # 项目配置
├── Cargo.lock              # 依赖锁定
├── README.md               # 项目说明
├── CLAUDE.md               # 项目指南
├── LICENSE                 # MIT 许可证
├── .env.example            # 环境变量示例
├── .gitignore              # Git 忽略规则
└── realconsole.yaml        # 配置文件示例
```

## 🔒 安全说明

此发布目录已自动排除以下内容：
- ❌ `.env` - 真实的环境变量和密钥
- ❌ `target/` - 编译产物
- ❌ `.git/` - Git 历史记录
- ❌ `coverage/`, `flamegraph/`, `memory/`, `sandbox/` - 开发工件
- ❌ `.claude/` - Claude 私有配置
- ❌ 所有临时文件和系统文件

## ✅ 发布前检查

在推送到公开仓库前，请确保：

1. **检查敏感信息**：
   ```bash
   # 搜索可能的密钥
   grep -r "DEEPSEEK_API_KEY\|sk-\|password\|secret" . --exclude-dir=.git

   # 检查 .env 文件是否被排除
   ls -la | grep ".env$"  # 应该没有输出
   ```

2. **验证 .gitignore**：
   ```bash
   cat .gitignore  # 确认规则完整
   ```

3. **测试构建**：
   ```bash
   cargo build --release
   cargo test
   ```

4. **清理编译产物**（可选）：
   ```bash
   cargo clean
   ```

## 🚀 发布步骤

1. **初始化 Git 仓库**（如果是新仓库）：
   ```bash
   cd publish/
   git init
   git add .
   git commit -m "chore: initial commit for v1.0.0 release"
   ```

2. **添加远程仓库**：
   ```bash
   git remote add origin https://github.com/your-username/realconsole.git
   ```

3. **推送代码**：
   ```bash
   git push -u origin main
   ```

4. **创建发布标签**：
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0 - Task Orchestration System"
   git push origin v1.0.0
   ```

## 📝 注意事项

- ⚠️ 永远不要提交包含真实密钥的 `.env` 文件
- ⚠️ 推送前务必检查 `git status` 确认没有敏感文件
- ⚠️ 使用 `git log` 确认提交历史中没有敏感信息
- ✅ 使用 `.env.example` 作为环境变量模板

## 🔗 相关链接

- **项目主页**: https://github.com/your-username/realconsole
- **问题反馈**: https://github.com/your-username/realconsole/issues
- **文档**: 查看 `docs/README.md`

---

**生成时间**: $(date)
**版本**: v1.0.0
