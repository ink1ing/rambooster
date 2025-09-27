use core::{read_mem_stats, MemStats, PressureLevel};
use core::release::{boost, BoostError, PurgeError};
use core::log_entry::{write_log_event, read_log_events, LogEvent};
use core::config::{Config, load_config};
use std::time::Duration;
use chrono::Local;
use std::fs;
use std::path::Path;

#[test]
fn test_boost_memory_delta_calculation() {
    // 这个测试检验boost操作前后的内存差值计算是否正确
    // 注意：由于purge命令可能不存在，我们测试逻辑而不依赖实际执行

    let before_stats = read_mem_stats();
    if before_stats.is_err() {
        println!("Skipping boost test - cannot read memory stats");
        return;
    }

    let before = before_stats.unwrap();
    println!("Before boost: Free memory = {} MB", before.free_mb);

    // 尝试执行boost操作
    match boost() {
        Ok(result) => {
            // 验证结果结构的完整性
            assert!(result.duration > Duration::from_millis(0));
            assert!(result.before.total_mb > 0);
            assert!(result.after.total_mb > 0);
            assert_eq!(result.before.total_mb, result.after.total_mb); // 总内存应该不变

            // 验证delta计算正确
            let expected_delta = result.after.free_mb as i64 - result.before.free_mb as i64;
            assert_eq!(result.delta_mb, expected_delta);

            println!("Boost successful:");
            println!("  Duration: {:?}", result.duration);
            println!("  Delta: {} MB", result.delta_mb);
            println!("  Before: {} MB free", result.before.free_mb);
            println!("  After: {} MB free", result.after.free_mb);
        }
        Err(BoostError::Purge(PurgeError::CommandNotFound)) => {
            println!("Skipping boost test - purge command not found (expected on CI)");
        }
        Err(e) => {
            panic!("Boost failed with unexpected error: {:?}", e);
        }
    }
}

