# Phase 6.1 实施计划 - 安全功能增强

**日期**: 2025-10-15
**目标**: 增加 8 个完全安全的意图，提升场景覆盖率到 54%
**时间**: 1-2 天
**状态**: 准备实施

---

## 概述

基于 [SECURITY_ANALYSIS.md](./SECURITY_ANALYSIS.md) 的评估，Phase 6.1 将仅实现无风险或低风险的功能：

- ✅ **文件查询类**: find_files_by_name, count_file_stats, compare_files
- ✅ **网络测试类**: ping_host
- ✅ **系统信息类**: view_env_var, check_service_status
- ✅ **文件管理类**: create_directory, create_symlink

---

## 实施任务清单

### 批次 1: 完全安全（只读操作）- 优先级 1

#### 1. find_files_by_name - 按名称查找文件

**用例**: #11 查找文件按名称或类型

**关键词**:
```rust
vec![
    "查找".to_string(),
    "搜索".to_string(),
    "寻找".to_string(),
    "文件".to_string(),
    "名字".to_string(),
    "名称".to_string(),
    "find".to_string(),
]
```

**正则模式**:
```rust
vec![
    r"(?i)(查找|搜索|寻找).*(文件|file).*名(字|称)".to_string(),
    r"(?i)(find|locate).*(file|name)".to_string(),
    r"(?i)名(字|称).*为.*的.*文件".to_string(),
]
```

**实体**:
- `name`: String (文件名模式)
- `path`: Path (搜索路径，默认 ".")

**模板命令**:
```bash
find {path} -name '{name}' -type f
```

**示例查询**:
- "查找名为 config.yaml 的文件"
- "搜索当前目录下的 test.txt"
- "find file named main.rs"

---

#### 2. count_file_stats - 统计文件行数/字数

**用例**: #42 统计文件行数、字数

**关键词**:
```rust
vec![
    "统计".to_string(),
    "计算".to_string(),
    "行数".to_string(),
    "字数".to_string(),
    "单词".to_string(),
    "wc".to_string(),
]
```

**正则模式**:
```rust
vec![
    r"(?i)统计.*(行数|字数|单词)".to_string(),
    r"(?i)(行数|字数|单词).*统计".to_string(),
    r"(?i)wc.*file".to_string(),
]
```

**实体**:
- `file`: Path (文件路径)

**模板命令**:
```bash
wc {file}
```

**示例查询**:
- "统计 README.md 的行数"
- "计算 main.rs 的字数"
- "wc src/main.rs"

---

#### 3. compare_files - 比较文件差异

**用例**: #41 比较两个文件的差异

**关键词**:
```rust
vec![
    "比较".to_string(),
    "对比".to_string(),
    "差异".to_string(),
    "不同".to_string(),
    "diff".to_string(),
]
```

**正则模式**:
```rust
vec![
    r"(?i)比较.*(文件|file)".to_string(),
    r"(?i)(差异|不同|区别)".to_string(),
    r"(?i)diff.*file".to_string(),
]
```

**实体**:
- `file1`: Path (第一个文件)
- `file2`: Path (第二个文件)

**模板命令**:
```bash
diff -u {file1} {file2}
```

**示例查询**:
- "比较 old.txt 和 new.txt"
- "对比 config.yaml 与 config.yaml.bak 的差异"
- "diff main.rs backup/main.rs"

---

#### 4. ping_host - 测试网络连通性

**用例**: #20 测试网络连通性

**关键词**:
```rust
vec![
    "ping".to_string(),
    "测试".to_string(),
    "检测".to_string(),
    "网络".to_string(),
    "连通".to_string(),
    "可达".to_string(),
]
```

**正则模式**:
```rust
vec![
    r"(?i)ping.*host".to_string(),
    r"(?i)测试.*(网络|连通)".to_string(),
    r"(?i)(检测|测试).*(可达|连接)".to_string(),
]
```

**实体**:
- `host`: String (主机名或IP)
- `count`: Number (默认 4)

**模板命令**:
```bash
ping -c {count} {host}
```

**示例查询**:
- "ping google.com"
- "测试 baidu.com 的网络连通性"
- "检测 192.168.1.1 是否可达"

---

### 批次 2: 低危操作（需要基本验证）- 优先级 2

#### 5. view_env_var - 查看环境变量

**用例**: #39 查看环境变量

**关键词**:
```rust
vec![
    "查看".to_string(),
    "显示".to_string(),
    "环境变量".to_string(),
    "变量".to_string(),
    "env".to_string(),
    "echo".to_string(),
]
```

**正则模式**:
```rust
vec![
    r"(?i)(查看|显示).*(环境变量|变量)".to_string(),
    r"(?i)env.*var".to_string(),
    r"(?i)echo.*\$".to_string(),
]
```

