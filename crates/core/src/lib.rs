pub mod release;
pub mod db;
pub mod log_entry;
pub mod processes;
pub mod config;
pub mod daemon;
pub mod security;
pub mod interactive;

use serde::{Serialize, Deserialize};
use std::mem;

// Define constants for memory conversion
const BYTES_PER_MB: u64 = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemStats {
    pub total_mb: u64,
    pub free_mb: u64,
    pub active_mb: u64, // Not available in sysinfo
    pub inactive_mb: u64, // Not available in sysinfo
    pub wired_mb: u64, // Not available in sysinfo
    pub compressed_mb: u64, // Not available in sysinfo
    pub pressure: PressureLevel,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum PressureLevel { Normal, Warning, Critical }

#[cfg(not(feature = "use-sysinfo"))]
fn derive_pressure_level(stats: &MemStats) -> PressureLevel {
    if stats.total_mb == 0 { return PressureLevel::Normal; }
    let available_mb = stats.free_mb + stats.inactive_mb;
    let available_ratio = available_mb as f64 / stats.total_mb as f64;
    let compressed_ratio = stats.compressed_mb as f64 / stats.total_mb as f64;

    if available_ratio < 0.05 || compressed_ratio > 0.30 {
        PressureLevel::Critical
    } else if available_ratio < 0.15 || compressed_ratio > 0.20 {
        PressureLevel::Warning
    } else {
        PressureLevel::Normal
    }
}

#[cfg(feature = "use-sysinfo")]
fn derive_pressure_level(stats: &MemStats) -> PressureLevel {
    if stats.total_mb == 0 { return PressureLevel::Normal; }
    let free_ratio = stats.free_mb as f64 / stats.total_mb as f64;

    if free_ratio < 0.05 {
        PressureLevel::Critical
    } else if free_ratio < 0.15 {
        PressureLevel::Warning
    } else {
        PressureLevel::Normal
    }
}

#[cfg(not(feature = "use-sysinfo"))]
pub fn read_mem_stats() -> Result<MemStats, String> {
    unsafe {
        let host_port = libc::mach_host_self();
        if host_port == 0 { return Err("mach_host_self() returned 0".to_string()); }

        let mut vm_stats: libc::vm_statistics64 = mem::zeroed();
        let mut count = libc::HOST_VM_INFO64_COUNT;

        let kern_return = libc::host_statistics64(
            host_port,
            libc::HOST_VM_INFO64,
            &mut vm_stats as *mut _ as libc::host_info64_t,
            &mut count,
        );

        if kern_return != libc::KERN_SUCCESS { return Err(format!("host_statistics64() failed with code {}", kern_return)); }

        let page_size = libc::sysconf(libc::_SC_PAGESIZE) as u64;
        if page_size == 0 { return Err("sysconf(_SC_PAGESIZE) returned 0".to_string()); }

        let to_mb = |pages: u32| (pages as u64 * page_size) / BYTES_PER_MB;

        let mut total_mem: u64 = 0;
        let mut mib: [i32; 2] = [libc::CTL_HW, libc::HW_MEMSIZE];
        let mut size = mem::size_of::<u64>();
        if libc::sysctl(mib.as_mut_ptr(), 2, &mut total_mem as *mut _ as *mut libc::c_void, &mut size, std::ptr::null_mut(), 0) != 0 {
            return Err("sysctl for HW_MEMSIZE failed".to_string());
        }

        let mut stats = MemStats {
            total_mb: total_mem / BYTES_PER_MB,
            free_mb: to_mb(vm_stats.free_count),
            active_mb: to_mb(vm_stats.active_count),
            inactive_mb: to_mb(vm_stats.inactive_count),
            wired_mb: to_mb(vm_stats.wire_count),
            compressed_mb: to_mb(vm_stats.compressor_page_count),
            pressure: PressureLevel::Normal,
        };

        stats.pressure = derive_pressure_level(&stats);
        Ok(stats)
    }
}

#[cfg(feature = "use-sysinfo")]
pub fn read_mem_stats() -> Result<MemStats, String> {
    use sysinfo::{System};
    let mut sys = System::new_all();
    sys.refresh_memory();

    let mut stats = MemStats {
        total_mb: sys.total_memory() / BYTES_PER_MB,
        free_mb: sys.free_memory() / BYTES_PER_MB,
        active_mb: 0,
        inactive_mb: 0,
        wired_mb: 0,
        compressed_mb: 0,
        pressure: PressureLevel::Normal,
    };

    stats.pressure = derive_pressure_level(&stats);
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_mem_stats() {
        let stats = read_mem_stats();
        assert!(stats.is_ok());
        let stats = stats.unwrap();
        assert!(stats.total_mb > 0);
        assert!(stats.free_mb > 0);
    }

    #[test]
    fn pressure_level_logic() {
        let mut stats = MemStats {
            total_mb: 16384, // 16GB
            free_mb: 0,
            active_mb: 0,
            inactive_mb: 0,
            wired_mb: 0,
            compressed_mb: 0,
            pressure: PressureLevel::Normal,
        };

        // Normal
        stats.free_mb = 4000;
        stats.inactive_mb = 1000;
        stats.compressed_mb = 1000;
        assert_eq!(derive_pressure_level(&stats), PressureLevel::Normal);

        // Warning
        stats.free_mb = 1000;
        stats.inactive_mb = 1000;
        stats.compressed_mb = 1000;
        assert_eq!(derive_pressure_level(&stats), PressureLevel::Warning);

        // Critical
        stats.free_mb = 500;
        stats.inactive_mb = 100;
        stats.compressed_mb = 1000;
        assert_eq!(derive_pressure_level(&stats), PressureLevel::Critical);
    }
}
