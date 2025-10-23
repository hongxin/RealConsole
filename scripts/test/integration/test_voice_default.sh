#!/bin/bash
# 测试语音播报默认状态
#
# 验证启动时语音播报是关闭的，需要手动开启

set -e

echo "========================================="
echo "  语音播报默认状态测试"
echo "========================================="
echo ""

# 检查可执行文件
if ! command -v realconsole &> /dev/null; then
    echo "❌ realconsole 未安装"
    exit 1
fi

echo "✓ realconsole 已安装"
echo ""

echo "测试说明："
echo "1. 启动时语音播报应该是关闭的"
echo "2. 需要手动执行 /voice on 才能开启"
echo "3. 配置文件中的 voice.enabled 设置会被忽略"
echo ""

# 检查配置文件
CONFIG_FILE="$HOME/.realconsole/realconsole.yaml"
if [ -f "$CONFIG_FILE" ]; then
    echo "检查配置文件: $CONFIG_FILE"

    if grep -q "voice:" "$CONFIG_FILE" 2>/dev/null; then
        echo "  配置文件中包含 voice 配置："
        grep -A 5 "voice:" "$CONFIG_FILE" | sed 's/^/  /'
        echo ""
        echo "  ⚠️  注意: 无论配置如何，启动时都会强制关闭语音"
    else
        echo "  配置文件中没有 voice 配置（使用默认值）"
    fi
else
    echo "配置文件不存在（使用默认值）"
fi

echo ""
echo "========================================="
echo "  手动测试步骤"
echo "========================================="
echo ""
echo "1. 启动 realconsole:"
echo "   $ realconsole"
echo ""
echo "2. 检查语音状态（应该显示 OFF）:"
echo "   > /voice"
echo ""
echo "3. 开启语音播报:"
echo "   > /voice on"
echo ""
echo "4. 再次检查状态（应该显示 ON）:"
echo "   > /voice"
echo ""
echo "5. 测试语音播报（如果支持）:"
echo "   > /voice say 你好，这是测试"
echo ""
echo "6. 关闭语音播报:"
echo "   > /voice off"
echo ""
echo "7. 退出并重启 realconsole，确认语音又是关闭的"
echo ""
echo "========================================="
echo ""

echo "代码验证："
echo "检查 src/agent.rs 中的初始化逻辑..."
if grep -q "enabled: false, // 强制关闭" src/agent.rs 2>/dev/null; then
    echo "✓ 已确认：启动时强制 enabled: false"
else
    echo "❌ 代码可能未正确修改"
    exit 1
fi

echo ""
echo "✓ 所有验证通过！"
echo ""
echo "建议："
echo "  - 配置文件中的 voice.enabled 已无效，启动时强制关闭"
echo "  - 需要时使用 /voice on 手动开启"
echo "  - 重启应用后需要重新开启"
echo ""
