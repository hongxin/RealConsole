#!/bin/bash
# 演示 Ctrl+R 历史搜索功能

echo "=== Ctrl+R 历史搜索功能演示 ==="
echo ""

# 清理旧历史
rm -f ~/.realconsole/history.json

echo "步骤 1: 创建一些有意义的历史命令..."
./target/release/realconsole --once "!git status"
./target/release/realconsole --once "!git log --oneline"
./target/release/realconsole --once "!cargo build"
./target/release/realconsole --once "!cargo test"
./target/release/realconsole --once "!ls -la src/"
./target/release/realconsole --once "!find . -name '*.rs'"
./target/release/realconsole --once "!grep -r 'history' src/"

echo ""
echo "步骤 2: 验证历史记录已保存..."
./target/release/realconsole --once "/history"

echo ""
echo "步骤 3: 验证历史统计..."
./target/release/realconsole --once "/history stats"

echo ""
echo "=========================================="
echo "✨ Ctrl+R 交互式搜索使用指南"
echo "=========================================="
echo ""
echo "启动 RealConsole REPL:"
echo "  ./target/release/realconsole"
echo ""
echo "使用 Ctrl+R 搜索:"
echo "  1. 按 Ctrl+R 进入反向搜索模式"
echo "  2. 输入搜索关键词（如 'git', 'cargo', 'test'）"
echo "  3. rustyline 会实时显示匹配的历史命令"
echo "  4. 再按 Ctrl+R 可以循环查看更多匹配项"
echo "  5. 按 Enter 执行当前命令"
echo "  6. 按 Ctrl+G 或 Esc 取消搜索"
echo ""
echo "其他历史导航:"
echo "  - ↑/↓: 浏览历史命令"
echo "  - Ctrl+P/Ctrl+N: 上一个/下一个历史命令"
echo ""
echo "✓ 历史记录会在会话间持久化"
echo "✓ 支持智能排序（频率 + 时间）"
echo "✓ 重复命令会自动合并并增加计数"
echo ""
