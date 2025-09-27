use sysinfo::System;
use objc2_app_kit::NSWorkspace;
use serde::Serialize;

const BYTES_PER_MB: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: Vec<String>,
    pub rss_mb: u64,
    pub cpu_usage: f32,
    pub is_frontmost: bool,
}

fn get_frontmost_pid() -> Option<u32> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let front_app = workspace.frontmostApplication()?;
        Some(front_app.processIdentifier() as u32)
    }
}

pub fn get_all_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let frontmost_pid = get_frontmost_pid();

    sys.processes().values().map(|proc| {
        let pid = proc.pid().as_u32();
        ProcessInfo {
            pid,
            name: proc.name().to_string_lossy().into_owned(),
            cmd: proc.cmd().iter().map(|s| s.to_string_lossy().into_owned()).collect(),
            rss_mb: proc.memory() / BYTES_PER_MB,
            cpu_usage: proc.cpu_usage(),
            is_frontmost: frontmost_pid.map_or(false, |p| p == pid),
        }
    }).collect()
}

pub fn sort_and_take_processes(mut processes: Vec<ProcessInfo>, n: usize) -> Vec<ProcessInfo> {
    processes.sort_by(|a, b| b.rss_mb.cmp(&a.rss_mb));
    processes.into_iter().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn can_get_all_processes() {
        let processes = get_all_processes();
        assert!(!processes.is_empty());

        let current_pid = process::id();
        let current_process = processes.iter().find(|p| p.pid == current_pid);
        assert!(current_process.is_some());

        let info = current_process.unwrap();
        assert_eq!(info.pid, current_pid);
        assert!(info.rss_mb > 0);
        assert!(!info.name.is_empty());
    }

    #[test]
    fn can_get_frontmost_pid() {
        let pid = get_frontmost_pid();
        if let Some(pid) = pid {
            assert!(pid > 0);
        }
    }

    #[test]
    fn can_sort_and_take() {
        let p1 = ProcessInfo { pid: 1, name: "p1".to_string(), cmd: vec![], rss_mb: 100, cpu_usage: 0.0, is_frontmost: false };
        let p2 = ProcessInfo { pid: 2, name: "p2".to_string(), cmd: vec![], rss_mb: 300, cpu_usage: 0.0, is_frontmost: false };
        let p3 = ProcessInfo { pid: 3, name: "p3".to_string(), cmd: vec![], rss_mb: 200, cpu_usage: 0.0, is_frontmost: false };
        let processes = vec![p1.clone(), p2.clone(), p3.clone()];

        let sorted = sort_and_take_processes(processes, 2);
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0], p2);
        assert_eq!(sorted[1], p3);
    }
}