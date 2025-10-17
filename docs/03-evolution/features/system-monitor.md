# 系统监控工具

## 概述

系统监控工具是 RealConsole Phase 6 的核心功能之一，为程序员和运维工程师提供实时的系统资源监控能力。通过简洁的命令即可查看 CPU、内存、磁盘和进程状态，快速诊断系统性能问题。

**实现日期**: 2025-10-16
**版本**: v0.6.0 (Phase 6)
**支持平台**: macOS, Linux

## 核心功能

### 1. 系统整体状态 (`/sys`)

一览系统所有关键资源：

```bash
> /sys

系统资源监控
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

CPU
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  核心数: 16 核
  使用率: 10.6% (用户: 3.3%, 系统: 7.4%)
  空闲: 89.4%
  █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░

内存
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  总容量: 30996 MB
  已使用: 14179 MB (45.7%)
  可用: 16816 MB
  ██████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░

磁盘
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  /
    使用: 11.0 GB / 1843.2 GB (9.0%)
    可用: 117.0 GB
    ███░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░

快捷命令
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  /cpu 查看 CPU 详情
  /sysm 查看内存详情
  /disk 查看磁盘详情
  /top 查看进程列表
```

**特性**：
- ✅ 一屏显示所有关键指标
- ✅ 可视化条形图
- ✅ 颜色编码（绿/黄/红）
- ✅ 快捷命令导航

### 2. CPU 详细信息 (`/cpu`)

深度分析 CPU 使用情况：

```bash
> /cpu

CPU 信息
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  CPU 核心数: 16 核

使用率详情
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  用户进程: 11.50%
  系统进程: 7.26%
  总使用率: 18.76%
  空闲: 81.23%

使用率分布
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  用户: █████
  系统: ███
  空闲: ████████████████████████████████████████

健康度评估
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  状态: ● 低负载 (系统空闲)
```

**分析指标**：
- 📊 CPU 核心数
- 📈 用户/系统进程分别使用率
- 💚 健康度评估（4级）
  - 低负载 (< 50%)
  - 中等负载 (50-70%)
  - 高负载 (70-90%)
  - 严重负载 (> 90%)

### 3. 内存使用情况 (`/sysm`, `/memory-info`)

实时监控内存状态：

```bash
> /sysm

内存信息
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  总内存: 30989 MB (30 GB)

使用情况
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  已使用: 14260 MB (46.0%)
  可用: 16728 MB (54.0%)

内存分布
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ███████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░
  已使用 ██████████ 可用

健康度评估
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  状态: ● 内存充足 (运行正常)
```

**监控维度**：
- 💾 总容量 (MB/GB)
- 📊 已使用/可用 (绝对值+百分比)
- 🎨 可视化内存分布
- 🏥 健康度评估
  - 内存充足 (< 70%)
  - 内存紧张 (70-90%)
  - 内存不足 (> 90%)

### 4. 磁盘空间监控 (`/disk`)

查看所有磁盘分区的空间使用：

```bash
> /disk

磁盘空间
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#1 /
  文件系统: /dev/disk3s1s1
  使用: 447.84 GB / 1843.18 GB (24.3%)
  可用: 1395.34 GB
  ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░

#2 /System/Volumes/Data
  文件系统: /dev/disk3s5
  使用: 442.12 GB / 1843.18 GB (24.0%)
  可用: 1401.06 GB
  ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░

  ⚠ 磁盘空间较少  // 当使用率 > 80% 时显示
```

**监控范围**：
- 📂 所有挂载点
- 🗃️ 文件系统类型
- 📏 使用/总容量 (GB)
- 🎯 使用率百分比
- ⚠️  自动告警（80%+）

### 5. 进程监控 (`/top`)

查看占用资源最多的进程：

```bash
> /top 5

Top 5 进程
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  PID      CPU%   MEM%   进程名
  ─────────────────────────────────────────
  71286     43.1%   0.7% claude
  617       17.4%   1.4% WindowServer
  609        5.2%   0.1% kernel_task
  5529       5.2%   0.2% Terminal
  48900      4.1%   0.6% Google Chrome

  💡 使用 /top [N] 查看前 N 个进程
```

**功能特点**：
- 🔢 PID 显示
- 📊 CPU 使用率排序
- 💾 内存使用率
- 🎨 颜色编码高资源进程
- ⚙️  可自定义显示数量

## 技术实现

### 架构设计

