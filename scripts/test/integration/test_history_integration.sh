#!/bin/bash
# 测试历史记录和 Ctrl+R 功能集成

echo "=== 测试历史记录功能 ==="
echo ""

# 清理旧的历史文件
rm -f ~/.realconsole/history.json

echo "1. 执行几条命令创建历史..."
./target/release/realconsole --once "!echo 'first command'"
./target/release/realconsole --once "!ls -la"
./target/release/realconsole --once "!pwd"
./target/release/realconsole --once "!echo 'search me'"
./target/release/realconsole --once "!date"

echo ""
echo "2. 查看历史记录..."
./target/release/realconsole --once "/history"

echo ""
echo "3. 测试搜索功能..."
./target/release/realconsole --once "/history search echo"

echo ""
echo "4. 检查历史文件..."
if [ -f ~/.realconsole/history.json ]; then
    echo "✓ 历史文件已创建: ~/.realconsole/history.json"
    echo "历史文件内容预览:"
    head -20 ~/.realconsole/history.json
else
    echo "✗ 历史文件不存在"
fi

echo ""
echo "=== 手动测试说明 ==="
echo ""
echo "现在可以手动测试 Ctrl+R 交互式搜索："
echo "1. 运行: ./target/release/realconsole"
echo "2. 按 Ctrl+R 开始反向搜索"
echo "3. 输入 'echo' 应该会找到 'echo' 相关的历史命令"
echo "4. 再按 Ctrl+R 可以找到下一个匹配项"
echo "5. 按 Enter 执行，或 Ctrl+G 取消"
echo ""
echo "✨ 注意: rustyline 的 Ctrl+R 是内置功能，应该能够搜索所有加载的历史记录"
