# Intent DSL 扩展计划

**日期**: 2025-10-15
**版本**: v0.5.1 计划
**当前状态**: 11 个内置意图 → 目标 20 个内置意图

---

## 🎯 设计原则

基于用例文件 `selected-cases.md` 的分析，我们选择以下标准：

1. **安全性优先** - 只实现只读操作，不修改系统状态
2. **普适性强** - 命令需跨平台兼容（macOS/Linux）
3. **高频使用** - 日常运维中常用的场景
4. **无需权限** - 不需要 root/sudo 权限

---

## 📊 现状分析

### 已实现的意图（11个）

| 意图名 | 领域 | 对应用例 |
|--------|------|----------|
| count_python_lines | FileOps | - |
| count_files | FileOps | - |
| find_large_files | FileOps | #20 查找大文件 |
| find_recent_files | FileOps | - |
| grep_pattern | DataOps | #5 搜索关键词（部分）|
| sort_lines | DataOps | - |
| count_pattern | DataOps | - |
| analyze_errors | DiagnosticOps | - |
| check_disk_usage | DiagnosticOps | #2 磁盘使用 |
| list_processes | SystemOps | #4 运行进程 |
| check_memory_usage | SystemOps | #3 内存使用 |

### 用例覆盖分析

从 30 个用例中，我们已覆盖：
- ✅ #2 磁盘使用
- ✅ #3 内存使用
- ✅ #4 运行进程
- ✅ #5 搜索文件（部分）
- ✅ #20 查找大文件

**覆盖率**: 5/30 ≈ 17%

---

## 🚀 扩展计划

### Phase 1: 高优先级（5个新意图）

#### 1. `list_directory` - 查看目录内容
**用例**: #1 查看当前目录下的文件和子目录

**关键词**: `查看`, `显示`, `列出`, `目录`, `文件`, `ls`, `list`

**正则模式**:
- `(查看|显示|列出).*(目录|文件夹|当前)`
- `(ls|list).*(dir|directory|files?)`

**参数**:
- `path` (可选): 目录路径，默认 `.`
- `show_hidden` (可选): 是否显示隐藏文件，默认 false

**命令模板**:
```bash
# 基础版本
ls -lh {{path}}

# 显示隐藏文件
ls -lah {{path}}
```

**置信度阈值**: 0.55

---

#### 2. `check_cpu_usage` - 检查CPU使用率
**用例**: #10 检查系统负载和CPU使用率

**关键词**: `检查`, `查看`, `CPU`, `使用率`, `负载`, `load`

**正则模式**:
- `(检查|查看).*(CPU|cpu|负载|load)`
- `(CPU|cpu).*(使用|占用|情况)`

**参数**: 无

**命令模板**:
```bash
# macOS
top -l 1 | head -n 10 | grep -E "(CPU|Load)"

# 或更简洁的版本
uptime && top -l 1 | grep "CPU usage"
```

**置信度阈值**: 0.6

---

#### 3. `check_network_connections` - 检查网络连接
**用例**: #7 检查网络连接状态和端口监听

**关键词**: `网络`, `连接`, `端口`, `监听`, `netstat`, `socket`

**正则模式**:
- `(检查|查看).*(网络|连接|端口)`
- `(netstat|lsof|socket).*(端口|port|listen)`

**参数**:
- `port` (可选): 指定端口号

**命令模板**:
```bash
# 基础版本 - 显示监听端口
netstat -an | grep LISTEN | head -n 20

# 指定端口
netstat -an | grep LISTEN | grep {{port}}
```

**置信度阈值**: 0.6

---

#### 4. `view_system_logs` - 查看系统日志
**用例**: #6 查看系统日志文件的最新内容

**关键词**: `日志`, `log`, `查看`, `系统`, `错误`, `警告`

**正则模式**:
- `(查看|显示).*(日志|log)`
- `(系统|system).*(日志|log|错误)`

**参数**:
- `lines` (可选): 显示行数，默认 50

**命令模板**:
```bash
# macOS - 查看系统日志最新50条
log show --predicate 'eventMessage contains "error" OR eventMessage contains "fail"' --info --last 1h | tail -n {{lines}}

# 或者更简单的版本
tail -n {{lines}} /var/log/system.log 2>/dev/null || echo "需要管理员权限查看系统日志"
```

**置信度阈值**: 0.6

---

#### 5. `check_uptime` - 查看系统运行时长
**用例**: #9 查看系统启动时间和运行时长

**关键词**: `运行时长`, `启动时间`, `uptime`, `多久`, `开机`

**正则模式**:
- `(系统|机器).*(运行|启动|开机).*(时间|多久)`
- `(uptime|运行时长|开机时间)`

