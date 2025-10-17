# RealConsole Makefile
# 简化常见操作的快捷命令

.PHONY: help build release install uninstall test clean run wizard

# 默认目标：显示帮助
help:
	@echo "RealConsole 快捷命令"
	@echo ""
	@echo "构建命令:"
	@echo "  make build          - 编译 debug 版本"
	@echo "  make release        - 编译 release 版本"
	@echo ""
	@echo "安装命令:"
	@echo "  make install        - 安装到用户目录 (~/.local/bin)"
	@echo "  make uninstall      - 卸载 RealConsole"
	@echo ""
	@echo "测试命令:"
	@echo "  make test           - 运行所有测试"
	@echo "  make test-intent    - 运行 Intent DSL 测试"
	@echo "  make coverage       - 生成测试覆盖率报告"
	@echo ""
	@echo "运行命令:"
	@echo "  make run            - 运行 debug 版本"
	@echo "  make wizard         - 运行配置向导"
	@echo ""
	@echo "清理命令:"
	@echo "  make clean          - 清理编译产物"
	@echo "  make clean-all      - 清理所有生成文件"
	@echo ""
	@echo "代码质量:"
	@echo "  make fmt            - 格式化代码"
	@echo "  make lint           - 运行 clippy 检查"
	@echo "  make check          - 检查代码（不编译）"
	@echo ""

# 编译 debug 版本
build:
	@echo "编译 debug 版本..."
	cargo build

# 编译 release 版本
release:
	@echo "编译 release 版本..."
	cargo build --release

# 安装到用户目录
install: release
	@echo "安装 RealConsole..."
	./install.sh

# 卸载
uninstall:
	@echo "卸载 RealConsole..."
	./uninstall.sh

# 运行所有测试
test:
	@echo "运行测试..."
	cargo test

# 运行 Intent DSL 测试
test-intent:
	@echo "运行 Intent DSL 测试..."
	cargo test --test test_intent_

# 生成测试覆盖率报告
coverage:
	@echo "生成测试覆盖率报告..."
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		cargo llvm-cov --html; \
		echo "覆盖率报告已生成: target/llvm-cov/html/index.html"; \
	else \
		echo "未安装 cargo-llvm-cov，请运行:"; \
		echo "  cargo install cargo-llvm-cov"; \
	fi

# 运行 debug 版本
run: build
	@echo "运行 RealConsole (debug)..."
	./target/debug/realconsole

# 运行配置向导
wizard: release
	@echo "运行配置向导..."
	./target/release/realconsole wizard

# 清理编译产物
clean:
	@echo "清理编译产物..."
	cargo clean

# 清理所有生成文件
clean-all: clean
	@echo "清理所有生成文件..."
	@rm -rf coverage/
	@rm -rf target/
	@echo "完成"

# 格式化代码
fmt:
	@echo "格式化代码..."
	cargo fmt

# 运行 clippy 检查
lint:
	@echo "运行 clippy 检查..."
	cargo clippy -- -D warnings

# 检查代码（不编译）
check:
	@echo "检查代码..."
	cargo check

# 构建并运行 release 版本（快速测试）
quick: release
	@echo "运行 RealConsole (release)..."
	./target/release/realconsole

# 完整的 CI 流程
ci: fmt lint test
	@echo "✓ CI 检查通过"

# 准备发布
prepare-release: clean ci release
	@echo "✓ 发布准备完成"
	@echo ""
	@echo "可执行文件位置: target/release/realconsole"
	@echo "运行安装命令: make install"
