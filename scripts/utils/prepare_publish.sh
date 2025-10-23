#!/bin/bash
# RealConsole 发布准备脚本
# 用途：创建一个干净的发布目录，排除敏感信息和临时文件

set -e  # 遇到错误立即退出

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== RealConsole v1.0.0 发布准备脚本 ===${NC}"
echo ""

# 工作目录
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PUBLISH_DIR="$PROJECT_ROOT/publish"

echo "项目根目录: $PROJECT_ROOT"
echo "发布目录: $PUBLISH_DIR"
echo ""

# 清理旧的发布目录
if [ -d "$PUBLISH_DIR" ]; then
    echo -e "${YELLOW}清理旧的发布目录...${NC}"
    rm -rf "$PUBLISH_DIR"
fi

# 创建发布目录结构
echo -e "${GREEN}创建发布目录结构...${NC}"
mkdir -p "$PUBLISH_DIR"

# 复制源代码
echo "复制源代码..."
cp -r "$PROJECT_ROOT/src" "$PUBLISH_DIR/"

# 复制测试
echo "复制测试..."
cp -r "$PROJECT_ROOT/tests" "$PUBLISH_DIR/"

# 复制性能测试
echo "复制性能测试..."
cp -r "$PROJECT_ROOT/benches" "$PUBLISH_DIR/"

# 复制文档
echo "复制文档..."
cp -r "$PROJECT_ROOT/docs" "$PUBLISH_DIR/"

# 复制示例
echo "复制示例..."
cp -r "$PROJECT_ROOT/examples" "$PUBLISH_DIR/"

# 复制配置示例
echo "复制配置示例..."
cp -r "$PROJECT_ROOT/config" "$PUBLISH_DIR/"

# 复制脚本（排除本脚本自己）
echo "复制脚本..."
mkdir -p "$PUBLISH_DIR/scripts"
cp -r "$PROJECT_ROOT/scripts"/*.sh "$PUBLISH_DIR/scripts/" 2>/dev/null || true

# 复制项目配置文件
echo "复制项目配置文件..."
cp "$PROJECT_ROOT/Cargo.toml" "$PUBLISH_DIR/"
cp "$PROJECT_ROOT/Cargo.lock" "$PUBLISH_DIR/"

# 复制说明文档
echo "复制说明文档..."
cp "$PROJECT_ROOT/README.md" "$PUBLISH_DIR/"
cp "$PROJECT_ROOT/CLAUDE.md" "$PUBLISH_DIR/"
cp "$PROJECT_ROOT/PROJECT_STRUCTURE.md" "$PUBLISH_DIR/"
cp "$PROJECT_ROOT/RELEASE_CHECKLIST.md" "$PUBLISH_DIR/"

# 复制配置文件示例（不包含 .env）
echo "复制配置文件示例..."
cp "$PROJECT_ROOT/.env.example" "$PUBLISH_DIR/"
cp "$PROJECT_ROOT/realconsole.yaml" "$PUBLISH_DIR/"
cp "$PROJECT_ROOT/realconsole-r.yaml" "$PUBLISH_DIR/"

# 复制许可证（如果存在）
if [ -f "$PROJECT_ROOT/LICENSE" ]; then
    echo "复制许可证..."
    cp "$PROJECT_ROOT/LICENSE" "$PUBLISH_DIR/"
fi

# 创建 .gitignore
echo "创建 .gitignore..."
cat > "$PUBLISH_DIR/.gitignore" << 'EOF'
# Rust 编译产物
/target/
Cargo.lock
**/*.rs.bk
*.pdb

# IDE 配置
.vscode/
.idea/
*.swp
*.swo
*~
.DS_Store

# 密钥和敏感信息
.env
*.key
*.pem
*.p12
id_rsa*
*.secret

# 本地配置
*.local.yaml
config.local.yaml

# 开发工件
/coverage/
/flamegraph/
/memory/
/sandbox/
/testbed/

# 日志文件
*.log
logs/

# 临时文件
*.tmp
*.temp
test.txt
temp/

# macOS 系统文件
.DS_Store
.AppleDouble
.LSOverride

# Windows 系统文件
Thumbs.db
ehthumbs.db
Desktop.ini

# 性能分析
perf.data*
*.svg
*.prof

# Claude 配置（私有）
.claude/

# 备份文件
*.bak
*.backup
*~

# 本地数据库
*.db
*.sqlite
*.sqlite3

# Node modules（如果有）
node_modules/

# Python（如果有）
__pycache__/
*.py[cod]
venv/
.venv/
EOF

# 创建 LICENSE 文件（如果不存在）
if [ ! -f "$PUBLISH_DIR/LICENSE" ]; then
    echo "创建 LICENSE 文件..."
    cat > "$PUBLISH_DIR/LICENSE" << 'EOF'
MIT License

Copyright (c) 2025 RealConsole Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
fi

# 创建 PUBLISH_README.md 说明文件
echo "创建发布说明..."
cat > "$PUBLISH_DIR/PUBLISH_README.md" << 'EOF'
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
EOF

# 清理可能的敏感文件
echo ""
echo -e "${YELLOW}清理敏感文件...${NC}"
find "$PUBLISH_DIR" -name ".env" -type f -delete 2>/dev/null || true
find "$PUBLISH_DIR" -name "*.key" -type f -delete 2>/dev/null || true
find "$PUBLISH_DIR" -name "*.secret" -type f -delete 2>/dev/null || true
find "$PUBLISH_DIR" -name ".DS_Store" -type f -delete 2>/dev/null || true

# 统计信息
echo ""
echo -e "${GREEN}=== 发布准备完成 ===${NC}"
echo ""
echo "📊 统计信息："
echo "  源代码文件: $(find "$PUBLISH_DIR/src" -name "*.rs" | wc -l | tr -d ' ')"
echo "  测试文件: $(find "$PUBLISH_DIR/tests" -name "*.rs" | wc -l | tr -d ' ')"
echo "  文档文件: $(find "$PUBLISH_DIR/docs" -name "*.md" | wc -l | tr -d ' ')"
echo "  总大小: $(du -sh "$PUBLISH_DIR" | cut -f1)"
echo ""
echo "✅ 发布目录已创建: $PUBLISH_DIR"
echo ""
echo -e "${YELLOW}⚠️  发布前请务必检查：${NC}"
echo "  1. cd publish/"
echo "  2. grep -r 'DEEPSEEK_API_KEY\\|sk-\\|password\\|secret' . --exclude-dir=.git"
echo "  3. 查看 PUBLISH_README.md 了解详细说明"
echo ""
