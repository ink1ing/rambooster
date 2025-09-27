use crate::processes::ProcessInfo;
use std::collections::HashSet;
use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant};
use std::io::ErrorKind;
use sysinfo::{System, Signal, Pid, ProcessRefreshKind, ProcessesToUpdate};
use crate::{MemStats, read_mem_stats};
use serde::Serialize;


#[derive(Debug)]
pub enum BoostError {
    Purge(PurgeError),
    Stats(String),
}

#[derive(Debug, Serialize, Clone)]
pub struct BoostResult {
    pub before: MemStats,
    pub after: MemStats,
    pub delta_mb: i64,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
}

#[derive(Debug)]
pub enum PurgeError {
    CommandNotFound,
    ExecutionFailed(ExitStatus),
    IoError(std::io::Error),
}

impl From<std::io::Error> for PurgeError {
    fn from(err: std::io::Error) -> PurgeError {
        if err.kind() == ErrorKind::NotFound {
            PurgeError::CommandNotFound
        } else {
            PurgeError::IoError(err)
        }
    }
}

pub fn purge() -> Result<(Duration, ExitStatus), PurgeError> {
    let start = Instant::now();

    // 首先尝试直接执行purge（某些系统配置可能允许）
    let output = Command::new("/usr/sbin/purge").output();

    let final_output = match output {
        Ok(out) if out.status.success() => out,
        _ => {
            // 如果直接执行失败，使用sudo但不需要交互式密码
            Command::new("sudo")
                .arg("-n") // 非交互模式，如果需要密码会失败
                .arg("/usr/sbin/purge")
                .output()?
        }
    };

    let duration = start.elapsed();

    if final_output.status.success() {
        Ok((duration, final_output.status))
    } else {
        Err(PurgeError::ExecutionFailed(final_output.status))
    }
}

pub fn boost() -> Result<BoostResult, BoostError> {
    let before_stats = read_mem_stats().map_err(BoostError::Stats)?;

    let (duration, _) = purge().map_err(BoostError::Purge)?;

    let after_stats = read_mem_stats().map_err(BoostError::Stats)?;

    let delta = after_stats.free_mb as i64 - before_stats.free_mb as i64;

    Ok(BoostResult {
        before: before_stats,
        after: after_stats,
        delta_mb: delta,
        duration,
    })
}

pub fn get_candidate_processes<'a>(
    processes: &'a [ProcessInfo],
    rss_threshold_mb: u64,
    whitelist: &HashSet<String>,
    blacklist: &HashSet<String>,
) -> Vec<&'a ProcessInfo> {
    processes
        .iter()
        .filter(|p| {
            if p.rss_mb < rss_threshold_mb { return false; }
            if p.is_frontmost { return false; }
            if blacklist.contains(&p.name) { return false; }
            if !whitelist.is_empty() && !whitelist.contains(&p.name) { return false; }
            true
        })
        .collect()
}
pub fn terminate(pid: u32, force: bool) -> bool {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let sysinfo_pid = Pid::from_u32(pid);

    if let Some(process) = sys.process(sysinfo_pid) {
        // 尝试优雅终止
        if process.kill_with(Signal::Term).unwrap_or(false) {
            std::thread::sleep(Duration::from_secs(2));
            sys.refresh_processes(ProcessesToUpdate::All, true);

            // 检查进程是否已终止
            if sys.process(sysinfo_pid).is_none() {
                return true;
            }

            // 如果需要强制终止
            if force {
                if let Some(process) = sys.process(sysinfo_pid) {
                    return process.kill_with(Signal::Kill).unwrap_or(false);
                }
            }
        }
    }
    false
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{self, Command};

    #[test]
    #[ignore] // This test can be slow and requires /usr/bin/purge to exist.
    fn can_purge() {
        let result = purge();
        match result {
            Ok((duration, status)) => {
                assert!(duration.as_millis() > 0);
                assert!(status.success());
            }
            Err(PurgeError::CommandNotFound) => {
                eprintln!("Skipping purge test: /usr/bin/purge not found.");
            }
            Err(e) => panic!("purge() failed unexpectedly: {:?}", e),
        }
    }

    #[test]
    fn can_filter_candidates() {
        let p1 = ProcessInfo { pid: 1, name: "good_process".to_string(), rss_mb: 600, is_frontmost: false, cmd: vec![], cpu_usage: 0.0 };
        let p2 = ProcessInfo { pid: 2, name: "too_small".to_string(), rss_mb: 400, is_frontmost: false, cmd: vec![], cpu_usage: 0.0 };
        let p3 = ProcessInfo { pid: 3, name: "frontmost".to_string(), rss_mb: 700, is_frontmost: true, cmd: vec![], cpu_usage: 0.0 };
        let p4 = ProcessInfo { pid: 4, name: "blacklisted".to_string(), rss_mb: 800, is_frontmost: false, cmd: vec![], cpu_usage: 0.0 };
        let p5 = ProcessInfo { pid: 5, name: "whitelisted".to_string(), rss_mb: 900, is_frontmost: false, cmd: vec![], cpu_usage: 0.0 };

        let processes = vec![p1.clone(), p2.clone(), p3.clone(), p4.clone(), p5.clone()];

        let mut blacklist = HashSet::new();
        blacklist.insert("blacklisted".to_string());

        let candidates = get_candidate_processes(&processes, 500, &HashSet::new(), &blacklist);
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().any(|p| p.pid == 1));
        assert!(candidates.iter().any(|p| p.pid == 5));

        let mut whitelist = HashSet::new();
        whitelist.insert("whitelisted".to_string());
        let candidates = get_candidate_processes(&processes, 500, &whitelist, &blacklist);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].pid, 5);
    }

    #[test]
    #[ignore] // This test is flaky and affects other processes.
    fn can_terminate() {
        let child = Command::new("sleep").arg("10").spawn().unwrap();
        let pid = child.id();

        let terminated = terminate(pid, true);
        assert!(terminated);

        let output = Command::new("ps").arg("-p").arg(pid.to_string()).output().unwrap();
        assert!(!String::from_utf8_lossy(&output.stdout).contains(&pid.to_string()));
    }

    #[test]
    #[ignore] // Slow, depends on `purge` command.
    fn can_boost() {
        let result = boost();
        match result {
            Ok(res) => {
                assert!(res.duration.as_millis() > 0);
                println!("Freed up {} MB", res.delta_mb);
            }
            Err(e) => match e {
                BoostError::Purge(PurgeError::CommandNotFound) => {
                    eprintln!("Skipping boost test: /usr/bin/purge not found.");
                }
                _ => panic!("boost() failed unexpectedly: {:?}", e),
            },
        }
    }
}
