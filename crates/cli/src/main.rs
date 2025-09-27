use clap::{Parser, Subcommand};
use rambo_core::processes::{get_all_processes, sort_and_take_processes, ProcessInfo};
use rambo_core::release::{terminate, get_candidate_processes, boost, BoostResult};
use rambo_core::{read_mem_stats, MemStats};
use rambo_core::log_entry::{read_log_events, LogEvent, cleanup_old_logs, clear_all_logs, get_logs_size, list_log_files};
use rambo_core::config::load_config;
use rambo_core::daemon::{Daemon, install_launchd_agent, uninstall_launchd_agent};
use rambo_core::security::{filter_safe_processes, require_confirmation};
use serde::Serialize;
use chrono::Utc;
use std::collections::HashSet;
use std::path::Path;
use std::io::Write;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Override RSS threshold in MB
    #[arg(long, global = true)]
    rss_threshold: Option<u64>,

    /// Override log backend (jsonl or sqlite)
    #[arg(long, global = true)]
    log_backend: Option<String>,

    /// Enable process termination
    #[arg(long, global = true)]
    enable_termination: Option<bool>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current memory stats and top processes
    Status(StatusArgs),
    /// Free up memory by running the purge command
    Boost(BoostArgs),
    /// Suggest processes that can be terminated to free memory
    Suggest(SuggestArgs),
    /// Terminate a process by its PID
    Kill(KillArgs),
    /// Show logs for a specific day
    Log(LogArgs),
    /// Manage log files (cleanup, clear, info)
    Logs(LogsArgs),
    /// Run diagnostics to check for required tools and permissions
    Doctor,
    /// Run as a background daemon to monitor memory pressure
    Daemon(DaemonArgs),
}

#[derive(Parser)]
struct StatusArgs {
    /// Output in JSON format
    #[arg(long)]
    json: bool,

    /// Number of top processes to show
    #[arg(long, default_value_t = 10)]
    top: usize,
}

#[derive(Parser)]
struct LogArgs {
    /// The date to show logs for (YYYY-MM-DD). Defaults to today.
    #[arg(default_value_t = Utc::now().format("%Y-%m-%d").to_string())]
    date: String,
}

#[derive(Parser)]
struct KillArgs {
    /// The Process ID to terminate
    pid: u32,

    /// Force kill (SIGKILL) without waiting for graceful shutdown (SIGTERM)
    #[arg(long)]
    force: bool,
}

#[derive(Parser)]
struct SuggestArgs {
    /// Output in JSON format
    #[arg(long)]
    json: bool,

    /// RSS threshold in MB for a process to be considered a candidate
    #[arg(long, default_value_t = 50)]
    rss_threshold: u64,
}

#[derive(Parser)]
struct BoostArgs {
    /// Output in JSON format
    #[arg(long)]
    json: bool,
}

#[derive(Parser)]
struct DaemonArgs {
    /// Install launchd agent for automatic startup
    #[arg(long)]
    install: bool,

    /// Uninstall launchd agent
    #[arg(long)]
    uninstall: bool,

    /// Run in foreground (don't daemonize)
    #[arg(long)]
    foreground: bool,
}

#[derive(Parser)]
struct LogsArgs {
    #[command(subcommand)]
    action: LogsAction,
}

#[derive(Subcommand)]
enum LogsAction {
    /// Show information about log files
    Info,
    /// Cleanup old log files based on retention policy
    Cleanup,
    /// Clear all log files
    Clear {
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// List all log files
    List,
}

#[derive(Serialize)]
struct StatusOutput {
    mem_stats: MemStats,
    processes: Vec<rambo_core::processes::ProcessInfo>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load configuration (defaults → file → env vars → CLI flags)
    let mut config = load_config().map_err(|e| format!("Failed to load config: {}", e))?;

    // Override with CLI flags
    if let Some(threshold) = cli.rss_threshold {
        config.rss_threshold_mb = threshold;
    }

    if let Some(backend) = &cli.log_backend {
        config.log_backend = backend.clone();
    }

    if let Some(enable) = cli.enable_termination {
        config.enable_process_termination = enable;
    }

