//! 系统监控工具
//!
//! 提供系统资源监控能力：
//! - CPU 使用率
//! - 内存使用情况
//! - 磁盘空间
//! - 进程列表

use serde::{Deserialize, Serialize};
use std::process::Command;

/// CPU 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU 核心数
    pub cores: usize,
    /// 系统 CPU 使用率 (%)
    pub system_usage: f64,
    /// 用户 CPU 使用率 (%)
    pub user_usage: f64,
    /// 空闲率 (%)
    pub idle: f64,
}

/// 内存信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// 总内存 (MB)
    pub total_mb: u64,
    /// 已使用内存 (MB)
    pub used_mb: u64,
    /// 可用内存 (MB)
    pub available_mb: u64,
    /// 使用率 (%)
    pub usage_percent: f64,
}

/// 磁盘信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// 挂载点
    pub mount_point: String,
    /// 文件系统
    pub filesystem: String,
    /// 总容量 (GB)
    pub total_gb: f64,
    /// 已使用 (GB)
    pub used_gb: f64,
    /// 可用空间 (GB)
    pub available_gb: f64,
    /// 使用率 (%)
    pub usage_percent: f64,
}

/// 进程信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// 进程 ID
    pub pid: u32,
    /// 进程名称
    pub name: String,
    /// CPU 使用率 (%)
    pub cpu_percent: f64,
    /// 内存使用率 (%)
    pub mem_percent: f64,
}

/// 系统监控器
pub struct SystemMonitor;

impl SystemMonitor {
    /// 获取 CPU 信息
    pub fn get_cpu_info() -> Result<CpuInfo, String> {
        #[cfg(target_os = "macos")]
        {
            Self::get_cpu_info_macos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::get_cpu_info_linux()
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Err("不支持的操作系统".to_string())
        }
    }