```
src/system_monitor.rs              // 核心监控引擎
├── CpuInfo                        // CPU 信息
│   ├── cores                     // 核心数
│   ├── user_usage                // 用户使用率
│   ├── system_usage              // 系统使用率
│   └── idle                      // 空闲率
├── MemoryInfo                     // 内存信息
│   ├── total_mb                  // 总容量
│   ├── used_mb                   // 已使用
│   ├── available_mb              // 可用
│   └── usage_percent             // 使用率
├── DiskInfo                       // 磁盘信息
│   ├── mount_point               // 挂载点
│   ├── filesystem                // 文件系统
│   ├── total_gb / used_gb        // 容量
│   └── usage_percent             // 使用率
├── ProcessInfo                    // 进程信息
│   ├── pid                       // 进程 ID
│   ├── name                      // 进程名
│   ├── cpu_percent               // CPU 使用率
│   └── mem_percent               // 内存使用率
└── SystemMonitor                  // 监控器
    ├── get_cpu_info()            // 获取 CPU
    ├── get_memory_info()         // 获取内存
    ├── get_disk_info()           // 获取磁盘
    └── get_top_processes()       // 获取进程

src/commands/system_cmd.rs         // 命令处理层
├── handle_sys_status()            // /sys 命令
├── handle_cpu()                   // /cpu 命令
├── handle_mem()                   // /sysm 命令
├── handle_disk()                  // /disk 命令
└── handle_top()                   // /top 命令
```

### 跨平台实现

#### macOS 实现

```rust
// CPU - 使用 top 命令
fn get_cpu_info_macos() -> Result<CpuInfo, String> {
    // 核心数: sysctl -n hw.ncpu
    // 使用率: top -l 1 -n 0
    // 解析: CPU usage: 3.57% user, 10.71% sys, 85.71% idle
}

// 内存 - 使用 vm_stat
fn get_memory_info_macos() -> Result<MemoryInfo, String> {
    // vm_stat 输出页面统计
    // 计算: (active + wired) / total * 100
}
```

#### Linux 实现

```rust
// CPU - 使用 top 命令
fn get_cpu_info_linux() -> Result<CpuInfo, String> {
    // 核心数: nproc
    // 使用率: top -bn1
    // 解析: %Cpu(s):  3.0 us,  1.0 sy,  96.0 id
}

// 内存 - 使用 free 命令
fn get_memory_info_linux() -> Result<MemoryInfo, String> {
    // free -m 输出 MB 单位
    // 格式: Mem: total used available
}
```

#### 通用实现

```rust
// 磁盘 - df 命令（跨平台）
pub fn get_disk_info() -> Result<Vec<DiskInfo>, String> {
    // df -h 人类可读格式
    // 过滤特殊文件系统（devfs, map等）
}

// 进程 - ps 命令（跨平台）
pub fn get_top_processes(limit: usize) -> Result<Vec<ProcessInfo>, String> {
    // ps aux 列出所有进程
    // macOS: 按CPU排序
    // Linux: ps aux --sort=-%cpu
}
```

### 核心算法

#### 1. 百分比提取

```rust
fn extract_percentage(s: &str) -> f64 {
    // "3.57% user" → 3.57
    // "10%" → 10.0
    s.chars()
        .filter(|c| c.is_numeric() || *c == '.')
        .collect::<String>()
        .parse::<f64>()
        .unwrap_or(0.0)
}
```

#### 2. 大小解析

```rust
fn parse_size(s: &str) -> f64 {
    let num = extract_number(s);

    // 转换为 GB
    if s.contains('T') { num * 1024.0 }
    else if s.contains('G') { num }
    else if s.contains('M') { num / 1024.0 }
    else if s.contains('K') { num / 1024.0 / 1024.0 }
    else { num }
}
```

#### 3. 健康度评估

```rust
let health_status = match (error_rate, usage_percent) {
    (usage, _) if usage > 90.0 => ("严重", "需立即处理", "●".red()),
    (usage, _) if usage > 70.0 => ("警告", "需要关注", "●".yellow()),
    (usage, _) if usage > 50.0 => ("中等", "正常", "●".yellow()),
    _ => ("良好", "系统空闲", "●".green()),
};
```

## 使用场景

### 场景 1: 性能问题排查

```bash
# 系统变慢，快速诊断
> /sys

# 发现 CPU 使用率高
> /cpu
  总使用率: 95.5%
  状态: ● 严重负载

# 查看占用 CPU 的进程
> /top 10

# 找到罪魁祸首: process_heavy (PID 12345)
> !kill 12345
```

### 场景 2: 内存泄漏检测

```bash
# 监控内存使用
> /sysm
  已使用: 28000 MB (90.3%)
  状态: ● 内存不足

# 查看内存占用进程
> /top 10

# 发现异常进程并处理
```

