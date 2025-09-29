use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use crate::config::Config;
use crate::release::boost;
use crate::hotkey::GlobalHotkey;
use crate::{read_mem_stats, PressureLevel};

pub struct Daemon {
    config: Config,
    last_boost: Arc<Mutex<Option<Instant>>>,
    hotkey: Option<GlobalHotkey>,
}

impl Daemon {
    pub fn new(config: Config) -> Self {
        let hotkey = if config.hotkey.enabled {
            Some(GlobalHotkey::new(config.hotkey.clone()))
        } else {
            None
        };

        Self {
            config,
            last_boost: Arc::new(Mutex::new(None)),
            hotkey,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        println!("Starting RAM Booster daemon...");
        println!("Monitoring memory pressure (throttle interval: {}s)", self.config.throttle_interval_seconds);

        // Start hotkey monitoring if enabled
        if let Some(hotkey) = &self.hotkey {
            let last_boost = self.last_boost.clone();
            let throttle_interval = self.config.throttle_interval_seconds;

            if let Err(e) = hotkey.start_monitoring(move || {
                println!("🎹 快捷键 Control+R 被按下，触发内存清理...");

                // 检查throttle
                let should_boost = {
                    let last_boost_guard = last_boost.lock().unwrap();
                    if let Some(last) = *last_boost_guard {
                        let elapsed = last.elapsed();
                        let throttle_duration = Duration::from_secs(throttle_interval);
                        if elapsed < throttle_duration {
                            let remaining = throttle_duration - elapsed;
                            println!("⏱️  内存清理仍在冷却中，请等待 {:.1}s", remaining.as_secs_f32());
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                };

                if should_boost {
                    match boost() {
                        Ok(result) => {
                            // 更新last_boost时间
                            let mut last_boost_guard = last_boost.lock().unwrap();
                            *last_boost_guard = Some(Instant::now());
                            drop(last_boost_guard);

                            println!("✅ 快捷键内存清理完成:");
                            println!("   释放内存: {} MB", result.delta_mb);
                            println!("   用时: {:.2}s", result.duration.as_secs_f32());
                            println!("   可用内存: {} MB → {} MB", result.before.free_mb, result.after.free_mb);
                        }
                        Err(e) => {
                            eprintln!("❌ 快捷键内存清理失败: {:?}", e);
                        }
                    }
                }
            }) {
                eprintln!("警告: 全局快捷键启动失败: {}", e);
                eprintln!("将继续运行内存压力监控，但快捷键功能不可用");
            }
        }

        // Start memory pressure monitoring thread
        let (tx, rx) = mpsc::channel();
        let config = self.config.clone();

        thread::spawn(move || {
            memory_pressure_monitor(tx, config.throttle_interval_seconds);
        });

        // Main daemon loop
        loop {
            match rx.recv() {
                Ok(pressure_level) => {
                    if self.should_trigger_boost(&pressure_level) {
                        self.handle_memory_pressure(pressure_level);
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving memory pressure event: {}", e);
                    return Err(format!("Memory pressure monitoring failed: {}", e));
                }
            }
        }
    }

    fn should_trigger_boost(&self, pressure_level: &PressureLevel) -> bool {
        // Only boost on warning or critical pressure
        if !matches!(pressure_level, PressureLevel::Warning | PressureLevel::Critical) {
            return false;
        }

        // Check throttle interval
        let last_boost_guard = self.last_boost.lock().unwrap();
        if let Some(last_boost) = *last_boost_guard {
            let elapsed = last_boost.elapsed();
            let throttle_duration = Duration::from_secs(self.config.throttle_interval_seconds);
            if elapsed < throttle_duration {
                println!("Memory boost throttled (last boost was {:.1}s ago)", elapsed.as_secs_f32());
                return false;
            }
        }

        true
    }

    fn handle_memory_pressure(&mut self, pressure_level: PressureLevel) {
        println!("Memory pressure detected: {:?}", pressure_level);

        match boost() {
            Ok(result) => {
                let mut last_boost_guard = self.last_boost.lock().unwrap();
                *last_boost_guard = Some(Instant::now());
                drop(last_boost_guard);

                println!("Memory boost completed:");
                println!("  Freed: {} MB in {:.2}s", result.delta_mb, result.duration.as_secs_f32());
                println!("  Free memory: {} MB → {} MB", result.before.free_mb, result.after.free_mb);
            }
            Err(e) => {
                eprintln!("Memory boost failed: {:?}", e);
            }
        }
    }
}

fn memory_pressure_monitor(tx: mpsc::Sender<PressureLevel>, check_interval_secs: u64) {
    let check_interval = Duration::from_secs(std::cmp::max(check_interval_secs / 10, 5)); // Check more frequently than boost interval

    loop {
        match read_mem_stats() {
            Ok(stats) => {
                // Send pressure level if it has changed significantly
                if let Err(_) = tx.send(stats.pressure) {
                    eprintln!("Failed to send memory pressure event - daemon may have stopped");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read memory stats: {}", e);
            }
        }

        thread::sleep(check_interval);
    }
}

pub fn install_launchd_agent(config: &Config) -> Result<(), String> {
    use std::fs;
    use std::env;

    let home_dir = env::var("HOME").map_err(|_| "Could not determine home directory")?;
    let agents_dir = format!("{}/Library/LaunchAgents", home_dir);
    let plist_path = format!("{}/com.rambo.daemon.plist", agents_dir);

    // Create LaunchAgents directory if it doesn't exist
    fs::create_dir_all(&agents_dir)
        .map_err(|e| format!("Failed to create LaunchAgents directory: {}", e))?;

    // Get current executable path
    let exe_path = env::current_exe()
        .map_err(|e| format!("Could not determine executable path: {}", e))?;

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.rambo.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}/Library/Logs/rambo-daemon.log</string>
    <key>StandardErrorPath</key>
    <string>{}/Library/Logs/rambo-daemon-error.log</string>
    <key>ThrottleInterval</key>
    <integer>{}</integer>
</dict>
</plist>"#,
        exe_path.display(),
        home_dir,
        home_dir,
        config.throttle_interval_seconds
    );

    // Write plist file
    fs::write(&plist_path, plist_content)
        .map_err(|e| format!("Failed to write plist file: {}", e))?;

    println!("LaunchAgent plist created at: {}", plist_path);
    println!("To start the daemon, run: launchctl load {}", plist_path);

    Ok(())
}

pub fn uninstall_launchd_agent() -> Result<(), String> {
    use std::env;
    use std::fs;
    use std::process::Command;

    let home_dir = env::var("HOME").map_err(|_| "Could not determine home directory")?;
    let plist_path = format!("{}/Library/LaunchAgents/com.rambo.daemon.plist", home_dir);

    if std::path::Path::new(&plist_path).exists() {
        // First try to unload the service
        let output = Command::new("launchctl")
            .args(&["unload", &plist_path])
            .output()
            .map_err(|e| format!("Failed to run launchctl unload: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Warning: Failed to unload service (may not be running): {}", stderr);
        } else {
            println!("LaunchAgent service unloaded successfully");
        }

        // Remove plist file
        fs::remove_file(&plist_path)
            .map_err(|e| format!("Failed to remove plist file: {}", e))?;

        println!("LaunchAgent plist removed: {}", plist_path);
    } else {
        return Err("LaunchAgent plist not found - daemon is not installed".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::PressureLevel;

    #[test]
    fn test_daemon_creation() {
        let config = Config::default();
        let daemon = Daemon::new(config.clone());
        assert_eq!(daemon.config.rss_threshold_mb, config.rss_threshold_mb);
        let last_boost_guard = daemon.last_boost.lock().unwrap();
        assert!(last_boost_guard.is_none());
    }

    #[test]
    fn test_should_trigger_boost_normal_pressure() {
        let config = Config::default();
        let daemon = Daemon::new(config);

        // Normal pressure should not trigger boost
        assert!(!daemon.should_trigger_boost(&PressureLevel::Normal));
    }

    #[test]
    fn test_should_trigger_boost_warning_pressure() {
        let config = Config::default();
        let daemon = Daemon::new(config);

        // Warning pressure should trigger boost
        assert!(daemon.should_trigger_boost(&PressureLevel::Warning));
    }

    #[test]
    fn test_should_trigger_boost_critical_pressure() {
        let config = Config::default();
        let daemon = Daemon::new(config);

        // Critical pressure should trigger boost
        assert!(daemon.should_trigger_boost(&PressureLevel::Critical));
    }

    #[test]
    fn test_throttle_logic() {
        let mut config = Config::default();
        config.throttle_interval_seconds = 1; // Short interval for testing
        let daemon = Daemon::new(config);

        // First boost should be allowed
        assert!(daemon.should_trigger_boost(&PressureLevel::Critical));

        // Simulate a boost just happened
        {
            let mut last_boost_guard = daemon.last_boost.lock().unwrap();
            *last_boost_guard = Some(std::time::Instant::now());
        }

        // Immediate second boost should be throttled
        assert!(!daemon.should_trigger_boost(&PressureLevel::Critical));

        // Wait for throttle interval to pass
        std::thread::sleep(Duration::from_millis(1100));

        // Now boost should be allowed again
        assert!(daemon.should_trigger_boost(&PressureLevel::Critical));
    }

    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();

        assert_eq!(config.rss_threshold_mb, cloned.rss_threshold_mb);
        assert_eq!(config.log_backend, cloned.log_backend);
        assert_eq!(config.throttle_interval_seconds, cloned.throttle_interval_seconds);
    }
}