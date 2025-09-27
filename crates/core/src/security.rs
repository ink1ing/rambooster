use crate::processes::ProcessInfo;

/// System processes that should never be terminated
const SYSTEM_PROCESSES: &[&str] = &[
    "kernel_task",
    "launchd",
    "WindowServer",
    "loginwindow",
    "SystemUIServer",
    "Dock",
    "Finder",
    "Activity Monitor",
    "sudo",
    "su",
    "ssh",
    "sshd",
    "systemd",
    "init",
    "kthread",
    "migration",
    "rcu_gp",
    "rcu_par_gp",
    "watchdog",
    "systemd-logind",
    "systemd-networkd",
    "systemd-resolved",
];

/// Critical process names that should never be terminated
const CRITICAL_PATTERNS: &[&str] = &[
    "kernel",
    "system",
    "System",
    "Apple",
    "Security",
    "security",
    "coreaudio",
    "CoreAudio",
    "bluetooth",
    "Bluetooth",
    "wifi",
    "WiFi",
];

#[derive(Debug, PartialEq)]
pub enum SafetyLevel {
    Safe,
    Risky,
    Dangerous,
    Forbidden,
}

#[derive(Debug)]
pub struct SafetyCheck {
    pub level: SafetyLevel,
    pub reason: String,
    pub warnings: Vec<String>,
}

pub fn check_process_safety(process: &ProcessInfo) -> SafetyCheck {
    let mut warnings = Vec::new();

    // Check if it's a system process
    if SYSTEM_PROCESSES.contains(&process.name.as_str()) {
        return SafetyCheck {
            level: SafetyLevel::Forbidden,
            reason: format!("System process '{}' must not be terminated", process.name),
            warnings,
        };
    }

    // Check for critical patterns in name
    for pattern in CRITICAL_PATTERNS {
        if process.name.to_lowercase().contains(&pattern.to_lowercase()) {
            return SafetyCheck {
                level: SafetyLevel::Dangerous,
                reason: format!("Process '{}' contains critical pattern '{}'", process.name, pattern),
                warnings,
            };
        }
    }

    // Check if process is running as root (PID 0 or very low PID numbers are suspicious)
    if process.pid == 0 {
        return SafetyCheck {
            level: SafetyLevel::Forbidden,
            reason: "Cannot terminate process with PID 0".to_string(),
            warnings,
        };
    }

    // Processes with very low PIDs are usually system processes
    if process.pid < 100 {
        warnings.push(format!("Low PID {} suggests system process", process.pid));
        return SafetyCheck {
            level: SafetyLevel::Dangerous,
            reason: format!("Low PID {} indicates potential system process", process.pid),
            warnings,
        };
    }

    // Check if it's the current process or parent processes
    let current_pid = std::process::id();
    if process.pid == current_pid {
        return SafetyCheck {
            level: SafetyLevel::Forbidden,
            reason: "Cannot terminate own process".to_string(),
            warnings,
        };
    }

    // Process is frontmost (user is actively using it)
    if process.is_frontmost {
        warnings.push("Process is currently in the foreground".to_string());
        return SafetyCheck {
            level: SafetyLevel::Risky,
            reason: "Process is currently being used by the user".to_string(),
            warnings,
        };
    }

    // High memory usage but otherwise seems safe
    if process.rss_mb > 1000 {
        warnings.push(format!("High memory usage: {} MB", process.rss_mb));
    }

    SafetyCheck {
        level: SafetyLevel::Safe,
        reason: "Process appears safe to terminate".to_string(),
        warnings,
    }
}

pub fn filter_safe_processes(
    processes: &[ProcessInfo],
    allow_risky: bool,
) -> Vec<&ProcessInfo> {
    processes
        .iter()
        .filter(|p| {
            let safety = check_process_safety(p);
            match safety.level {
                SafetyLevel::Safe => true,
                SafetyLevel::Risky => allow_risky,
                SafetyLevel::Dangerous | SafetyLevel::Forbidden => false,
            }
        })
        .collect()
}