**实体**:
- `var`: String (变量名，可选，默认显示全部)

**模板命令**:
```bash
# 查看全部
env

# 查看特定变量
echo ${var}
```

**示例查询**:
- "查看所有环境变量"
- "显示 PATH 变量"
- "echo $HOME"

---

#### 6. check_service_status - 查看服务状态

**用例**: #22 查看服务状态

**关键词**:
```rust
vec![
    "查看".to_string(),
    "检查".to_string(),
    "服务".to_string(),
    "状态".to_string(),
    "运行".to_string(),
    "service".to_string(),
]
```

**正则模式**:
```rust
vec![
    r"(?i)(查看|检查).*(服务|service)".to_string(),
    r"(?i)服务.*状态".to_string(),
    r"(?i)(systemctl|launchctl).*status".to_string(),
]
```

**实体**:
- `service`: String (服务名称)

**模板命令**:
```bash
# macOS
launchctl list | grep {service}

# Linux
systemctl status {service}
```

**跨平台实现**:
需要根据平台选择命令

**示例查询**:
- "查看 nginx 服务状态"
- "检查 mysql 是否运行"
- "systemctl status docker"

---

#### 7. create_directory - 创建目录

**用例**: #4 创建新的目录

**关键词**:
```rust
vec![
    "创建".to_string(),
    "新建".to_string(),
    "建立".to_string(),
    "目录".to_string(),
    "文件夹".to_string(),
    "mkdir".to_string(),
]
```

**正则模式**:
```rust
vec![
    r"(?i)(创建|新建|建立).*(目录|文件夹|folder)".to_string(),
    r"(?i)mkdir.*dir".to_string(),
]
```

**实体**:
- `path`: Path (目录路径)

**模板命令**:
```bash
mkdir -p {path}
```

**安全措施**:
- 路径验证（禁止系统目录）
- 检查磁盘空间
- 父目录必须存在或使用 `-p`

**示例查询**:
- "创建一个名为 test 的目录"
- "新建文件夹 backup/2024"
- "mkdir build/debug"

---

#### 8. create_symlink - 创建符号链接

**用例**: #40 创建符号链接

**关键词**:
```rust
vec![
    "创建".to_string(),
    "建立".to_string(),
    "符号链接".to_string(),
    "软链接".to_string(),
    "链接".to_string(),
    "ln".to_string(),
]
```

**正则模式**:
```rust
vec![
    r"(?i)(创建|建立).*(符号链接|软链接|symlink)".to_string(),
    r"(?i)ln.*-s".to_string(),
]
```

**实体**:
- `source`: Path (源文件)
- `target`: Path (链接名称)

**模板命令**:
```bash
ln -s {source} {target}
```

**安全措施**:
- 验证源文件存在
- 路径验证
- 目标不存在检查

**示例查询**:
- "创建 /usr/local/bin/python3 到 python 的符号链接"
- "建立软链接 config.yaml 指向 config.prod.yaml"
- "ln -s src/main.rs main_link.rs"

---

## 代码变更清单

### 1. src/dsl/intent/builtin.rs

#### 变更 1: 更新文档注释
```rust
//! 本模块提供精选的 24 个高频意图和模板，涵盖日常使用的 80% 场景。
//!                    ^^                                        ^^^
//!                   (从16更新到24)                       (从80%更新到90%)
```

#### 变更 2: 更新 all_intents() 方法
在第 94 行后添加新的意图调用：
```rust
pub fn all_intents(&self) -> Vec<Intent> {
    vec![
        // ===== 文件操作类 (FileOps) =====
        // ... 现有5个 ...
        self.find_files_by_name(),    // 新增
        // ===== 数据处理类 (DataOps) =====
        // ... 现有3个 ...
        self.count_file_stats(),      // 新增
        self.compare_files(),         // 新增
        // ===== 诊断分析类 (DiagnosticOps) =====
        // ... 现有3个 ...
        // ===== 系统管理类 (SystemOps) =====
        // ... 现有5个 ...
        self.ping_host(),             // 新增
        self.view_env_var(),          // 新增
        self.check_service_status(),  // 新增 (已存在？检查)
        // ===== 文件管理类 (新类别？) =====
        self.create_directory(),      // 新增
        self.create_symlink(),        // 新增
    ]
}
```

#### 变更 3: 更新 all_templates() 方法
类似地添加对应的模板调用

#### 变更 4: 添加意图和模板定义
在第 685 行后（`check_uptime` 模板之后）添加8个新的意图定义和模板函数

### 2. 测试文件更新