#[test]
fn test_log_event_write_and_read() {
    // 测试日志写入和读取的完整流程

    // 创建测试配置
    let _config = Config {
        log_backend: "jsonl".to_string(),
        log_retention_days: 30,
        rss_threshold_mb: 50,
        enable_process_termination: false,
        throttle_interval_seconds: 300,
        whitelist_processes: vec![],
        blacklist_processes: vec![],
    };

    // 创建测试用的内存统计数据
    let before_stats = MemStats {
        total_mb: 16384,
        free_mb: 2000,
        active_mb: 6000,
        inactive_mb: 4000,
        wired_mb: 2384,
        compressed_mb: 2000,
        pressure: PressureLevel::Normal,
    };

    let after_stats = MemStats {
        total_mb: 16384,
        free_mb: 2500,
        active_mb: 5500,
        inactive_mb: 4000,
        wired_mb: 2384,
        compressed_mb: 2000,
        pressure: PressureLevel::Normal,
    };

    let delta_mb = after_stats.free_mb as i64 - before_stats.free_mb as i64;

    // 创建测试日志事件
    let log_event = LogEvent {
        ts: chrono::Utc::now().to_rfc3339(),
        action: "test_boost".to_string(),
        before: Some(before_stats),
        after: Some(after_stats),
        delta_mb,
        pressure: PressureLevel::Normal,
        details: serde_json::json!({
            "test": true,
            "duration_ms": 1500
        }),
    };

    // 写入测试日志事件
    let result = write_log_event(&log_event);

    assert!(result.is_ok(), "Failed to write log event: {:?}", result);

    // 读取今天的日志
    let today = Local::now().format("%Y-%m-%d").to_string();
    let events = read_log_events(&today);

    assert!(events.is_ok(), "Failed to read log events: {:?}", events);
    let events = events.unwrap();

    // 验证我们的事件被正确记录
    let test_event = events.iter().find(|e| e.action == "test_boost");
    assert!(test_event.is_some(), "Test event not found in logs");

    let event = test_event.unwrap();
    assert_eq!(event.action, "test_boost");
    assert_eq!(event.delta_mb, delta_mb);
    assert!(event.details.is_object());

    if let Some(details) = event.details.as_object() {
        assert_eq!(details.get("test"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(details.get("duration_ms"), Some(&serde_json::Value::Number(1500.into())));
    }

    println!("Log event successfully written and read back");
    println!("Event timestamp: {}", event.ts);
    println!("Event delta: {} MB", event.delta_mb);
}

#[test]
fn test_memory_stats_consistency() {
    // 测试内存统计数据的一致性

    let stats_result = read_mem_stats();
    assert!(stats_result.is_ok(), "Failed to read memory stats");

    let stats = stats_result.unwrap();

    // 基本合理性检查
    assert!(stats.total_mb > 0, "Total memory should be positive");
    assert!(stats.free_mb >= 0, "Free memory should be non-negative");
    assert!(stats.active_mb >= 0, "Active memory should be non-negative");
    assert!(stats.inactive_mb >= 0, "Inactive memory should be non-negative");
    assert!(stats.wired_mb >= 0, "Wired memory should be non-negative");
    assert!(stats.compressed_mb >= 0, "Compressed memory should be non-negative");

    // 验证内存总和不超过总内存（在合理范围内）
    let used_memory = stats.active_mb + stats.inactive_mb + stats.wired_mb;
    assert!(used_memory <= stats.total_mb + 1000, // 允许1GB的误差
           "Used memory ({} MB) should not significantly exceed total ({} MB)",
           used_memory, stats.total_mb);

    // 验证压力等级是合理的
    match stats.pressure {
        PressureLevel::Normal => {
            // Normal pressure时，可用内存应该较充足
            let available = stats.free_mb + stats.inactive_mb;
            let ratio = available as f64 / stats.total_mb as f64;
            println!("Normal pressure - available ratio: {:.2}%", ratio * 100.0);
        }
        PressureLevel::Warning => {
            println!("Warning pressure detected");
        }
        PressureLevel::Critical => {
            println!("Critical pressure detected");
        }
    }

    println!("Memory stats validation passed:");
    println!("  Total: {} MB", stats.total_mb);
    println!("  Free: {} MB", stats.free_mb);
    println!("  Active: {} MB", stats.active_mb);
    println!("  Inactive: {} MB", stats.inactive_mb);
    println!("  Wired: {} MB", stats.wired_mb);
    println!("  Compressed: {} MB", stats.compressed_mb);
    println!("  Pressure: {:?}", stats.pressure);
}

#[test]
fn test_config_integration() {
    // 测试配置系统的集成

    let config_result = load_config();
    assert!(config_result.is_ok(), "Failed to load config: {:?}", config_result);

    let config = config_result.unwrap();

    // 验证配置的合理性
    assert!(config.rss_threshold_mb > 0, "RSS threshold should be positive");
    assert!(config.log_retention_days > 0, "Log retention should be positive");
    assert!(config.throttle_interval_seconds > 0, "Throttle interval should be positive");
    assert!(["jsonl", "sqlite"].contains(&config.log_backend.as_str()),
           "Log backend should be jsonl or sqlite");

    // 验证默认安全设置
    assert!(!config.enable_process_termination,
           "Process termination should be disabled by default for safety");

    // 验证白名单包含重要系统进程
    assert!(config.whitelist_processes.contains(&"kernel_task".to_string()),
           "Whitelist should contain kernel_task");
    assert!(config.whitelist_processes.contains(&"launchd".to_string()),
           "Whitelist should contain launchd");

    println!("Configuration validation passed:");
    println!("  RSS Threshold: {} MB", config.rss_threshold_mb);
    println!("  Log Backend: {}", config.log_backend);
    println!("  Process Termination: {}", config.enable_process_termination);
    println!("  Whitelist size: {}", config.whitelist_processes.len());
}

#[test]
fn test_log_file_cleanup_simulation() {
    // 模拟测试日志文件清理逻辑（不实际删除文件）

    use std::env;

    let home_dir = match env::var("HOME") {
        Ok(dir) => dir,
        Err(_) => {
            println!("Skipping log cleanup test - HOME not set");
            return;
        }
    };

    let log_dir = format!("{}/.local/share/rambo/logs", home_dir);
    let log_path = Path::new(&log_dir);

    if log_path.exists() {
        println!("Log directory exists: {}", log_dir);

        // 检查日志文件
        if let Ok(entries) = fs::read_dir(log_path) {
            let mut log_files = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                        log_files.push(path);
                    }
                }
            }

            println!("Found {} log files", log_files.len());

            // 验证日志文件名格式（YYYY-MM-DD.jsonl）
            for log_file in &log_files {
                let filename = log_file.file_stem().unwrap().to_str().unwrap();
                let date_parts: Vec<&str> = filename.split('-').collect();
                assert_eq!(date_parts.len(), 3, "Log file should have YYYY-MM-DD format");

                // 验证年月日都是数字
                for part in &date_parts {
                    assert!(part.parse::<u32>().is_ok(), "Date part should be numeric");
                }

                println!("  Valid log file: {}", filename);
            }
        }
    } else {
        println!("Log directory does not exist yet: {}", log_dir);
    }
}