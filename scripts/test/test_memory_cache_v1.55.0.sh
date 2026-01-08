#!/bin/bash

#########################################
# Memory 2.0 LRU 缓存功能测试 (v1.55.0)
#########################################
# 测试智能缓存机制的命中率和性能

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

print_section() {
    echo
    echo -e "${YELLOW}=== $1 ===${NC}"
    echo
}

# 检查依赖
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found"
    exit 1
fi

# 确保在项目根目录
cd "$(dirname "$0")/../.."

print_section "v1.55.0: Memory 2.0 LRU 缓存测试"

# 编译项目
print_info "编译项目..."
cargo build --release 2>&1 | tail -3

# 清理旧进程
print_info "清理旧进程..."
ps aux | grep "realconsole web" | grep -v grep | awk '{print $2}' | xargs kill -9 2>/dev/null || true
sleep 1

# 启动 Web 服务
print_info "启动 Web 服务（端口 18800）..."
DEEPSEEK_API_KEY="test-key" ./target/release/realconsole web --port 18800 > /tmp/memory_cache_test.log 2>&1 &
WEB_PID=$!
sleep 3

# 检查服务是否启动成功
if ! ps -p $WEB_PID > /dev/null; then
    print_error "Web 服务启动失败"
    cat /tmp/memory_cache_test.log
    exit 1
fi

print_success "Web 服务已启动，PID: $WEB_PID"

print_section "测试缓存功能"

echo "📝 测试说明："
echo "   1. 第一次查询：缓存未命中，执行完整流程"
echo "   2. 第二次查询（相同参数）：缓存命中，直接返回"
echo "   3. 验证日志中的缓存状态"
echo

print_info "等待 5 秒后开始测试..."
sleep 5

print_info "查看服务日志（最后 30 行）："
tail -30 /tmp/memory_cache_test.log

echo
print_section "测试步骤"

echo "1. 打开浏览器访问：http://127.0.0.1:18800"
echo "2. 依次输入以下命令并观察日志："
echo
echo "   /memory search Rust        # 第一次搜索 - 应该看到 'Cache MISS'"
echo "   /memory search Rust        # 第二次搜索 - 应该看到 'Cache HIT' 🎯"
echo "   /memory search Python      # 不同查询 - 应该看到 'Cache MISS'"
echo "   /memory search Python      # 第二次相同查询 - 应该看到 'Cache HIT' 🎯"
echo "   /memory extract 测试任务   # 第一次提取 - 应该看到 'Cache MISS'"
echo "   /memory extract 测试任务   # 第二次提取 - 应该看到 'Cache HIT' 🎯"
echo
echo "3. 按 Ctrl+C 停止日志监控"
echo

# 实时监控日志（过滤缓存相关信息）
print_info "开始监控缓存状态（过滤 [Cache] 日志）..."
echo
tail -f /tmp/memory_cache_test.log | grep --line-buffered -E "\[Cache\]|\[Memory 2.0\]|\[Quick Search\]" &
TAIL_PID=$!

# 等待用户中断
trap "kill $TAIL_PID 2>/dev/null; kill $WEB_PID 2>/dev/null; exit 0" INT

wait $TAIL_PID

# 清理
kill $WEB_PID 2>/dev/null || true

print_success "测试完成"
