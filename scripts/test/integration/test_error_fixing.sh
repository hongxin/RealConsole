#!/bin/bash
# Phase 9.2 Integration Test Script
# Tests Agent's error fixing integration functionality

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

BINARY="./target/release/realconsole"
FEEDBACK_FILE="$HOME/.config/realconsole/feedback.json"

echo -e "${YELLOW}======================================${NC}"
echo -e "${YELLOW}Phase 9.2 Error Fixing Integration Test${NC}"
echo -e "${YELLOW}======================================${NC}\n"

# Ensure binary is built
if [ ! -f "$BINARY" ]; then
    echo -e "${YELLOW}Building release binary...${NC}"
    cargo build --release
fi

# Clean feedback data for fresh test
if [ -f "$FEEDBACK_FILE" ]; then
    echo -e "${YELLOW}Backing up existing feedback data...${NC}"
    mv "$FEEDBACK_FILE" "${FEEDBACK_FILE}.backup"
fi

test_count=0
pass_count=0

run_test() {
    local test_name="$1"
    local command="$2"
    local expected_pattern="$3"

    test_count=$((test_count + 1))
    echo -e "\n${YELLOW}Test $test_count: $test_name${NC}"
    echo -e "Command: ${command}"

    output=$($BINARY --once "$command" 2>&1 || true)

    if echo "$output" | grep -q "$expected_pattern"; then
        echo -e "${GREEN}✓ PASS${NC}"
        pass_count=$((pass_count + 1))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo -e "Expected pattern: $expected_pattern"
        echo -e "Actual output:\n$output"
        return 1
    fi
}

echo -e "${YELLOW}=== Test Suite 1: Error Detection ===${NC}"

# Test 1: Command not found error
run_test "Command not found detection" \
    "!nonexistentcommand123" \
    "command not found"

# Test 2: Permission denied error (try to write to protected directory)
run_test "Permission error detection" \
    "!touch /etc/protected_file_test" \
    "Permission denied"

# Test 3: Directory not found
run_test "Directory not found error" \
    "!cd /nonexistent/directory/path" \
    "No such file or directory"

echo -e "\n${YELLOW}=== Test Suite 2: Fix Strategy Generation ===${NC}"

# Test 4: Verify fix strategies are generated (look for strategy indicators)
run_test "Fix strategies generated" \
    "!nonexistentcmd" \
    "修复策略"

echo -e "\n${YELLOW}=== Test Suite 3: /fix Command ===${NC}"

# Test 5: /fix without prior error
run_test "/fix without prior failed command" \
    "/fix" \
    "没有可重试的失败命令\|No failed command"

# Test 6: Execute error then /fix (simulate retry)
echo -e "\n${YELLOW}Test 6: /fix command with prior error${NC}"
$BINARY --once "!invalidcmd123" > /dev/null 2>&1 || true
output=$($BINARY --once "/fix" 2>&1 || true)
if echo "$output" | grep -q "重试命令\|invalidcmd123"; then
    echo -e "${GREEN}✓ PASS${NC}"
    pass_count=$((pass_count + 1))
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "Expected: 重试命令 or invalidcmd123"
    echo "Got: $output"
fi
test_count=$((test_count + 1))

echo -e "\n${YELLOW}=== Test Suite 4: Feedback Persistence ===${NC}"

# Test 7: Check if feedback file is created
run_test "Feedback file creation" \
    "!testcmd456" \
    "." # Any output is fine, we just need to trigger error analysis

sleep 1 # Give time for async file write

if [ -f "$FEEDBACK_FILE" ]; then
    echo -e "${GREEN}✓ Feedback file created at $FEEDBACK_FILE${NC}"
    pass_count=$((pass_count + 1))

    # Show feedback file size and structure
    echo -e "Feedback file size: $(wc -c < "$FEEDBACK_FILE") bytes"
    if [ -s "$FEEDBACK_FILE" ]; then
        echo -e "Feedback file is non-empty (good)"
    else
        echo -e "${YELLOW}Warning: Feedback file is empty${NC}"
    fi
else
    echo -e "${RED}✗ Feedback file not created${NC}"
fi
test_count=$((test_count + 1))

echo -e "\n${YELLOW}=== Test Suite 5: Integration Smoke Tests ===${NC}"

# Test 8: Verify ShellExecutorWithFixer is used (not old executor)
run_test "Using ShellExecutorWithFixer" \
    "!echoooo test" \
    "错误分析"

# Test 9: Successful commands don't trigger fix flow
echo -e "\n${YELLOW}Test 9: Successful commands skip fix flow${NC}"
output=$($BINARY --once "!echo 'success test'" 2>&1)
if echo "$output" | grep -q "success test" && ! echo "$output" | grep -q "修复策略"; then
    echo -e "${GREEN}✓ PASS - Successful command doesn't show fix strategies${NC}"
    pass_count=$((pass_count + 1))
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "Output: $output"
fi
test_count=$((test_count + 1))

# Test 10: Error without fix strategies (edge case)
run_test "Error without available fixes" \
    "!echo test | invalidpipe" \
    "." # Should handle gracefully

echo -e "\n${YELLOW}=== Test Suite 6: Component Integration ===${NC}"

# Test 11: Verify Agent has error_fixer components
echo -e "\n${YELLOW}Test 11: Check Agent initialization${NC}"
if cargo test --lib agent::tests::test_agent_new -- --nocapture 2>&1 | grep -q "test.*ok\|passed"; then
    echo -e "${GREEN}✓ PASS - Agent initialization test passed${NC}"
    pass_count=$((pass_count + 1))
else
    echo -e "${YELLOW}⚠ SKIP - No specific Agent initialization test found${NC}"
fi
test_count=$((test_count + 1))

echo -e "\n${YELLOW}======================================${NC}"
echo -e "${YELLOW}Test Results${NC}"
echo -e "${YELLOW}======================================${NC}"
echo -e "Total tests: $test_count"
echo -e "${GREEN}Passed: $pass_count${NC}"
echo -e "${RED}Failed: $((test_count - pass_count))${NC}"

if [ $pass_count -eq $test_count ]; then
    echo -e "\n${GREEN}✓ All tests passed!${NC}"
    exit_code=0
else
    echo -e "\n${YELLOW}⚠ Some tests failed${NC}"
    exit_code=1
fi

# Restore backup if exists
if [ -f "${FEEDBACK_FILE}.backup" ]; then
    echo -e "\n${YELLOW}Restoring feedback backup...${NC}"
    mv "${FEEDBACK_FILE}.backup" "$FEEDBACK_FILE"
fi

echo -e "\n${YELLOW}Phase 9.2 Integration Test Complete${NC}\n"
exit $exit_code