**参数**: 无

**命令模板**:
```bash
uptime
```

**置信度阈值**: 0.55

---

### Phase 2: 中优先级（4个新意图）

#### 6. `test_network_connectivity` - 测试网络连通性
**用例**: #8 测试网络连通性和延迟

**关键词**: `ping`, `测试`, `网络`, `连通`, `延迟`, `connectivity`

**正则模式**:
- `(ping|测试).*(网络|连接|连通)`
- `(网络|网站).*(测试|检查|是否).*(通|连接)`

**参数**:
- `host`: 目标主机，默认 `8.8.8.8`
- `count`: ping 次数，默认 4

**命令模板**:
```bash
ping -c {{count}} {{host}}
```

**置信度阈值**: 0.6

---

#### 7. `show_system_info` - 显示系统信息
**用例**: #14 查看系统硬件信息和型号

**关键词**: `系统信息`, `硬件`, `型号`, `配置`, `system info`

**正则模式**:
- `(查看|显示).*(系统|硬件).*(信息|配置|型号)`
- `(system|hardware).*(info|information)`

**参数**: 无

**命令模板**:
```bash
# macOS
system_profiler SPHardwareDataType SPSoftwareDataType | head -n 30

# 或更简洁
uname -a && sw_vers
```

**置信度阈值**: 0.6

---

#### 8. `show_kernel_version` - 显示内核版本
**用例**: #22 查看系统内核版本和发行版信息

**关键词**: `内核`, `版本`, `发行版`, `kernel`, `version`, `uname`

**正则模式**:
- `(查看|显示).*(内核|系统).*(版本|信息)`
- `(kernel|uname|版本).*(version|信息)`

**参数**: 无

**命令模板**:
```bash
uname -a
```

**置信度阈值**: 0.55

---

#### 9. `show_env_path` - 显示环境变量PATH
**用例**: #30 查看系统环境变量和路径配置

**关键词**: `环境变量`, `PATH`, `路径`, `env`, `配置`

**正则模式**:
- `(查看|显示).*(环境变量|PATH|路径)`
- `(env|环境).*(path|变量|配置)`

**参数**: 无

**命令模板**:
```bash
# 格式化显示 PATH
echo $PATH | tr ':' '\n'
```

**置信度阈值**: 0.55

---

## 📈 实施后目标

| 指标 | 当前 | Phase 1 | Phase 2 |
|------|------|---------|---------|
| **内置意图数** | 11 | 16 | 20 |
| **用例覆盖率** | 17% | 33% | 40% |
| **领域分布** | 4 类 | 4 类 | 4 类 |

### 领域分布（Phase 2 完成后）

- **FileOps（文件操作）**: 5 个
  - count_python_lines, count_files, find_large_files, find_recent_files
  - ✨ list_directory

- **DataOps（数据处理）**: 3 个
  - grep_pattern, sort_lines, count_pattern

- **DiagnosticOps（诊断分析）**: 2 个
  - analyze_errors, check_disk_usage

- **SystemOps（系统管理）**: 10 个
  - list_processes, check_memory_usage
  - ✨ check_cpu_usage, check_network_connections, view_system_logs, check_uptime
  - ✨ test_network_connectivity, show_system_info, show_kernel_version, show_env_path

---

## 🔧 技术实现

### 文件修改

1. **src/dsl/intent/builtin.rs**
   - 添加 9 个新的 intent 方法
   - 添加 9 个新的 template 方法
   - 更新 `all_intents()` 和 `all_templates()`
   - 更新模块文档 "11个" → "20个"

2. **src/commands/core.rs**
   - 更新 `/version` 输出: "11 个内置意图" → "20 个内置意图"

3. **tests/**
   - 更新所有测试的意图数量期望值
   - 添加新意图的单元测试

---

## ✅ 实施步骤

### Phase 1（当前）
1. ✅ 分析用例，制定方案
2. ⏳ 实现 5 个高优先级意图
3. ⏳ 编写单元测试
4. ⏳ 验证跨平台兼容性

### Phase 2（后续）
5. ⏳ 实现 4 个中优先级意图
6. ⏳ 全面测试
7. ⏳ 更新文档

---

## 🎯 成功标准

- ✅ 所有新意图均为只读操作（安全）
- ✅ 命令跨平台兼容（macOS/Linux）
- ✅ 不需要 root 权限
- ✅ 所有测试通过
- ✅ 用例覆盖率提升到 40%+

---

**设计完成日期**: 2025-10-15
**预计实施时间**: Phase 1 (1-2小时), Phase 2 (1小时)
