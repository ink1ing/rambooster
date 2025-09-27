use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};
use crate::{read_mem_stats};
use crate::processes::{get_all_processes, sort_and_take_processes};
use crate::release::{boost, BoostResult, get_candidate_processes};
use crate::log_entry::{write_log_event, LogEvent, list_log_files, get_logs_size};
use chrono::Utc;
use std::fs;

#[derive(Debug, Clone)]
pub enum DataLevel {
    Minimal,    // 最少信息
    Standard,   // 标准信息
    Detailed,   // 详细信息
    Verbose,    // 冗长信息
}

#[derive(Debug, Clone)]
pub enum VisualizationLevel {
    Minimal,    // 最简可视化 - 仅基本信息
    Standard,   // 标准可视化 - 进度条和基本动画
    Enhanced,   // 增强可视化 - 详细进度和彩色输出
    Rich,       // 丰富可视化 - 全面的视觉效果和动画
}

#[derive(Debug, Clone)]
pub enum BoostLevel {
    Low,        // 低等级清理
    Mid,        // 中等级清理
    High,       // 高等级清理
    Killer,     // 杀手模式 - 最激进的清理
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Txt,
    Markdown,
}

pub struct InteractiveSession {
    pub data_level: DataLevel,
    pub boost_level: BoostLevel,
    pub visualization_level: VisualizationLevel,
    pub last_boost_result: Option<BoostResult>,
    pub session_history: Vec<String>,
}

impl Default for InteractiveSession {
    fn default() -> Self {
        Self {
            data_level: DataLevel::Standard,
            boost_level: BoostLevel::Killer,
            visualization_level: VisualizationLevel::Standard,
            last_boost_result: None,
            session_history: Vec::new(),
        }
    }
}

impl InteractiveSession {
    pub fn new() -> Self {
        Self::default()
    }

    fn show_progress_bar(&self, step: usize, total: usize, message: &str) {
        match self.visualization_level {
            VisualizationLevel::Minimal => {
                println!("{} {}/{}", message, step, total);
            },
            VisualizationLevel::Standard | VisualizationLevel::Enhanced | VisualizationLevel::Rich => {
                let progress = step as f32 / total as f32;
                let bar_width = 30;
                let filled = (bar_width as f32 * progress) as usize;

                let bar_char = match self.visualization_level {
                    VisualizationLevel::Standard => "█",
                    VisualizationLevel::Enhanced => "▓",
                    VisualizationLevel::Rich => "🟩",
                    _ => "█",
                };

                let empty_char = match self.visualization_level {
                    VisualizationLevel::Rich => "🟨",
                    _ => "░",
                };

                let bar = format!("{}{}",
                    bar_char.repeat(filled),
                    empty_char.repeat(bar_width - filled)
                );

                print!("\r{} [{}] {:.1}% ({}/{})", message, bar, progress * 100.0, step, total);
                io::stdout().flush().unwrap();
            }
        }
    }

    fn show_spinner(&self, message: &str, duration_ms: u64) {
        if matches!(self.visualization_level, VisualizationLevel::Minimal) {
            println!("{}", message);
            return;
        }

        let frames = match self.visualization_level {
            VisualizationLevel::Standard => vec!["│", "/", "─", "\\"],
            VisualizationLevel::Enhanced => vec!["◰", "◱", "◲", "◳"],
            VisualizationLevel::Rich => vec!["🌎", "🌍", "🌏", "🌎"],
            _ => vec!["│", "/", "─", "\\"],
        };

        let start = Instant::now();
        let mut frame_idx = 0;

        while start.elapsed().as_millis() < duration_ms as u128 {
            print!("\r{} {} {}", frames[frame_idx % frames.len()], message, frames[frame_idx % frames.len()]);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(100));
            frame_idx += 1;
        }