#### src/dsl/intent/builtin.rs 测试部分
- 更新测试中的意图数量断言：`16` → `24`
- 添加新意图的单元测试

---

## 测试计划

### 单元测试

```bash
cargo test --lib dsl::intent::builtin
```

**预期结果**:
- ✅ `test_builtin_creation`: 24 个意图和模板
- ✅ 所有新增意图的匹配测试通过
- ✅ 所有新增模板的生成测试通过

### 集成测试

```bash
# 测试 find_files_by_name
echo "查找名为 Cargo.toml 的文件" | ./target/release/realconsole

# 测试 count_file_stats
echo "统计 README.md 的行数" | ./target/release/realconsole

# 测试 compare_files
echo "比较 .env.example 和 .env" | ./target/release/realconsole

# 测试 ping_host
echo "ping google.com" | ./target/release/realconsole

# 测试 view_env_var
echo "查看 PATH 环境变量" | ./target/release/realconsole

# 测试 check_service_status (macOS)
echo "查看 sshd 服务状态" | ./target/release/realconsole

# 测试 create_directory
echo "创建目录 /tmp/test_realconsole" | ./target/release/realconsole

# 测试 create_symlink
echo "创建符号链接 /tmp/test_link 指向 /tmp/test_realconsole" | ./target/release/realconsole
```

---

## 风险和缓解

### 风险 1: 意图匹配冲突
**描述**: 新意图的关键词可能与现有意图冲突
**概率**: 中
**影响**: 中
**缓解**:
- 使用更高的置信度阈值（0.60+）
- 更精确的正则模式
- 充分测试边界情况

### 风险 2: 跨平台兼容性
**描述**: `check_service_status` 在 macOS 和 Linux 上命令不同
**概率**: 高
**影响**: 高
**缓解**:
- 使用 `cfg!(target_os = "macos")` 条件编译
- 模板中包含两套命令
- 文档明确说明平台差异

### 风险 3: 安全验证不足
**描述**: `create_directory` 和 `create_symlink` 可能被滥用
**概率**: 低
**影响**: 中
**缓解**:
- 路径严格验证
- 黑名单保护系统目录
- 清晰的错误提示

---

## 成功指标

### 量化指标
- ✅ Intent 数量：16 → 24 (+8, +50%)
- ✅ 场景覆盖率：36% → 54% (+18%, +9个场景)
- ✅ 测试通过率：100%
- ✅ 编译无警告

### 质量指标
- ✅ 所有新意图有单元测试
- ✅ 集成测试覆盖典型查询
- ✅ 文档完整（描述、示例、安全说明）
- ✅ 代码风格一致

---

## 实施顺序

### Day 1 上午: 批次 1 (完全安全)
1. 添加 `find_files_by_name`
2. 添加 `count_file_stats`
3. 添加 `compare_files`
4. 添加 `ping_host`
5. 编译测试

### Day 1 下午: 批次 2 (低危操作)
6. 添加 `view_env_var`
7. 添加 `check_service_status` (跨平台)
8. 编译测试

### Day 2 上午: 批次 3 (需要验证)
9. 添加 `create_directory` (含安全检查)
10. 添加 `create_symlink` (含验证)
11. 编译测试

### Day 2 下午: 测试和文档
12. 运行所有单元测试
13. 运行集成测试
14. 更新文档
15. 创建 Phase 6.1 完成报告

---

## 后续步骤

### Phase 6.1 完成后
1. 发布更新
2. 收集用户反馈
3. 修复发现的问题

### Phase 6.2 准备
1. 评估 Phase 6.1 效果
2. 设计中危操作的安全机制
3. 规划 Phase 6.2 实施

---

## 决策记录

### 决策 1: 不实现删除操作
**原因**: 风险太高，可能导致数据丢失
**状态**: ✅ 确认，暂缓到 Phase 6.3+

### 决策 2: 限制 ping 次数
**原因**: 防止长时间占用
**实施**: 默认 4 次，最多 10 次
**状态**: ✅ 确认

### 决策 3: create_directory 使用 `-p`
**原因**: 自动创建父目录，用户体验更好
**风险**: 可能意外创建多级目录
**缓解**: 在提示中说明
**状态**: ✅ 确认

---

## 参考文档

- [50_CASES_COVERAGE_ANALYSIS.md](./50_CASES_COVERAGE_ANALYSIS.md) - 场景覆盖分析
- [SECURITY_ANALYSIS.md](./SECURITY_ANALYSIS.md) - 安全性评估
- [INTENT_DSL_GUIDE.md](../guides/INTENT_DSL_GUIDE.md) - Intent DSL 指南

---

**文档版本**: v1.0
**创建日期**: 2025-10-15
**准备开始**: 等待确认
