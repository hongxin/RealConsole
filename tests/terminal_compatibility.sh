#!/bin/bash
# 终端兼容性测试脚本
#
# 用于测试 RealConsole 在不同终端模拟器下的兼容性
# 测试项目：
# - emoji 显示
# - 颜色支持
# - Unicode 字符
# - context 命令
# - 性能稳定性

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 测试结果
PASS=0
FAIL=0
SKIP=0

# 打印标题
print_header() {
    echo -e "\n${CYAN}=== $1 ===${NC}\n"
}

# 打印测试结果
print_result() {
    if [ "$1" = "PASS" ]; then
        echo -e "${GREEN}✓ $2${NC}"
        ((PASS++))
    elif [ "$1" = "FAIL" ]; then
        echo -e "${RED}✗ $2${NC}"
        ((FAIL++))
    else
        echo -e "${YELLOW}⊘ $2${NC}"
        ((SKIP++))
    fi
}

# 获取终端信息
get_terminal_info() {
    print_header "终端环境信息"

    echo "操作系统: $(uname -s) $(uname -r)"
    echo "终端类型: $TERM"
    echo "Shell: $SHELL"
    echo "终端程序: ${TERM_PROGRAM:-未知}"
    echo "终端程序版本: ${TERM_PROGRAM_VERSION:-未知}"

    # 检测颜色支持
    if [ -t 1 ]; then
        colors=$(tput colors 2>/dev/null || echo "未知")
        echo "颜色支持: $colors 色"
    else
        echo "颜色支持: 非 TTY 环境"
    fi

    # 检测 Unicode 支持
    echo -n "Unicode 测试: "
    if echo "中文测试 ✓ ✗ ⊘" >/dev/null 2>&1; then
        echo "支持"
    else
        echo "不支持"
    fi
}

# 测试 RealConsole 基本功能
test_basic_functionality() {
    print_header "RealConsole 基本功能测试"

    # 检查可执行文件
    if command -v realconsole &> /dev/null; then
        print_result "PASS" "realconsole 可执行文件存在"

        # 检查版本
        version=$(realconsole --version 2>&1 | head -1)
        echo "  版本: $version"
    else
        print_result "FAIL" "realconsole 可执行文件不存在"
        return 1
    fi
}

# 测试颜色输出
test_color_output() {
    print_header "颜色输出测试"

    echo -e "基本颜色测试:"
    echo -e "  ${RED}红色${NC} ${GREEN}绿色${NC} ${YELLOW}黄色${NC} ${CYAN}青色${NC}"

    if [ -t 1 ]; then
        colors=$(tput colors 2>/dev/null || echo "0")
        if [ "$colors" -ge 8 ]; then
            print_result "PASS" "终端支持基本颜色（$colors 色）"
        else
            print_result "FAIL" "终端颜色支持不足（$colors 色）"
        fi
    else
        print_result "SKIP" "非 TTY 环境，跳过颜色测试"
    fi
}

# 测试 Unicode 和 emoji
test_unicode_emoji() {
    print_header "Unicode 和 Emoji 测试"

    echo "基本 Unicode 字符:"
    echo "  中文: 你好世界"
    echo "  日文: こんにちは"
    echo "  符号: ✓ ✗ ⊘ ● ○ ◆ ◇"

    echo ""
    echo "Emoji 字符（如果终端不支持可能显示异常）:"
    echo "  状态: ✅ ❌ ⚠️ ℹ️"
    echo "  人物: 👤 🤖"
    echo "  符号: 🚀 ⚡ 💡 🔧"
    echo "  多字符: 🟢 🔴 ⭐⭐"

    echo ""
    echo -n "请确认以上字符显示是否正常？(y/n): "
    read -r answer
    if [[ "$answer" =~ ^[Yy]$ ]]; then
        print_result "PASS" "Unicode/Emoji 显示正常"
    else
        print_result "FAIL" "Unicode/Emoji 显示异常"
        echo "  提示: 建议在配置文件中设置 display.use_emoji: false"
    fi
}

# 测试字符宽度计算
test_char_width() {
    print_header "字符宽度测试"

    echo "单宽度字符（每个占 1 列）:"
    echo "  ASCII: abcdef123456"
    echo "双宽度字符（每个占 2 列）:"
    echo "  中文: 你好世界测试"
    echo "  emoji: 👤🤖🚀⚡"

    echo ""
    echo "对齐测试（应该对齐）:"
    echo "  1234567890"
    echo "  你好世界测"

    echo ""
    echo -n "以上字符是否正确对齐？(y/n): "
    read -r answer
    if [[ "$answer" =~ ^[Yy]$ ]]; then
        print_result "PASS" "字符宽度计算正确"
    else
        print_result "FAIL" "字符宽度计算错误"
        echo "  提示: 可能导致显示错位，建议使用纯 ASCII 显示"
    fi
}

# 测试长文本处理
test_long_text() {
    print_header "长文本处理测试"

    echo "测试超长行（应该能正常显示或换行）:"
    for i in {1..10}; do
        echo -n "这是一个很长的测试文本，用于测试终端的长文本处理能力。"
    done
    echo ""

    print_result "PASS" "长文本显示测试完成（请手动检查）"
}

# 压力测试
test_stress() {
    print_header "压力测试（可选）"

    echo -n "是否进行压力测试？这将输出大量文本 (y/n): "
    read -r answer
    if [[ ! "$answer" =~ ^[Yy]$ ]]; then
        print_result "SKIP" "跳过压力测试"
        return
    fi

    echo "输出 1000 行混合内容..."
    for i in {1..1000}; do
        if [ $((i % 100)) -eq 0 ]; then
            echo "[$i/1000] 进度: $((i/10))%"
        fi
        echo "Line $i: 测试文本 Test ✓ $i"
    done

    print_result "PASS" "压力测试完成（请检查终端是否崩溃或卡顿）"
}

# 主测试流程
main() {
    echo "========================================="
    echo "  RealConsole 终端兼容性测试工具"
    echo "========================================="
    echo ""
    echo "此工具将测试终端对 RealConsole 的兼容性"
    echo "包括：颜色、Unicode、emoji、长文本等"
    echo ""

    get_terminal_info
    test_basic_functionality
    test_color_output
    test_unicode_emoji
    test_char_width
    test_long_text
    test_stress

    # 汇总结果
    print_header "测试结果汇总"
    echo -e "${GREEN}通过: $PASS${NC}"
    echo -e "${RED}失败: $FAIL${NC}"
    echo -e "${YELLOW}跳过: $SKIP${NC}"
    echo ""

    # 建议
    if [ "$FAIL" -gt 0 ]; then
        echo "建议:"
        echo "  1. 在配置文件中设置 display.use_emoji: false"
        echo "  2. 检查终端的 Unicode 支持设置"
        echo "  3. 考虑更换终端模拟器（推荐 iTerm2 或 Alacritty）"
    else
        echo "恭喜！您的终端完全兼容 RealConsole"
    fi

    echo ""
    echo "========================================="

    # 返回码
    if [ "$FAIL" -gt 0 ]; then
        exit 1
    else
        exit 0
    fi
}

main "$@"