    /// macOS CPU 信息
    #[cfg(target_os = "macos")]
    fn get_cpu_info_macos() -> Result<CpuInfo, String> {
        // 获取 CPU 核心数
        let cores_output = Command::new("sysctl")
            .args(["-n", "hw.ncpu"])
            .output()
            .map_err(|e| format!("执行 sysctl 失败: {}", e))?;

        let cores = String::from_utf8_lossy(&cores_output.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(1);

        // 使用 top 命令获取 CPU 使用率
        let output = Command::new("top")
            .args(["-l", "1", "-n", "0"])
            .output()
            .map_err(|e| format!("执行 top 失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        // 解析 CPU 使用率
        // 格式: CPU usage: 3.57% user, 10.71% sys, 85.71% idle
        let mut user_usage = 0.0;
        let mut system_usage = 0.0;
        let mut idle = 0.0;

        for line in output_str.lines() {
            if line.contains("CPU usage:") {
                // 提取百分比
                let parts: Vec<&str> = line.split(',').collect();
                for part in parts {
                    if part.contains("user") {
                        user_usage = Self::extract_percentage(part);
                    } else if part.contains("sys") {
                        system_usage = Self::extract_percentage(part);
                    } else if part.contains("idle") {
                        idle = Self::extract_percentage(part);
                    }
                }
                break;
            }
        }

        Ok(CpuInfo {
            cores,
            system_usage,
            user_usage,
            idle,
        })
    }

    /// Linux CPU 信息
    #[cfg(target_os = "linux")]
    fn get_cpu_info_linux() -> Result<CpuInfo, String> {
        // 获取 CPU 核心数
        let cores_output = Command::new("nproc")
            .output()
            .map_err(|e| format!("执行 nproc 失败: {}", e))?;

        let cores = String::from_utf8_lossy(&cores_output.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(1);

        // 使用 top 命令获取 CPU 使用率
        let output = Command::new("top")
            .args(["-bn1"])
            .output()
            .map_err(|e| format!("执行 top 失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        let mut user_usage = 0.0;
        let mut system_usage = 0.0;
        let mut idle = 0.0;

        for line in output_str.lines() {
            if line.contains("%Cpu") || line.contains("Cpu(s)") {
                // 格式: %Cpu(s):  3.0 us,  1.0 sy,  0.0 ni, 96.0 id
                let parts: Vec<&str> = line.split(',').collect();
                for part in parts {
                    if part.contains("us") {
                        user_usage = Self::extract_percentage(part);
                    } else if part.contains("sy") {
                        system_usage = Self::extract_percentage(part);
                    } else if part.contains("id") {
                        idle = Self::extract_percentage(part);
                    }
                }
                break;
            }
        }

        Ok(CpuInfo {
            cores,
            system_usage,
            user_usage,
            idle,
        })
    }

    /// 获取内存信息
    pub fn get_memory_info() -> Result<MemoryInfo, String> {
        #[cfg(target_os = "macos")]
        {
            Self::get_memory_info_macos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::get_memory_info_linux()
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Err("不支持的操作系统".to_string())
        }
    }

    /// macOS 内存信息
    #[cfg(target_os = "macos")]
    fn get_memory_info_macos() -> Result<MemoryInfo, String> {
        let output = Command::new("vm_stat")
            .output()
            .map_err(|e| format!("执行 vm_stat 失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        let page_size = 4096; // macOS page size is typically 4KB
        let mut free_pages = 0u64;
        let mut active_pages = 0u64;
        let mut inactive_pages = 0u64;
        let mut wired_pages = 0u64;

        for line in output_str.lines() {
            if line.contains("Pages free:") {
                free_pages = Self::extract_number(line);
            } else if line.contains("Pages active:") {
                active_pages = Self::extract_number(line);
            } else if line.contains("Pages inactive:") {
                inactive_pages = Self::extract_number(line);
            } else if line.contains("Pages wired down:") {
                wired_pages = Self::extract_number(line);
            }
        }

        let total_mb = (free_pages + active_pages + inactive_pages + wired_pages) * page_size / 1024 / 1024;
        let used_mb = (active_pages + wired_pages) * page_size / 1024 / 1024;
        let available_mb = (free_pages + inactive_pages) * page_size / 1024 / 1024;
        let usage_percent = if total_mb > 0 {
            (used_mb as f64 / total_mb as f64) * 100.0
        } else {
            0.0
        };

        Ok(MemoryInfo {
            total_mb,
            used_mb,
            available_mb,
            usage_percent,
        })
    }

    /// Linux 内存信息
    #[cfg(target_os = "linux")]
    fn get_memory_info_linux() -> Result<MemoryInfo, String> {
        let output = Command::new("free")
            .args(["-m"]) // 以 MB 为单位
            .output()
            .map_err(|e| format!("执行 free 失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        // 解析 free 输出
        // 格式:
        //              total        used        free      shared  buff/cache   available
        // Mem:         16384        8192        4096         512        4096        7680

        let mut total_mb = 0u64;
        let mut used_mb = 0u64;
        let mut available_mb = 0u64;

        for line in output_str.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 7 {
                    total_mb = parts[1].parse().unwrap_or(0);
                    used_mb = parts[2].parse().unwrap_or(0);
                    available_mb = parts[6].parse().unwrap_or(0);
                }
                break;
            }
        }

        let usage_percent = if total_mb > 0 {
            (used_mb as f64 / total_mb as f64) * 100.0
        } else {
            0.0
        };

        Ok(MemoryInfo {
            total_mb,
            used_mb,
            available_mb,
            usage_percent,
        })
    }

    /// 获取磁盘信息
    pub fn get_disk_info() -> Result<Vec<DiskInfo>, String> {
        let output = Command::new("df")
            .args(["-h"])
            .output()
            .map_err(|e| format!("执行 df 失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        let mut disks = Vec::new();

        for line in output_str.lines().skip(1) {
            // 跳过标题行
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let filesystem = parts[0].to_string();
                let total = Self::parse_size(parts[1]);
                let used = Self::parse_size(parts[2]);
                let available = Self::parse_size(parts[3]);
                let usage_percent = Self::extract_percentage(parts[4]);
                let mount_point = parts[5].to_string();

                // 过滤掉特殊文件系统
                if !filesystem.starts_with("devfs")
                    && !filesystem.starts_with("map")
                    && !mount_point.starts_with("/dev")
                {
                    disks.push(DiskInfo {
                        mount_point,
                        filesystem,
                        total_gb: total,
                        used_gb: used,
                        available_gb: available,
                        usage_percent,
                    });
                }
            }
        }

        Ok(disks)
    }

    /// 获取进程列表（按 CPU 使用率排序）
    pub fn get_top_processes(limit: usize) -> Result<Vec<ProcessInfo>, String> {
        #[cfg(target_os = "macos")]
        {
            Self::get_top_processes_macos(limit)
        }

        #[cfg(target_os = "linux")]
        {
            Self::get_top_processes_linux(limit)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Err("不支持的操作系统".to_string())
        }
    }

    /// macOS 进程列表
    #[cfg(target_os = "macos")]
    fn get_top_processes_macos(limit: usize) -> Result<Vec<ProcessInfo>, String> {
        let output = Command::new("ps")
            .args(["aux"])
            .output()
            .map_err(|e| format!("执行 ps 失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();

        for line in output_str.lines().skip(1) {
            // 跳过标题行
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                if let Ok(pid) = parts[1].parse::<u32>() {
                    let cpu_percent = parts[2].parse::<f64>().unwrap_or(0.0);
                    let mem_percent = parts[3].parse::<f64>().unwrap_or(0.0);
                    let name = parts[10].to_string();

                    processes.push(ProcessInfo {
                        pid,
                        name,
                        cpu_percent,
                        mem_percent,
                    });
                }
            }
        }

        // 按 CPU 使用率排序
        processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());

        Ok(processes.into_iter().take(limit).collect())
    }

    /// Linux 进程列表
    #[cfg(target_os = "linux")]
    fn get_top_processes_linux(limit: usize) -> Result<Vec<ProcessInfo>, String> {
        let output = Command::new("ps")
            .args(["aux", "--sort=-%cpu"])
            .output()
            .map_err(|e| format!("执行 ps 失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();

        for line in output_str.lines().skip(1).take(limit) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                if let Ok(pid) = parts[1].parse::<u32>() {
                    let cpu_percent = parts[2].parse::<f64>().unwrap_or(0.0);
                    let mem_percent = parts[3].parse::<f64>().unwrap_or(0.0);
                    let name = parts[10].to_string();

                    processes.push(ProcessInfo {
                        pid,
                        name,
                        cpu_percent,
                        mem_percent,
                    });
                }
            }
        }

        Ok(processes)
    }

    /// 提取百分比数字
    fn extract_percentage(s: &str) -> f64 {
        s.chars()
            .filter(|c| c.is_numeric() || *c == '.')
            .collect::<String>()
            .parse::<f64>()
            .unwrap_or(0.0)
    }

    /// 提取数字
    fn extract_number(s: &str) -> u64 {
        s.chars()
            .filter(|c| c.is_numeric())
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(0)
    }

    /// 解析大小（如 "10G", "500M"）转换为 GB
    fn parse_size(s: &str) -> f64 {
        let num_str: String = s.chars().filter(|c| c.is_numeric() || *c == '.').collect();
        let num = num_str.parse::<f64>().unwrap_or(0.0);

        if s.contains('G') || s.contains('g') {
            num
        } else if s.contains('M') || s.contains('m') {
            num / 1024.0
        } else if s.contains('K') || s.contains('k') {
            num / 1024.0 / 1024.0
        } else if s.contains('T') || s.contains('t') {
            num * 1024.0
        } else {
            num
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_percentage() {
        assert_eq!(SystemMonitor::extract_percentage("3.57% user"), 3.57);
        assert_eq!(SystemMonitor::extract_percentage("10.71%"), 10.71);
        assert_eq!(SystemMonitor::extract_percentage("85%"), 85.0);
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(SystemMonitor::extract_number("Pages free: 123456."), 123456);
        assert_eq!(SystemMonitor::extract_number("12345"), 12345);
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(SystemMonitor::parse_size("10G"), 10.0);
        assert_eq!(SystemMonitor::parse_size("500M"), 500.0 / 1024.0);
        assert_eq!(SystemMonitor::parse_size("1024K"), 1.0 / 1024.0);
        assert_eq!(SystemMonitor::parse_size("1T"), 1024.0);
    }

    #[test]
    fn test_get_cpu_info() {
        // 这个测试在实际系统上运行
        let result = SystemMonitor::get_cpu_info();
        if result.is_ok() {
            let cpu_info = result.unwrap();
            assert!(cpu_info.cores > 0);
            assert!(cpu_info.idle >= 0.0 && cpu_info.idle <= 100.0);
        }
    }

    #[test]
    fn test_get_memory_info() {
        let result = SystemMonitor::get_memory_info();
        if result.is_ok() {
            let mem_info = result.unwrap();
            assert!(mem_info.total_mb > 0);
            assert!(mem_info.usage_percent >= 0.0 && mem_info.usage_percent <= 100.0);
        }
    }

    #[test]
    fn test_get_disk_info() {
        let result = SystemMonitor::get_disk_info();
        if result.is_ok() {
            let disks = result.unwrap();
            assert!(!disks.is_empty());
            for disk in disks {
                assert!(disk.total_gb > 0.0);
            }
        }
    }

    #[test]
    fn test_get_top_processes() {
        let result = SystemMonitor::get_top_processes(5);
        if result.is_ok() {
            let processes = result.unwrap();
            assert!(processes.len() <= 5);
        }
    }
}