    match &cli.command {
        Commands::Status(args) => {
            let mem_stats = read_mem_stats()?;
            let processes = get_all_processes();
            let top_processes = sort_and_take_processes(processes, args.top);

            if args.json {
                let output = StatusOutput {
                    mem_stats,
                    processes: top_processes,
                };
                let json_string = serde_json::to_string_pretty(&output)?;
                println!("{}", json_string);
            } else {
                print_status_human(&mem_stats, &top_processes);
            }
        }
        Commands::Boost(args) => {
            println!("Boosting memory... This may take a moment.");
            match boost() {
                Ok(boost_result) => {
                    if args.json {
                        let json_string = serde_json::to_string_pretty(&boost_result)?;
                        println!("{}", json_string);
                    } else {
                        print_boost_human(&boost_result);
                    }
                }
                Err(e) => {
                    match e {
                        rambo_core::release::BoostError::Purge(rambo_core::release::PurgeError::CommandNotFound) => {
                            eprintln!("Error: /usr/bin/purge command not found.");
                            eprintln!("Please install Xcode Command Line Tools and try again.");
                            eprintln!("You can install them by running: xcode-select --install");
                            std::process::exit(1);
                        }
                        _ => {
                            return Err(format!("Boost failed: {:?}", e).into());
                        }
                    }
                }
            }
        }
        Commands::Suggest(args) => {
            let all_processes = get_all_processes();

            // Use threshold from CLI args or config
            let threshold = if args.rss_threshold != 50 {
                args.rss_threshold
            } else {
                config.rss_threshold_mb
            };

            let whitelist: HashSet<String> = config.whitelist_processes.iter().cloned().collect();
            let blacklist: HashSet<String> = config.blacklist_processes.iter().cloned().collect();

            let candidates = get_candidate_processes(
                &all_processes,
                threshold,
                &whitelist,
                &blacklist,
            );

            // Apply additional safety filtering - convert back to owned processes first
            let candidate_processes: Vec<ProcessInfo> = candidates.iter().map(|&p| p.clone()).collect();
            let safe_candidates = filter_safe_processes(&candidate_processes, false); // Only show safe processes

            if args.json {
                let json_string = serde_json::to_string_pretty(&safe_candidates)?;
                println!("{}", json_string);
            } else {
                print_suggest_human(&safe_candidates);
            }
        }
        Commands::Kill(args) => {
            // Check if process termination is enabled in config
            if !config.enable_process_termination {
                eprintln!("Process termination is disabled in configuration.");
                eprintln!("To enable, set enable_process_termination = true in config or use --enable-termination flag.");
                std::process::exit(1);
            }

            // Find the process to get its info for safety checking
            let all_processes = get_all_processes();
            let target_process = all_processes.iter().find(|p| p.pid == args.pid);

            match target_process {
                Some(process) => {
                    // Use security module for confirmation
                    if require_confirmation(process) {
                        println!("Terminating process {}...", args.pid);
                        let success = terminate(args.pid, args.force);
                        if success {
                            println!("Process {} terminated successfully.", args.pid);
                        } else {
                            eprintln!("Failed to terminate process {}. It might not exist or you may not have permission.", args.pid);
                        }
                    } else {
                        println!("Termination cancelled.");
                    }
                }
                None => {
                    eprintln!("Process with PID {} not found.", args.pid);
                }
            }
        }
        Commands::Log(args) => {
            let events = read_log_events(&args.date)?;
            if events.is_empty() {
                println!("No logs found for {}.\n", args.date);
            } else {
                print_logs_human(&events);
            }
        }
        Commands::Logs(args) => {
            match &args.action {
                LogsAction::Info => {
                    match get_logs_size() {
                        Ok(total_size) => {
                            let size_mb = total_size as f64 / 1024.0 / 1024.0;
                            println!("--- Log Information ---");
                            println!("Total log size: {:.2} MB ({} bytes)", size_mb, total_size);

                            match list_log_files() {
                                Ok(files) => {
                                    println!("Log files ({}):", files.len());
                                    for (filename, size) in files {
                                        let file_size_kb = size as f64 / 1024.0;
                                        println!("  {}: {:.1} KB", filename, file_size_kb);
                                    }
                                }
                                Err(e) => eprintln!("Failed to list log files: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Failed to get log information: {}", e),
                    }
                }
                LogsAction::Cleanup => {
                    match cleanup_old_logs(config.log_retention_days) {
                        Ok(deleted_count) => {
                            if deleted_count > 0 {
                                println!("Cleaned up {} old log files (older than {} days)",
                                         deleted_count, config.log_retention_days);
                            } else {
                                println!("No old log files to clean up");
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to cleanup logs: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                LogsAction::Clear { yes } => {
                    if !yes {
                        print!("Are you sure you want to clear ALL log files? [y/N]: ");
                        std::io::stdout().flush().unwrap();
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        if !input.trim().to_lowercase().starts_with('y') {
                            println!("Operation cancelled.");
                            return Ok(());
                        }
                    }

                    match clear_all_logs() {
                        Ok(deleted_count) => {
                            println!("Cleared {} log files", deleted_count);
                        }
                        Err(e) => {
                            eprintln!("Failed to clear logs: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                LogsAction::List => {
                    match list_log_files() {
                        Ok(files) => {
                            if files.is_empty() {
                                println!("No log files found");
                            } else {
                                println!("--- Log Files ---");
                                println!("{:<12} {:>10}", "Date", "Size (KB)");
                                println!("{:-<12} {:->10}", "", "");
                                for (filename, size) in files {
                                    let file_size_kb = size as f64 / 1024.0;
                                    println!("{:<12} {:>10.1}", filename, file_size_kb);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to list log files: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Doctor => {
            println!("--- RAM Booster Doctor ---");

            // 1. Check for `purge` command
            let purge_path = Path::new("/usr/bin/purge");
            if purge_path.exists() {
                println!("[✓] /usr/bin/purge command found.");
            } else {
                println!("[✗] /usr/bin/purge command not found.");
                println!("    ➔ Memory boosting will not work.");
                println!("    ➔ To fix, install Xcode Command Line Tools: xcode-select --install");
            }

            // 2. Show current configuration
            println!("\n--- Current Configuration ---");
            println!("RSS Threshold: {} MB", config.rss_threshold_mb);
            println!("Log Backend: {}", config.log_backend);
            println!("Log Retention: {} days", config.log_retention_days);
            println!("Process Termination: {}", if config.enable_process_termination { "enabled" } else { "disabled" });
            println!("Throttle Interval: {} seconds", config.throttle_interval_seconds);
            println!("Whitelist: {:?}", config.whitelist_processes);
            println!("Blacklist: {:?}", config.blacklist_processes);

            // 3. Check for permissions
            println!("\n--- Permissions ---");
            check_permissions();

            // 4. Check for launchd agent
            println!("\n--- LaunchAgent Status ---");
            check_launchd_agent_status();
            println!("\nDoctor check complete.");
        }
        Commands::Daemon(args) => {
            if args.install {
                match install_launchd_agent(&config) {
                    Ok(()) => {
                        println!("LaunchAgent installed successfully.");
                        println!("The daemon will start automatically at login.");
                        println!("To start it now, run: launchctl load ~/Library/LaunchAgents/com.rambo.daemon.plist");
                    }
                    Err(e) => {
                        eprintln!("Failed to install LaunchAgent: {}", e);
                        std::process::exit(1);
                    }
                }
            } else if args.uninstall {
                match uninstall_launchd_agent() {
                    Ok(()) => {
                        println!("LaunchAgent uninstalled successfully.");
                    }
                    Err(e) => {
                        eprintln!("Failed to uninstall LaunchAgent: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Run daemon
                if !args.foreground {
                    println!("Starting daemon in background...");
                    println!("Use --foreground to run in foreground mode");
                    println!("Use --install to install as a LaunchAgent");
                    println!("Logs will be written to ~/Library/Logs/rambo-daemon.log");
                }

                let mut daemon = Daemon::new(config);
                if let Err(e) = daemon.run() {
                    eprintln!("Daemon failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn check_permissions() {
    use std::process::Command;

    // Check if we can read memory stats
    match rambo_core::read_mem_stats() {
        Ok(_) => println!("[✓] Memory statistics access: OK"),
        Err(e) => {
            println!("[✗] Memory statistics access failed: {}", e);
            println!("    ➔ This may require additional permissions on some systems");
        }
    }

    // Check if we can list processes
    let processes = rambo_core::processes::get_all_processes();
    if processes.is_empty() {
        println!("[✗] Process listing: Failed (no processes found)");
        println!("    ➔ This may require additional permissions");
    } else {
        println!("[✓] Process listing: OK ({} processes found)", processes.len());
    }

    // Check if we have write access to config directory
    match rambo_core::config::get_config_path() {
        Ok(config_path) => {
            let parent_dir = config_path.parent().unwrap();
            if parent_dir.exists() {
                println!("[✓] Config directory access: OK");
            } else {
                println!("[!] Config directory not found (will be created when needed)");
            }
        }
        Err(e) => {
            println!("[✗] Config directory access failed: {}", e);
        }
    }

    // Check if we have write access to log directory
    match std::env::var("HOME") {
        Ok(home) => {
            let log_dir = format!("{}/.local/share/rambo/logs", home);
            let log_path = Path::new(&log_dir);
            if log_path.exists() {
                println!("[✓] Log directory access: OK");
            } else {
                println!("[!] Log directory not found (will be created when needed)");
            }
        }
        Err(_) => {
            println!("[✗] Could not determine home directory for log access check");
        }
    }
}

fn check_launchd_agent_status() {
    use std::process::Command;
    use std::env;

    let home_dir = match env::var("HOME") {
        Ok(dir) => dir,
        Err(_) => {
            println!("[✗] Could not determine home directory");
            return;
        }
    };

    let plist_path = format!("{}/Library/LaunchAgents/com.rambo.daemon.plist", home_dir);
    let plist_exists = Path::new(&plist_path).exists();

    if plist_exists {
        println!("[✓] LaunchAgent plist file found: {}", plist_path);

        // Check if the agent is loaded
        let output = Command::new("launchctl")
            .args(&["list", "com.rambo.daemon"])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.contains("com.rambo.daemon") {
                        println!("[✓] LaunchAgent is loaded and running");

                        // Try to parse the PID from the output
                        for line in stdout.lines() {
                            if let Ok(pid) = line.trim().parse::<u32>() {
                                if pid > 0 {
                                    println!("    ➔ Running with PID: {}", pid);
                                    break;
                                }
                            }
                        }
                    } else {
                        println!("[!] LaunchAgent is loaded but may not be running properly");
                    }
                } else {
                    println!("[!] LaunchAgent is not loaded");
                    println!("    ➔ To load: launchctl load {}", plist_path);
                }
            }
            Err(e) => {
                println!("[✗] Failed to check LaunchAgent status: {}", e);
                println!("    ➔ launchctl may not be available");
            }
        }
    } else {
        println!("[!] LaunchAgent not installed");
        println!("    ➔ To install: rambo daemon --install");
        println!("    ➔ Plist would be created at: {}", plist_path);
    }
}

fn print_logs_human(events: &[LogEvent]) {
    println!("--- Logs ---");
    for event in events {
        println!("[{}] Action: {}", event.ts, event.action);
        if event.delta_mb != 0 {
            println!("  Delta: {} MB", event.delta_mb);
        }
        if let Some(details) = event.details.as_object() {
            if !details.is_empty() {
                println!("  Details: {}", serde_json::to_string(details).unwrap_or_default());
            }
        }
    }
}

fn print_suggest_human(candidates: &[&rambo_core::processes::ProcessInfo]) {
    if candidates.is_empty() {
        println!("No candidate processes found to terminate.");
        return;
    }

    println!("--- Candidate Processes to Terminate ---");
    println!("{:<6} {:<25} {:>10}", "PID", "Name", "RSS (MB)");
    println!("{:-<6} {:-<25} {:->10}", "", "", "");

    for p in candidates {
        let name = if p.name.len() > 23 {
            format!("{}...", &p.name[..23])
        } else {
            p.name.clone()
        };
        println!("{:<6} {:<25} {:>10}", p.pid, name, p.rss_mb);
    }
}

fn print_boost_human(result: &BoostResult) {
    println!("\n--- Boost Result ---");
    println!("  Time taken: {:.2}s", result.duration.as_secs_f32());
    if result.delta_mb >= 0 {
        println!("  Memory freed: {} MB", result.delta_mb);
    } else {
        println!("  Memory increased: {} MB", -result.delta_mb);
    }
    println!("\n  Before: {} MB free", result.before.free_mb);
    println!("  After:  {} MB free", result.after.free_mb);
}

fn print_status_human(mem_stats: &MemStats, processes: &[rambo_core::processes::ProcessInfo]) {
    println!("--- Memory Stats ---");
    println!("  Total: {} MB", mem_stats.total_mb);
    println!("  Free: {} MB", mem_stats.free_mb);
    println!("  Active: {} MB", mem_stats.active_mb);
    println!("  Inactive: {} MB", mem_stats.inactive_mb);
    println!("  Wired: {} MB", mem_stats.wired_mb);
    println!("  Compressed: {} MB", mem_stats.compressed_mb);
    println!("  Pressure: {:?}", mem_stats.pressure);
    println!("\n--- Top {} Processes (by memory) ---", processes.len());
    println!("{:<6} {:<25} {:>10}", "PID", "Name", "RSS (MB)");
    println!("{:-<6} {:-<25} {:->10}", "", "", "");

    for p in processes {
        let name = if p.name.len() > 23 {
            format!("{}...", &p.name[..23])
        } else {
            p.name.clone()
        };
        println!("{:<6} {:<25} {:>10}", p.pid, name, p.rss_mb);
    }
}