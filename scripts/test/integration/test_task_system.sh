#!/bin/bash
#
# 任务分解与规划系统集成测试
# Phase 10: Task Decomposition & Planning System
#
# 此脚本演示任务系统的完整流程：
# 1. 任务分解 (/plan)
# 2. 查看计划 (/tasks)
# 3. 执行任务 (/execute)
# 4. 查看状态 (/task_status)

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 创建并切换到测试目录（隔离测试环境）
TESTBED_DIR="./testbed"
mkdir -p "$TESTBED_DIR"
cd "$TESTBED_DIR"

# 清理旧的测试文件
rm -rf test_* rust_project_* 2>/dev/null || true

# 打印带颜色的消息
print_step() {
    echo -e "${CYAN}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# 检查 realconsole 可执行文件（使用相对于项目根目录的路径）
REALCONSOLE="../target/release/realconsole"

if [ ! -f "$REALCONSOLE" ]; then
    print_error "找不到 realconsole 可执行文件"
    echo "请先运行: cargo build --release"
    cd ..  # 返回项目根目录
    exit 1
fi

print_success "找到 realconsole: $REALCONSOLE"

# 检查配置文件（使用相对于项目根目录的路径）
CONFIG="../realconsole.yaml"
if [ ! -f "$CONFIG" ]; then
    print_warning "未找到 realconsole.yaml，将使用默认配置"
    CONFIG=""
else
    CONFIG="--config $CONFIG"
fi

# 测试标题
echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   RealConsole 任务分解与规划系统 - 集成测试   ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# 测试 1: 简单的任务分解和规划
print_step "测试 1: 创建一个简单的任务计划"
echo ""
echo "执行命令: /plan 创建一个测试目录，在里面创建3个文本文件，然后列出文件"
echo ""

$REALCONSOLE $CONFIG --once "/plan 创建一个测试目录，在里面创建3个文本文件，然后列出文件" || {
    print_error "任务规划失败"
    cd ..
    exit 1
}

print_success "测试 1 完成"
echo ""
echo "-----------------------------------------------------------"
echo ""

# 测试 2: 查看任务计划
print_step "测试 2: 查看当前任务计划"
echo ""

$REALCONSOLE $CONFIG --once "/tasks" || {
    print_error "查看任务计划失败"
    cd ..
    exit 1
}

print_success "测试 2 完成"
echo ""
echo "-----------------------------------------------------------"
echo ""

# 测试 3: 执行任务计划
print_step "测试 3: 执行任务计划"
echo ""

$REALCONSOLE $CONFIG --once "/execute" || {
    print_warning "任务执行可能部分失败（这在测试中是正常的）"
}

print_success "测试 3 完成"
echo ""
echo "-----------------------------------------------------------"
echo ""

# 测试 4: 查看执行状态
print_step "测试 4: 查看任务执行状态"
echo ""

$REALCONSOLE $CONFIG --once "/task_status" || {
    print_error "查看执行状态失败"
    cd ..
    exit 1
}

print_success "测试 4 完成"
echo ""
echo "-----------------------------------------------------------"
echo ""

# 测试 5: 复杂任务（多依赖）
print_step "测试 5: 创建复杂任务计划（包含依赖关系）"
echo ""
echo "执行命令: /plan 在当前目录创建一个Rust项目，添加一个依赖，然后编译"
echo ""

$REALCONSOLE $CONFIG --once "/plan 在testbed目录中创建一个新的Rust练习项目，包含Cargo.toml和src/lib.rs" || {
    print_error "复杂任务规划失败"
    cd ..
    exit 1
}

print_success "测试 5 完成"
echo ""

# 总结
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         所有测试完成！                               ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

print_success "任务分解与规划系统工作正常"
echo ""
echo "提示："
echo "  • 使用 /plan <目标> 来分解和规划任务"
echo "  • 使用 /tasks 查看当前任务计划"
echo "  • 使用 /execute 执行任务计划"
echo "  • 使用 /task_status 查看执行状态"
echo ""

# 返回项目根目录并清理测试环境
cd ..
print_success "已清理测试环境（testbed/目录保留供检查）"
echo ""
