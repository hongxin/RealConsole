//! 系统监控命令
//!
//! 提供系统资源监控和进程管理

use crate::command::Command;
use crate::system_monitor::SystemMonitor;
use colored::Colorize;

/// 注册系统监控相关命令
pub fn register_system_commands(registry: &mut crate::command::CommandRegistry) {
    // 系统状态命令
    registry.register(Command::from_fn(
        "sys",
        "显示系统资源状态（CPU、内存、磁盘）",
        handle_sys_status,
    ));

    // CPU 信息
    registry.register(Command::from_fn("cpu", "显示 CPU 使用情况", handle_cpu));

    // 内存信息
    registry.register(Command::from_fn(
        "memory-info",
        "显示系统内存使用情况",
        handle_mem,
    ));

    registry.register(Command::from_fn(
        "sysm",
        "显示内存使用（memory-info 的别名）",
        handle_mem,
    ));

    // 磁盘信息
    registry.register(Command::from_fn(
        "disk",
        "显示磁盘空间使用情况",
        handle_disk,
    ));

    // 进程列表
    registry.register(Command::from_fn(
        "top",
        "显示占用资源最多的进程",
        handle_top,
    ));
}

/// 处理 /sys 命令 - 显示系统整体状态
fn handle_sys_status(_arg: &str) -> String {
    let mut output = vec![];

    output.push(format!("\n{}", "系统资源监控".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

    // CPU 信息
    match SystemMonitor::get_cpu_info() {
        Ok(cpu) => {
            output.push(String::new());
            output.push(format!("{}", "CPU".green().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            output.push(format!(
                "  {}: {} 核",
                "核心数".dimmed(),
                cpu.cores.to_string().cyan()
            ));

            let total_usage = cpu.user_usage + cpu.system_usage;
            let usage_color = if total_usage > 80.0 {
                total_usage.to_string().red()
            } else if total_usage > 50.0 {
                total_usage.to_string().yellow()
            } else {
                total_usage.to_string().green()
            };

            output.push(format!(
                "  {}: {}% (用户: {:.1}%, 系统: {:.1}%)",
                "使用率".dimmed(),
                usage_color,
                cpu.user_usage,
                cpu.system_usage
            ));

            output.push(format!("  {}: {:.1}%", "空闲".dimmed(), cpu.idle));

            // CPU 使用率条形图
            let bar_width = 50;
            let filled = ((total_usage / 100.0) * bar_width as f64) as usize;
            let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);
            output.push(format!("  {}", bar.dimmed()));
        }
        Err(e) => {
            output.push(format!("{} {}", "CPU 信息获取失败:".red(), e));
        }
    }

    // 内存信息
    match SystemMonitor::get_memory_info() {
        Ok(mem) => {
            output.push(String::new());
            output.push(format!("{}", "内存".green().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            output.push(format!(
                "  {}: {} MB",
                "总容量".dimmed(),
                mem.total_mb.to_string().cyan()
            ));

            output.push(format!(
                "  {}: {} MB ({:.1}%)",
                "已使用".dimmed(),
                mem.used_mb,
                mem.usage_percent
            ));

            output.push(format!(
                "  {}: {} MB",
                "可用".dimmed(),
                mem.available_mb.to_string().green()
            ));

            // 内存使用率条形图
            let bar_width = 50;
            let filled = ((mem.usage_percent / 100.0) * bar_width as f64) as usize;
            let bar = if mem.usage_percent > 80.0 {
                "█".repeat(filled).red().to_string() + &"░".repeat(bar_width - filled)
            } else if mem.usage_percent > 60.0 {
                "█".repeat(filled).yellow().to_string() + &"░".repeat(bar_width - filled)
            } else {
                "█".repeat(filled).green().to_string() + &"░".repeat(bar_width - filled)
            };
            output.push(format!("  {}", bar));
        }
        Err(e) => {
            output.push(format!("{} {}", "内存信息获取失败:".red(), e));
        }
    }

    // 磁盘信息
    match SystemMonitor::get_disk_info() {
        Ok(disks) => {
            output.push(String::new());
            output.push(format!("{}", "磁盘".green().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            for disk in disks.iter().take(3) {
                // 只显示前3个
                output.push(format!("\n  {}", disk.mount_point.cyan().bold()));

                let usage_str = if disk.usage_percent > 90.0 {
                    format!("{:.1}%", disk.usage_percent).red()
                } else if disk.usage_percent > 70.0 {
                    format!("{:.1}%", disk.usage_percent).yellow()
                } else {
                    format!("{:.1}%", disk.usage_percent).green()
                };

                output.push(format!(
                    "    {}: {:.1} GB / {:.1} GB ({})",
                    "使用".dimmed(),
                    disk.used_gb,
                    disk.total_gb,
                    usage_str
                ));

                output.push(format!(
                    "    {}: {:.1} GB",
                    "可用".dimmed(),
                    disk.available_gb
                ));

                // 磁盘使用率条形图
                let bar_width = 40;
                let filled = ((disk.usage_percent / 100.0) * bar_width as f64) as usize;
                let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);
                output.push(format!("    {}", bar.dimmed()));
            }

            if disks.len() > 3 {
                output.push(format!(
                    "\n  {} 还有 {} 个磁盘",
                    "...".dimmed(),
                    disks.len() - 3
                ));
            }
        }
        Err(e) => {
            output.push(format!("{} {}", "磁盘信息获取失败:".red(), e));
        }
    }

    // 快捷命令提示
    output.push(String::new());
    output.push(format!("{}", "快捷命令".cyan().bold()));
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());
    output.push(format!("  {} 查看 CPU 详情", "/cpu".cyan()));
    output.push(format!("  {} 查看内存详情", "/sysm".cyan()));
    output.push(format!("  {} 查看磁盘详情", "/disk".cyan()));
    output.push(format!("  {} 查看进程列表", "/top".cyan()));

    output.push(String::new());

    output.join("\n")
}

/// 处理 /cpu 命令
fn handle_cpu(_arg: &str) -> String {
    match SystemMonitor::get_cpu_info() {
        Ok(cpu) => {
            let mut output = vec![];

            output.push(format!("\n{}", "CPU 信息".cyan().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            output.push(format!(
                "\n  {}: {} 核",
                "CPU 核心数".dimmed(),
                cpu.cores.to_string().cyan().bold()
            ));

            let total_usage = cpu.user_usage + cpu.system_usage;

            output.push(String::new());
            output.push(format!("{}", "使用率详情".green().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            output.push(format!("  {}: {:.2}%", "用户进程".dimmed(), cpu.user_usage));

            output.push(format!(
                "  {}: {:.2}%",
                "系统进程".dimmed(),
                cpu.system_usage
            ));

            output.push(format!("  {}: {:.2}%", "总使用率".dimmed(), total_usage));

            output.push(format!("  {}: {:.2}%", "空闲".dimmed(), cpu.idle));

            // 可视化
            output.push(String::new());
            output.push(format!("{}", "使用率分布".green().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            let bar_width = 50;
            let user_filled = ((cpu.user_usage / 100.0) * bar_width as f64) as usize;
            let sys_filled = ((cpu.system_usage / 100.0) * bar_width as f64) as usize;
            let idle_filled = ((cpu.idle / 100.0) * bar_width as f64) as usize;

            output.push(format!(
                "  {} {}",
                "用户:".dimmed(),
                "█".repeat(user_filled).cyan()
            ));

            output.push(format!(
                "  {} {}",
                "系统:".dimmed(),
                "█".repeat(sys_filled).yellow()
            ));

            output.push(format!(
                "  {} {}",
                "空闲:".dimmed(),
                "█".repeat(idle_filled).green()
            ));

            // 健康度评估
            output.push(String::new());
            output.push(format!("{}", "健康度评估".green().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            let status = if total_usage > 90.0 {
                ("严重负载", "建议检查高CPU进程", "●".red())
            } else if total_usage > 70.0 {
                ("高负载", "需要关注", "●".yellow())
            } else if total_usage > 50.0 {
                ("中等负载", "正常", "●".yellow())
            } else {
                ("低负载", "系统空闲", "●".green())
            };

            output.push(format!(
                "\n  {}: {} {} {}",
                "状态".dimmed(),
                status.2,
                status.0.bold(),
                format!("({})", status.1).dimmed()
            ));

            output.push(format!(
                "\n  {} {}\n",
                "[TIP]".cyan(),
                "使用 /top 查看占用 CPU 最多的进程".dimmed()
            ));

            output.join("\n")
        }
        Err(e) => format!("{} {}", "✗ 获取 CPU 信息失败:".red(), e),
    }
}

/// 处理 /mem 命令
fn handle_mem(_arg: &str) -> String {
    match SystemMonitor::get_memory_info() {
        Ok(mem) => {
            let mut output = vec![];

            output.push(format!("\n{}", "内存信息".cyan().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            output.push(format!(
                "\n  {}: {} MB ({} GB)",
                "总内存".dimmed(),
                mem.total_mb.to_string().cyan().bold(),
                (mem.total_mb as f64 / 1024.0).round()
            ));

            output.push(String::new());
            output.push(format!("{}", "使用情况".green().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            let used_color = if mem.usage_percent > 80.0 {
                mem.used_mb.to_string().red()
            } else if mem.usage_percent > 60.0 {
                mem.used_mb.to_string().yellow()
            } else {
                mem.used_mb.to_string().green()
            };

            output.push(format!(
                "  {}: {} MB ({:.1}%)",
                "已使用".dimmed(),
                used_color,
                mem.usage_percent
            ));

            output.push(format!(
                "  {}: {} MB ({:.1}%)",
                "可用".dimmed(),
                mem.available_mb.to_string().green(),
                100.0 - mem.usage_percent
            ));

            // 可视化
            output.push(String::new());
            output.push(format!("{}", "内存分布".green().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            let bar_width = 50;
            let used_filled = ((mem.usage_percent / 100.0) * bar_width as f64) as usize;

            let bar = if mem.usage_percent > 80.0 {
                "█".repeat(used_filled).red().to_string()
                    + &"░".repeat(bar_width - used_filled).dimmed().to_string()
            } else if mem.usage_percent > 60.0 {
                "█".repeat(used_filled).yellow().to_string()
                    + &"░".repeat(bar_width - used_filled).dimmed().to_string()
            } else {
                "█".repeat(used_filled).green().to_string()
                    + &"░".repeat(bar_width - used_filled).dimmed().to_string()
            };

            output.push(format!("  {}", bar));
            output.push(format!(
                "  {} {} {}",
                "已使用".dimmed(),
                "█".repeat(10),
                "可用".dimmed()
            ));

            // 健康度
            output.push(String::new());
            output.push(format!("{}", "健康度评估".green().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            let status = if mem.usage_percent > 90.0 {
                ("内存不足", "建议释放内存或关闭程序", "●".red())
            } else if mem.usage_percent > 70.0 {
                ("内存紧张", "需要关注", "●".yellow())
            } else {
                ("内存充足", "运行正常", "●".green())
            };

            output.push(format!(
                "\n  {}: {} {} {}",
                "状态".dimmed(),
                status.2,
                status.0.bold(),
                format!("({})", status.1).dimmed()
            ));

            output.push(format!(
                "\n  {} {}\n",
                "[TIP]".cyan(),
                "使用 /top 查看占用内存最多的进程".dimmed()
            ));

            output.join("\n")
        }
        Err(e) => format!("{} {}", "✗ 获取内存信息失败:".red(), e),
    }
}

/// 处理 /disk 命令
fn handle_disk(_arg: &str) -> String {
    match SystemMonitor::get_disk_info() {
        Ok(disks) => {
            let mut output = vec![];

            output.push(format!("\n{}", "磁盘空间".cyan().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            for (i, disk) in disks.iter().enumerate() {
                output.push(String::new());
                output.push(format!(
                    "{} {}",
                    format!("#{}", i + 1).dimmed(),
                    disk.mount_point.cyan().bold()
                ));

                output.push(format!(
                    "  {}: {}",
                    "文件系统".dimmed(),
                    disk.filesystem.dimmed()
                ));

                let usage_str = if disk.usage_percent > 90.0 {
                    format!("{:.1}%", disk.usage_percent).red()
                } else if disk.usage_percent > 70.0 {
                    format!("{:.1}%", disk.usage_percent).yellow()
                } else {
                    format!("{:.1}%", disk.usage_percent).green()
                };

                output.push(format!(
                    "  {}: {:.2} GB / {:.2} GB ({})",
                    "使用".dimmed(),
                    disk.used_gb,
                    disk.total_gb,
                    usage_str
                ));

                output.push(format!(
                    "  {}: {:.2} GB",
                    "可用".dimmed(),
                    disk.available_gb
                ));

                // 条形图
                let bar_width = 40;
                let filled = ((disk.usage_percent / 100.0) * bar_width as f64) as usize;
                let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);
                output.push(format!("  {}", bar.dimmed()));

                // 警告
                if disk.usage_percent > 90.0 {
                    output.push(format!("  {} 磁盘空间严重不足！", "⚠".red()));
                } else if disk.usage_percent > 80.0 {
                    output.push(format!("  {} 磁盘空间较少", "⚠".yellow()));
                }
            }

            output.push(String::new());

            output.join("\n")
        }
        Err(e) => format!("{} {}", "✗ 获取磁盘信息失败:".red(), e),
    }
}

/// 处理 /top 命令
fn handle_top(arg: &str) -> String {
    let limit = if arg.is_empty() {
        10
    } else {
        arg.trim().parse::<usize>().unwrap_or(10)
    };

    match SystemMonitor::get_top_processes(limit) {
        Ok(processes) => {
            let mut output = vec![];

            output.push(format!("\n{}", format!("Top {} 进程", limit).cyan().bold()));
            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed().to_string());

            output.push(format!(
                "\n  {:<8} {:<6} {:<6} {}",
                "PID".dimmed(),
                "CPU%".dimmed(),
                "MEM%".dimmed(),
                "进程名".dimmed()
            ));

            output.push(
                "  ─────────────────────────────────────────"
                    .dimmed()
                    .to_string(),
            );

            for proc in processes {
                let cpu_str = if proc.cpu_percent > 50.0 {
                    format!("{:>5.1}%", proc.cpu_percent).red()
                } else if proc.cpu_percent > 20.0 {
                    format!("{:>5.1}%", proc.cpu_percent).yellow()
                } else {
                    format!("{:>5.1}%", proc.cpu_percent).green()
                };

                let mem_str = if proc.mem_percent > 10.0 {
                    format!("{:>5.1}%", proc.mem_percent).red()
                } else if proc.mem_percent > 5.0 {
                    format!("{:>5.1}%", proc.mem_percent).yellow()
                } else {
                    format!("{:>5.1}%", proc.mem_percent).normal()
                };

                output.push(format!(
                    "  {:<8} {} {} {}",
                    proc.pid, cpu_str, mem_str, proc.name
                ));
            }

            output.push(format!(
                "\n  {} {}\n",
                "[TIP]".cyan(),
                "使用 /top [N] 查看前 N 个进程".dimmed()
            ));

            output.join("\n")
        }
        Err(e) => format!("{} {}", "✗ 获取进程列表失败:".red(), e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_sys_status() {
        let result = handle_sys_status("");
        assert!(result.contains("系统资源监控"));
    }

    #[test]
    fn test_handle_cpu() {
        let result = handle_cpu("");
        assert!(result.contains("CPU") || result.contains("失败"));
    }

    #[test]
    fn test_handle_mem() {
        let result = handle_mem("");
        assert!(result.contains("内存") || result.contains("失败"));
    }

    #[test]
    fn test_handle_disk() {
        let result = handle_disk("");
        assert!(result.contains("磁盘") || result.contains("失败"));
    }

    #[test]
    fn test_handle_top() {
        let result = handle_top("5");
        assert!(result.contains("进程") || result.contains("失败"));
    }

    #[test]
    fn test_handle_top_default() {
        let result = handle_top("");
        assert!(result.contains("Top 10") || result.contains("失败"));
    }
}