        print!("\r{} \u{2713}\n", message);
        io::stdout().flush().unwrap();
    }

    fn show_memory_animation(&self, before_mb: u32, _after_mb: u32, freed_mb: i32) {
        if matches!(self.visualization_level, VisualizationLevel::Minimal | VisualizationLevel::Standard) {
            return;
        }

        println!("\n✨ 内存清理动画:");

        for i in 0..=10 {
            let progress = i as f32 / 10.0;
            let current_mb = before_mb as f32 + (freed_mb as f32 * progress);

            let bar_char = if matches!(self.visualization_level, VisualizationLevel::Rich) {
                "🟦"  // 蓝色方块
            } else {
                "█"
            };

            let memory_bar = {
                let total_blocks = 20;
                let used_blocks = ((current_mb / before_mb as f32) * total_blocks as f32) as usize;
                let used_blocks = used_blocks.min(total_blocks);
                format!("{}{}",
                    bar_char.repeat(used_blocks),
                    "░".repeat(total_blocks - used_blocks)
                )
            };

            print!("\r💾 内存: [{}] {:.0}MB ", memory_bar, current_mb);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(150));
        }

        println!();
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.print_welcome();

        loop {
            self.print_prompt();

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            self.session_history.push(input.to_string());

            match self.handle_command(input) {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                },
                Err(e) => {
                    eprintln!("❌ 错误: {}", e);
                }
            }
        }

        println!("👋 再见！感谢使用 RAM Booster");
        Ok(())
    }

    fn print_welcome(&self) {
        // ASCII 艺术字横幅 - 螃蟹红色 (Rust 橙红色 #CE422B)
        println!("\x1b[38;5;196m██████╗  █████╗ ███╗   ███╗\x1b[0m");
        println!("\x1b[38;5;196m██╔══██╗██╔══██╗████╗ ████║\x1b[0m");
        println!("\x1b[38;5;196m██████╔╝███████║██╔████╔██║\x1b[0m");
        println!("\x1b[38;5;196m██╔══██╗██╔══██║██║╚██╔╝██║\x1b[0m");
        println!("\x1b[38;5;196m██║  ██║██║  ██║██║ ╚═╝ ██║\x1b[0m");
        println!("\x1b[38;5;196m╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝\x1b[0m");
        println!("\x1b[38;5;208m██████╗  ██████╗  ██████╗ ███████╗████████╗███████╗██████╗ \x1b[0m");
        println!("\x1b[38;5;208m██╔══██╗██╔═══██╗██╔═══██╗██╔════╝╚══██╔══╝██╔════╝██╔══██╗\x1b[0m");
        println!("\x1b[38;5;208m██████╔╝██║   ██║██║   ██║███████╗   ██║   █████╗  ██████╔╝\x1b[0m");
        println!("\x1b[38;5;208m██╔══██╗██║   ██║██║   ██║╚════██║   ██║   ██╔══╝  ██╔══██╗\x1b[0m");
        println!("\x1b[38;5;208m██████╔╝╚██████╔╝╚██████╔╝███████║   ██║   ███████╗██║  ██║\x1b[0m");
        println!("\x1b[38;5;208m╚═════╝  ╚═════╝  ╚═════╝ ╚══════╝   ╚═╝   ╚══════╝╚═╝  ╚═╝\x1b[0m");
        println!();
        println!("\x1b[38;5;214m                  🦀 RUST POWERED 🦀\x1b[0m");
        println!("\x1b[38;5;220m                   Performance++\x1b[0m");
        println!("\x1b[38;5;226m                     Memory Safe\x1b[0m");
        println!("\x1b[38;5;220m                     Zero-Cost++\x1b[0m");
        println!("\x1b[38;5;214m                   github@ink1ing\x1b[0m");
        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("💡 输入 'b' 开始清理内存");
        println!("📊 输入 'status' 查看当前状态");
        println!("⚙️  输入 '/help' 查看所有命令");
        println!("🚪 输入 'exit' 或 'quit' 退出");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    fn print_prompt(&self) {
        let level_indicator = match self.boost_level {
            BoostLevel::Low => "💚",
            BoostLevel::Mid => "💙",
            BoostLevel::High => "💜",
            BoostLevel::Killer => "💀",
        };

        let data_indicator = match self.data_level {
            DataLevel::Minimal => "📊",
            DataLevel::Standard => "📈",
            DataLevel::Detailed => "📋",
            DataLevel::Verbose => "📜",
        };

        print!("{} {} rb> ", level_indicator, data_indicator);
        io::stdout().flush().unwrap();
    }

    fn handle_command(&mut self, input: &str) -> Result<bool, Box<dyn std::error::Error>> {
        match input {
            // 退出命令
            "exit" | "quit" | "q" => return Ok(false),

            // 核心功能命令
            "boost" | "b" => {
                self.handle_boost()?;
            },
            "status" => {
                self.handle_status()?;
            },
            "clear" => {
                // 清屏
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush().unwrap();
            },

            // 斜杠命令
            cmd if cmd.starts_with('/') => {
                self.handle_slash_command(cmd)?;
            },

            // 帮助
            "help" | "?" => {
                self.print_help();
            },

            _ => {
                println!("❓ 未知命令: '{}'", input);
                println!("💡 输入 'help' 查看可用命令");
            }
        }

        Ok(true)
    }

    pub fn handle_boost(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.boost_level {
            BoostLevel::Killer => {
                println!("💀 启动杀手模式清理...");
                self.killer_boost()
            },
            _ => {
                match self.visualization_level {
                    VisualizationLevel::Minimal => println!("🔄 正在进行内存清理..."),
                    _ => self.show_spinner("🔄 正在进行内存清理", 1500),
                }
                self.standard_boost()
            }
        }
    }

    fn standard_boost(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let before_stats = read_mem_stats()?;
        let _start_time = Instant::now();

        match boost() {
            Ok(result) => {
                self.last_boost_result = Some(result.clone());

                // 记录日志
                let log_event = LogEvent {
                    ts: Utc::now().to_rfc3339(),
                    action: format!("interactive_boost_{:?}", self.boost_level).to_lowercase(),
                    before: Some(before_stats),
                    after: Some(result.after.clone()),
                    delta_mb: result.delta_mb,
                    pressure: result.after.pressure.clone(),
                    details: serde_json::json!({
                        "boost_level": format!("{:?}", self.boost_level),
                        "data_level": format!("{:?}", self.data_level),
                        "duration_seconds": result.duration.as_secs_f64(),
                        "interactive_session": true
                    }),
                };

                if let Err(e) = write_log_event(&log_event) {
                    eprintln!("⚠️  日志写入失败: {}", e);
                }

                // 根据数据级别显示结果
                self.print_boost_result(&result);

            },
            Err(e) => {
                println!("❌ 内存清理失败: {:?}", e);
                if let crate::release::BoostError::Purge(crate::release::PurgeError::CommandNotFound) = e {
                    println!("💡 请安装 Xcode Command Line Tools: xcode-select --install");
                }
            }
        }

        Ok(())
    }

    fn killer_boost(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⚠️  杀手模式：将执行多轮清理和进程终止");

        let before_stats = read_mem_stats()?;
        let mut total_freed = 0i64;
        let _start_time = Instant::now();

        // 第一轮：标准清理
        println!("🔄 第1轮: 标准内存清理");
        match boost() {
            Ok(result) => {
                total_freed += result.delta_mb;
                println!("✅ 第1轮释放: {} MB", result.delta_mb);
            },
            Err(e) => println!("❌ 第1轮失败: {:?}", e),
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        // 第二轮：进程清理
        println!("🔄 第2轮: 高内存进程清理");
        let processes = get_all_processes();
        let candidates = get_candidate_processes(&processes, 200, &std::collections::HashSet::new(), &std::collections::HashSet::new());

        let mut killed_count = 0;
        for process in candidates.iter().take(3) { // 最多终止3个高内存进程
            println!("💀 终止进程: {} (PID: {}, 内存: {} MB)", process.name, process.pid, process.rss_mb);
            if crate::release::terminate(process.pid, true) { // 使用强制终止
                killed_count += 1;
                println!("  ✅ 进程 {} 已终止", process.name);
                std::thread::sleep(std::time::Duration::from_millis(500));
            } else {
                println!("  ❌ 进程 {} 终止失败", process.name);
            }
        }
        println!("✅ 第2轮终止: {} 个进程", killed_count);

        std::thread::sleep(std::time::Duration::from_secs(2));

        // 第三轮：再次清理
        println!("🔄 第3轮: 深度内存清理");
        match boost() {
            Ok(result) => {
                total_freed += result.delta_mb;
                println!("✅ 第3轮释放: {} MB", result.delta_mb);
            },
            Err(e) => println!("❌ 第3轮失败: {:?}", e),
        }

        let end_stats = read_mem_stats()?;
        let duration = _start_time.elapsed();

        // 创建综合结果
        let final_result = BoostResult {
            before: before_stats.clone(),
            after: end_stats.clone(),
            delta_mb: total_freed,
            duration,
        };

        self.last_boost_result = Some(final_result.clone());

        // 记录日志
        let log_event = LogEvent {
            ts: Utc::now().to_rfc3339(),
            action: "interactive_boost_killer".to_string(),
            before: Some(before_stats.clone()),
            after: Some(final_result.after.clone()),
            delta_mb: total_freed,
            pressure: final_result.after.pressure.clone(),
            details: serde_json::json!({
                "boost_level": "Killer",
                "data_level": format!("{:?}", self.data_level),
                "duration_seconds": duration.as_secs_f64(),
                "interactive_session": true,
                "processes_killed": killed_count,
                "rounds": 3
            }),
        };

        if let Err(e) = write_log_event(&log_event) {
            eprintln!("⚠️  日志写入失败: {}", e);
        }

        // 显示最终结果
        println!();
        if !matches!(self.visualization_level, VisualizationLevel::Minimal) {
            self.show_memory_animation(
                before_stats.free_mb.try_into().unwrap_or(0),
                end_stats.free_mb.try_into().unwrap_or(0),
                total_freed.try_into().unwrap_or(0)
            );
        }
        println!("━━━ 💀 杀手模式完成 💀 ━━━");
        println!("⏱️  总耗时: {:.2}s", duration.as_secs_f64());
        println!("💀 终止进程: {} 个", killed_count);
        println!("🆓 总共释放: {} MB", total_freed);
        self.print_boost_result(&final_result);

        Ok(())
    }

    fn handle_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stats = read_mem_stats()?;
        let processes = get_all_processes();

        match self.data_level {
            DataLevel::Minimal => {
                println!("💾 内存: {}MB 可用 / {}MB 总计", stats.free_mb, stats.total_mb);
                println!("📊 压力: {:?}", stats.pressure);
            },
            DataLevel::Standard => {
                println!("━━━ 内存状态 ━━━");
                println!("💾 总内存: {} MB", stats.total_mb);
                println!("🆓 可用: {} MB", stats.free_mb);
                println!("🔥 活跃: {} MB", stats.active_mb);
                println!("💤 非活跃: {} MB", stats.inactive_mb);
                println!("📊 压力级别: {:?}", stats.pressure);

                let top_processes = sort_and_take_processes(processes, 5);
                println!("\n🔝 Top 5 进程:");
                for (i, p) in top_processes.iter().enumerate() {
                    println!("  {}. {} - {} MB", i + 1, p.name, p.rss_mb);
                }
            },
            DataLevel::Detailed => {
                println!("━━━ 详细内存状态 ━━━");
                println!("💾 总内存: {} MB", stats.total_mb);
                println!("🆓 可用: {} MB ({:.1}%)", stats.free_mb, (stats.free_mb as f64 / stats.total_mb as f64) * 100.0);
                println!("🔥 活跃: {} MB", stats.active_mb);
                println!("💤 非活跃: {} MB", stats.inactive_mb);
                println!("🔒 有线: {} MB", stats.wired_mb);
                println!("🗜️  压缩: {} MB", stats.compressed_mb);
                println!("📊 压力级别: {:?}", stats.pressure);

                let top_processes = sort_and_take_processes(processes, 10);
                println!("\n🔝 Top 10 进程:");
                for (i, p) in top_processes.iter().enumerate() {
                    let frontmost = if p.is_frontmost { " 🎯" } else { "" };
                    println!("  {:2}. {:25} {:>6} MB{}", i + 1,
                             if p.name.len() > 25 { &p.name[..25] } else { &p.name },
                             p.rss_mb, frontmost);
                }
            },
            DataLevel::Verbose => {
                println!("━━━ 完整内存报告 ━━━");
                println!("💾 总内存: {} MB", stats.total_mb);
                println!("🆓 可用内存: {} MB ({:.2}%)", stats.free_mb, (stats.free_mb as f64 / stats.total_mb as f64) * 100.0);
                println!("🔥 活跃内存: {} MB ({:.2}%)", stats.active_mb, (stats.active_mb as f64 / stats.total_mb as f64) * 100.0);
                println!("💤 非活跃内存: {} MB ({:.2}%)", stats.inactive_mb, (stats.inactive_mb as f64 / stats.total_mb as f64) * 100.0);
                println!("🔒 有线内存: {} MB ({:.2}%)", stats.wired_mb, (stats.wired_mb as f64 / stats.total_mb as f64) * 100.0);
                println!("🗜️  压缩内存: {} MB ({:.2}%)", stats.compressed_mb, (stats.compressed_mb as f64 / stats.total_mb as f64) * 100.0);
                println!("📊 压力级别: {:?}", stats.pressure);

                // 显示更多进程
                let top_processes = sort_and_take_processes(processes.clone(), 15);
                println!("\n🔝 Top 15 进程 (按内存使用):");
                println!("{:>6} {:25} {:>8} {:>8} {}", "PID", "进程名", "内存(MB)", "CPU%", "状态");
                println!("{:-^6} {:-^25} {:-^8} {:-^8} {:-^6}", "", "", "", "", "");
                for p in &top_processes {
                    let status = if p.is_frontmost { "前台" } else { "后台" };
                    println!("{:>6} {:25} {:>8} {:>7.3}% {}",
                             p.pid,
                             if p.name.len() > 25 { &p.name[..25] } else { &p.name },
                             p.rss_mb,
                             p.cpu_usage * 100.0,
                             status);
                }

                // 候选清理进程
                println!("\n🎯 候选清理进程:");
                let candidates = get_candidate_processes(&processes, 50, &std::collections::HashSet::new(), &std::collections::HashSet::new());
                if candidates.is_empty() {
                    println!("  无候选进程");
                } else {
                    for (i, p) in candidates.iter().take(5).enumerate() {
                        println!("  {}. {} (PID: {}) - {} MB", i + 1, p.name, p.pid, p.rss_mb);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_slash_command(&mut self, cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let command = parts[0];

        match command {
            "/viz" => {
                if let Some(arg) = parts.get(1) {
                    match *arg {
                        "minimal" | "m" => {
                            self.visualization_level = VisualizationLevel::Minimal;
                            println!("🔲 设置可视化级别: Minimal");
                        },
                        "standard" | "s" => {
                            self.visualization_level = VisualizationLevel::Standard;
                            println!("🎨 设置可视化级别: Standard");
                        },
                        "enhanced" | "e" => {
                            self.visualization_level = VisualizationLevel::Enhanced;
                            println!("✨ 设置可视化级别: Enhanced");
                        },
                        "rich" | "r" => {
                            self.visualization_level = VisualizationLevel::Rich;
                            println!("🎆 设置可视化级别: Rich");
                        },
                        _ => {
                            println!("❓ 无效的可视化级别。可选: minimal(m), standard(s), enhanced(e), rich(r)");
                        }
                    }
                } else {
                    println!("🎨 当前可视化级别: {:?}", self.visualization_level);
                    println!("💡 使用 /viz [minimal|standard|enhanced|rich] 更改");
                }
            },
            "/level" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "low" | "l" => {
                            self.boost_level = BoostLevel::Low;
                            println!("💚 设置清理强度: Low");
                        },
                        "mid" | "m" => {
                            self.boost_level = BoostLevel::Mid;
                            println!("💙 设置清理强度: Mid");
                        },
                        "high" | "h" => {
                            self.boost_level = BoostLevel::High;
                            println!("💜 设置清理强度: High");
                        },
                        "killer" | "k" => {
                            self.boost_level = BoostLevel::Killer;
                            println!("💀 设置清理强度: Killer");
                            println!("⚠️  警告: Killer模式将主动终止高内存进程!");
                        },
                        _ => {
                            println!("❓ 无效的强度级别。可选: low(l), mid(m), high(h), killer(k)");
                        }
                    }
                } else {
                    println!("🎛️  当前清理强度: {:?}", self.boost_level);
                    println!("💡 使用 /level [low|mid|high|killer] 更改");
                }
            },
            "/data" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "minimal" | "min" | "m" => {
                            self.data_level = DataLevel::Minimal;
                            println!("📊 设置数据详细度: 最少");
                        },
                        "standard" | "std" | "s" => {
                            self.data_level = DataLevel::Standard;
                            println!("📈 设置数据详细度: 标准");
                        },
                        "detailed" | "det" | "d" => {
                            self.data_level = DataLevel::Detailed;
                            println!("📋 设置数据详细度: 详细");
                        },
                        "verbose" | "verb" | "v" => {
                            self.data_level = DataLevel::Verbose;
                            println!("📜 设置数据详细度: 冗长");
                        },
                        _ => {
                            println!("❓ 无效的数据级别。可选: minimal(m), standard(s), detailed(d), verbose(v)");
                        }
                    }
                } else {
                    println!("📊 当前数据详细度: {:?}", self.data_level);
                    println!("💡 使用 /data [minimal|standard|detailed|verbose] 更改");
                }
            },
            "/export" => {
                if let Some(ref result) = self.last_boost_result {
                    if parts.len() > 1 {
                        let format = match parts[1] {
                            "json" | "j" => ExportFormat::Json,
                            "csv" | "c" => ExportFormat::Csv,
                            "txt" | "t" => ExportFormat::Txt,
                            "markdown" | "md" | "m" => ExportFormat::Markdown,
                            _ => {
                                println!("❓ 无效的格式。可选: json(j), csv(c), txt(t), markdown(md)");
                                return Ok(());
                            }
                        };
                        self.export_last_result(result, format)?;
                    } else {
                        println!("📤 可导出格式: json, csv, txt, markdown");
                        println!("💡 使用 /export [format] 导出最后一次清理结果");
                    }
                } else {
                    println!("❌ 没有可导出的清理结果。请先运行 'boost'");
                }
            },
            "/history" => {
                println!("📜 会话历史:");
                for (i, cmd) in self.session_history.iter().enumerate() {
                    println!("  {}. {}", i + 1, cmd);
                }
            },
            "/logs" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "info" => {
                            match get_logs_size() {
                                Ok(size) => {
                                    let size_mb = size as f64 / 1024.0 / 1024.0;
                                    println!("📊 日志大小: {:.2} MB", size_mb);

                                    if let Ok(files) = list_log_files() {
                                        println!("📁 日志文件: {} 个", files.len());
                                    }
                                },
                                Err(e) => println!("❌ 获取日志信息失败: {}", e),
                            }
                        },
                        "list" => {
                            match list_log_files() {
                                Ok(files) => {
                                    if files.is_empty() {
                                        println!("📁 暂无日志文件");
                                    } else {
                                        println!("📁 日志文件列表:");
                                        for (name, size) in files {
                                            let size_kb = size as f64 / 1024.0;
                                            println!("  📄 {} ({:.1} KB)", name, size_kb);
                                        }
                                    }
                                },
                                Err(e) => println!("❌ 列出日志文件失败: {}", e),
                            }
                        },
                        _ => {
                            println!("❓ 无效的日志命令。可选: info, list");
                        }
                    }
                } else {
                    println!("📋 日志命令: /logs [info|list]");
                }
            },
            "/help" => {
                self.print_help();
            },
            _ => {
                println!("❓ 未知的斜杠命令: {}", command);
                println!("💡 输入 '/help' 查看所有可用命令");
            }
        }

        Ok(())
    }

    fn print_boost_result(&self, result: &BoostResult) {
        match self.data_level {
            DataLevel::Minimal => {
                if result.delta_mb > 0 {
                    println!("✅ 释放了 {} MB 内存", result.delta_mb);
                } else {
                    println!("ℹ️  内存清理完成，变化: {} MB", result.delta_mb);
                }
            },
            DataLevel::Standard => {
                println!("━━━ 清理结果 ━━━");
                println!("⏱️  耗时: {:.2}s", result.duration.as_secs_f64());
                if result.delta_mb > 0 {
                    println!("✅ 释放内存: {} MB", result.delta_mb);
                } else {
                    println!("ℹ️  内存变化: {} MB", result.delta_mb);
                }
                println!("📊 清理前: {} MB 可用", result.before.free_mb);
                println!("📊 清理后: {} MB 可用", result.after.free_mb);
            },
            DataLevel::Detailed | DataLevel::Verbose => {
                println!("━━━ 详细清理结果 ━━━");
                println!("⏱️  执行时间: {:.3}s", result.duration.as_secs_f64());
                println!("🎯 清理强度: {:?}", self.boost_level);

                if result.delta_mb > 0 {
                    println!("✅ 成功释放: {} MB", result.delta_mb);
                } else {
                    println!("ℹ️  内存变化: {} MB", result.delta_mb);
                }

                println!("\n📊 内存对比:");
                println!("  🔄 清理前: {} MB 可用 / {} MB 总计", result.before.free_mb, result.before.total_mb);
                println!("  ✨ 清理后: {} MB 可用 / {} MB 总计", result.after.free_mb, result.after.total_mb);

                let improvement = ((result.after.free_mb as f64 - result.before.free_mb as f64) / result.before.total_mb as f64) * 100.0;
                println!("  📈 改善度: {:.2}%", improvement);

                println!("\n🎯 压力状态:");
                println!("  清理前: {:?}", result.before.pressure);
                println!("  清理后: {:?}", result.after.pressure);
            }
        }
    }

    fn export_last_result(&self, result: &BoostResult, format: ExportFormat) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = match format {
            ExportFormat::Json => format!("boost_result_{}.json", timestamp),
            ExportFormat::Csv => format!("boost_result_{}.csv", timestamp),
            ExportFormat::Txt => format!("boost_result_{}.txt", timestamp),
            ExportFormat::Markdown => format!("boost_result_{}.md", timestamp),
        };

        let content = match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&serde_json::json!({
                    "timestamp": timestamp,
                    "boost_level": format!("{:?}", self.boost_level),
                    "duration_seconds": result.duration.as_secs_f64(),
                    "delta_mb": result.delta_mb,
                    "before": {
                        "free_mb": result.before.free_mb,
                        "total_mb": result.before.total_mb,
                        "pressure": format!("{:?}", result.before.pressure)
                    },
                    "after": {
                        "free_mb": result.after.free_mb,
                        "total_mb": result.after.total_mb,
                        "pressure": format!("{:?}", result.after.pressure)
                    }
                }))?
            },
            ExportFormat::Csv => {
                format!("Timestamp,BoostLevel,DurationSeconds,DeltaMB,BeforeFreeMB,AfterFreeMB,BeforePressure,AfterPressure\n{},{:?},{:.3},{},{},{},{:?},{:?}",
                    timestamp,
                    self.boost_level,
                    result.duration.as_secs_f64(),
                    result.delta_mb,
                    result.before.free_mb,
                    result.after.free_mb,
                    result.before.pressure,
                    result.after.pressure
                )
            },
            ExportFormat::Txt => {
                format!("RAM Booster 清理报告\n===================\n\n时间戳: {}\n清理强度: {:?}\n执行时间: {:.3}s\n释放内存: {} MB\n\n清理前状态:\n  可用内存: {} MB\n  内存压力: {:?}\n\n清理后状态:\n  可用内存: {} MB\n  内存压力: {:?}\n",
                    timestamp,
                    self.boost_level,
                    result.duration.as_secs_f64(),
                    result.delta_mb,
                    result.before.free_mb,
                    result.before.pressure,
                    result.after.free_mb,
                    result.after.pressure
                )
            },
            ExportFormat::Markdown => {
                format!("# RAM Booster 清理报告\n\n**时间:** {}\n**清理强度:** {:?}\n**执行时间:** {:.3}s\n**释放内存:** {} MB\n\n## 清理前后对比\n\n| 项目 | 清理前 | 清理后 | 变化 |\n|------|--------|--------|------|\n| 可用内存 (MB) | {} | {} | {:+} |\n| 内存压力 | {:?} | {:?} | - |\n\n> 报告生成时间: {}\n",
                    timestamp,
                    self.boost_level,
                    result.duration.as_secs_f64(),
                    result.delta_mb,
                    result.before.free_mb,
                    result.after.free_mb,
                    result.after.free_mb as i64 - result.before.free_mb as i64,
                    result.before.pressure,
                    result.after.pressure,
                    Utc::now().to_rfc3339()
                )
            }
        };

        fs::write(&filename, content)?;
        println!("📁 导出成功: {}", filename);

        Ok(())
    }

    fn print_help(&self) {
        println!("━━━ 命令帮助 ━━━");
        println!("🎯 核心命令:");
        println!("  b/boost         - 执行内存清理");
        println!("  status          - 查看内存状态");
        println!("  clear           - 清屏");
        println!("  exit/quit/q     - 退出程序");
        println!();
        println!("⚙️  配置命令:");
        println!("  /level [强度]    - 设置清理强度 (low/mid/high/killer)");
        println!("                     💀 killer: 杀手模式，多轮清理+进程终止");
        println!("  /data [级别]     - 设置显示详细度 (minimal/standard/detailed/verbose)");
        println!("  /viz [级别]      - 设置可视化级别 (minimal/standard/enhanced/rich)");
        println!("                     🔲 minimal: 最简显示 | 🎨 standard: 进度条");
        println!("                     ✨ enhanced: 彩色效果 | 🎆 rich: 全面动画");
        println!();
        println!("📤 导出命令:");
        println!("  /export [格式]   - 导出最后清理结果 (json/csv/txt/markdown)");
        println!();
        println!("📋 其他命令:");
        println!("  /history        - 查看命令历史");
        println!("  /logs [操作]     - 日志管理 (info/list)");
        println!("  /help          - 显示此帮助");
    }
}