pub fn require_confirmation(process: &ProcessInfo) -> bool {
    let safety = check_process_safety(process);

    println!("\n⚠️  Process Termination Warning ⚠️");
    println!("Process: {} (PID: {})", process.name, process.pid);
    println!("Memory: {} MB", process.rss_mb);
    println!("Safety Level: {:?}", safety.level);
    println!("Reason: {}", safety.reason);

    if !safety.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &safety.warnings {
            println!("  • {}", warning);
        }
    }

    match safety.level {
        SafetyLevel::Forbidden => {
            println!("\n🚫 This process cannot be terminated for safety reasons.");
            return false;
        }
        SafetyLevel::Dangerous => {
            println!("\n💀 DANGER: Terminating this process may cause system instability!");
            println!("Are you absolutely sure you want to continue? (type 'YES' to confirm): ");
        }
        SafetyLevel::Risky => {
            println!("\n⚠️  This process may be important to the user experience.");
            println!("Are you sure you want to terminate it? (y/N): ");
        }
        SafetyLevel::Safe => {
            println!("\nThis process appears safe to terminate.");
            println!("Continue? (y/N): ");
        }
    }

    use std::io::{self, Write};

    print!("");
    io::stdout().flush().unwrap();
    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation).unwrap();
    let confirmation = confirmation.trim();

    match safety.level {
        SafetyLevel::Dangerous => confirmation == "YES",
        SafetyLevel::Risky | SafetyLevel::Safe => {
            confirmation.eq_ignore_ascii_case("y") || confirmation.eq_ignore_ascii_case("yes")
        }
        SafetyLevel::Forbidden => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_process(name: &str, pid: u32, rss_mb: u64, is_frontmost: bool) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: name.to_string(),
            cmd: vec![],
            rss_mb,
            cpu_usage: 0.0,
            is_frontmost,
        }
    }

    #[test]
    fn test_system_process_forbidden() {
        let process = create_test_process("kernel_task", 0, 100, false);
        let safety = check_process_safety(&process);
        assert_eq!(safety.level, SafetyLevel::Forbidden);
    }

    #[test]
    fn test_critical_pattern_dangerous() {
        let process = create_test_process("SomeSystemApp", 150, 100, false);
        let safety = check_process_safety(&process);
        assert_eq!(safety.level, SafetyLevel::Dangerous);
    }

    #[test]
    fn test_frontmost_risky() {
        let process = create_test_process("Safari", 1000, 500, true);
        let safety = check_process_safety(&process);
        assert_eq!(safety.level, SafetyLevel::Risky);
    }

    #[test]
    fn test_low_pid_dangerous() {
        let process = create_test_process("some_process", 50, 100, false);
        let safety = check_process_safety(&process);
        assert_eq!(safety.level, SafetyLevel::Dangerous);
    }

    #[test]
    fn test_normal_process_safe() {
        let process = create_test_process("MyApp", 1234, 200, false);
        let safety = check_process_safety(&process);
        assert_eq!(safety.level, SafetyLevel::Safe);
    }

    #[test]
    fn test_filter_safe_processes() {
        let processes = vec![
            create_test_process("kernel_task", 0, 100, false),      // Forbidden
            create_test_process("Safari", 1000, 500, true),        // Risky (frontmost)
            create_test_process("MyApp", 1234, 200, false),        // Safe
            create_test_process("SystemServer", 123, 300, false),  // Dangerous (critical pattern)
        ];

        let safe_only = filter_safe_processes(&processes, false);
        assert_eq!(safe_only.len(), 1);
        assert_eq!(safe_only[0].name, "MyApp");

        let allow_risky = filter_safe_processes(&processes, true);
        assert_eq!(allow_risky.len(), 2);
        assert!(allow_risky.iter().any(|p| p.name == "MyApp"));
        assert!(allow_risky.iter().any(|p| p.name == "Safari"));
    }
}