### 场景 3: 磁盘空间告警

```bash
> /disk

#1 /
  使用: 900 GB / 1000 GB (90.0%)
  ⚠ 磁盘空间严重不足！

# 查找大文件
> !du -sh /* | sort -hr | head -10

# 清理空间
```

### 场景 4: 日常健康检查

```bash
# 每日例行检查
> /sys

# 所有指标正常
CPU: ● 低负载 (系统空闲)
内存: ● 内存充足 (运行正常)
磁盘: 使用率 < 70%

# 继续工作
```

## 命令参考

| 命令 | 别名 | 功能 | 参数 |
|-----|------|------|------|
| `/sys` | - | 系统整体状态 | 无 |
| `/cpu` | - | CPU 详细信息 | 无 |
| `/memory-info` | `/sysm` | 内存使用情况 | 无 |
| `/disk` | - | 磁盘空间 | 无 |
| `/top` | - | 进程列表 | `[N]` (默认10) |

### 注意事项

⚠️  **命令名称**：
- 内存监控使用 `/sysm` 或 `/memory-info`
- `/mem` 是记忆管理命令的别名（避免冲突）

## 测试覆盖

### 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_percentage() {
        assert_eq!(extract_percentage("3.57%"), 3.57);
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("10G"), 10.0);
        assert_eq!(parse_size("500M"), 500.0 / 1024.0);
    }

    #[test]
    fn test_get_cpu_info() {
        let cpu = SystemMonitor::get_cpu_info().unwrap();
        assert!(cpu.cores > 0);
    }
}
```

**测试结果**: ✅ 13/13 tests passed

- ✅ `test_extract_percentage` - 百分比提取
- ✅ `test_extract_number` - 数字提取
- ✅ `test_parse_size` - 大小解析
- ✅ `test_get_cpu_info` - CPU 信息获取
- ✅ `test_get_memory_info` - 内存信息获取
- ✅ `test_get_disk_info` - 磁盘信息获取
- ✅ `test_get_top_processes` - 进程列表获取
- ✅ 命令处理层测试 (6个)

## 性能优化

- ⚡ **命令缓存**: 系统命令输出可缓存 1-2 秒
- ⚡ **按需加载**: 只在需要时获取详细信息
- ⚡ **轻量级**: 使用系统命令，无第三方依赖

## 未来规划

### Phase 6.5: 增强功能

- 🚀 **历史趋势**: 记录资源使用历史
- 🚀 **图表可视化**: ASCII 图表显示趋势
- 🚀 **自定义告警**: 设置资源阈值告警
- 🚀 **进程管理**: 直接 kill/nice 进程

### Phase 7: 高级监控

- 🔮 **实时监控**: watch 模式持续刷新
- 🔮 **网络监控**: 网络流量统计
- 🔮 **IO 监控**: 磁盘 I/O 性能
- 🔮 **GPU 监控**: GPU 使用率（NVIDIA/AMD）

### Phase 8: 分布式

- 📊 **远程监控**: 监控远程服务器
- 📊 **集群视图**: 多节点资源聚合
- 📊 **导出数据**: Prometheus 格式导出

## 平台兼容性

| 功能 | macOS | Linux | Windows |
|-----|-------|-------|---------|
| CPU 监控 | ✅ | ✅ | ❌ |
| 内存监控 | ✅ | ✅ | ❌ |
| 磁盘监控 | ✅ | ✅ | ❌ |
| 进程监控 | ✅ | ✅ | ❌ |

*Windows 支持计划在 Phase 7 实现*

## 错误处理

- ✅ 命令不存在 → 友好错误提示
- ✅ 权限不足 → 显示权限要求
- ✅ 平台不支持 → 提示支持的平台
- ✅ 解析失败 → 使用默认值/跳过

## 总结

系统监控工具通过简洁的命令提供强大的系统洞察：

- ✅ **实时性**: 即时获取系统状态
- ✅ **可视化**: 直观的条形图和颜色编码
- ✅ **智能化**: 自动健康度评估
- ✅ **轻量级**: 无第三方依赖，纯命令行
- ✅ **跨平台**: macOS 和 Linux 支持

这使 RealConsole 成为程序员和运维工程师日常系统监控的得力助手。

---

**相关文档**:
- [Log Analyzer](./LOG_ANALYZER.md)
- [Git Smart Assistant](./GIT_SMART_ASSISTANT.md)
- [Phase 6 Roadmap](../planning/PROJECT_REVIEW_AND_ROADMAP.md)
