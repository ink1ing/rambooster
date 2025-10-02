use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current: String,
    pub latest: Option<String>,
    pub update_available: bool,
}

#[derive(Debug)]
pub enum UpdateError {
    NetworkError(String),
    InstallationError(String),
    PermissionError(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            UpdateError::InstallationError(msg) => write!(f, "安装错误: {}", msg),
            UpdateError::PermissionError(msg) => write!(f, "权限错误: {}", msg),
            UpdateError::IoError(err) => write!(f, "IO错误: {}", err),
        }
    }
}

impl std::error::Error for UpdateError {}

impl From<std::io::Error> for UpdateError {
    fn from(err: std::io::Error) -> Self {
        UpdateError::IoError(err)
    }
}

/// 获取当前版本信息
pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 从GitHub API检查最新版本
pub fn check_latest_version() -> Result<String, UpdateError> {
    let output = Command::new("curl")
        .args(&[
            "-s",
            "-H", "Accept: application/vnd.github.v3+json",
            "https://api.github.com/repos/ink1ing/rambooster/releases/latest"
        ])
        .output()
        .map_err(|_| UpdateError::NetworkError("无法执行curl命令".to_string()))?;

    if !output.status.success() {
        return Err(UpdateError::NetworkError("获取版本信息失败".to_string()));
    }

    let response = String::from_utf8_lossy(&output.stdout);

    // 简单的JSON解析获取tag_name
    if let Some(start) = response.find("\"tag_name\":\"") {
        let start = start + 12; // "tag_name":"的长度
        if let Some(end) = response[start..].find('\"') {
            let version = &response[start..start + end];
            // 移除v前缀如果存在
            let clean_version = version.strip_prefix('v').unwrap_or(version);
            return Ok(clean_version.to_string());
        }
    }

    Err(UpdateError::NetworkError("解析版本信息失败".to_string()))
}

/// 比较版本号
pub fn compare_versions(v1: &str, v2: &str) -> std::cmp::Ordering {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect()
    };

    let version1 = parse_version(v1);
    let version2 = parse_version(v2);

    for i in 0..std::cmp::max(version1.len(), version2.len()) {
        let v1_part = version1.get(i).unwrap_or(&0);
        let v2_part = version2.get(i).unwrap_or(&0);

        match v1_part.cmp(v2_part) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    std::cmp::Ordering::Equal
}

/// 检查是否有更新
pub fn check_for_updates() -> Result<VersionInfo, UpdateError> {
    let current = get_current_version();
    let latest = match check_latest_version() {
        Ok(version) => Some(version.clone()),
        Err(_) => None,
    };

    let update_available = if let Some(ref latest_ver) = latest {
        compare_versions(&current, latest_ver) == std::cmp::Ordering::Less
    } else {
        false
    };

    Ok(VersionInfo {
        current,
        latest,
        update_available,
    })
}

/// 检测并清理旧版本
pub fn cleanup_old_versions() -> Result<Vec<String>, UpdateError> {
    let mut cleaned_files = Vec::new();

    // 检查可能的旧版本安装位置
    let mut possible_locations = vec![
        "/usr/local/bin/rb".to_string(),
        "/usr/local/bin/rambo".to_string(),
        "/usr/local/bin/rambooster".to_string(),
    ];

    if let Ok(home) = std::env::var("HOME") {
        possible_locations.push(format!("{}/.local/bin/rb.backup.*", home));
    }

    for location in &possible_locations {
        if location.contains('*') {
            // 处理通配符路径（备份文件）
            if let Ok(home) = std::env::var("HOME") {
                let backup_dir = format!("{}/.local/bin", home);
                if let Ok(entries) = fs::read_dir(&backup_dir) {
                    for entry in entries.flatten() {
                        let file_name = entry.file_name();
                        let file_name_str = file_name.to_string_lossy();
                        if file_name_str.starts_with("rb.backup.") {
                            let full_path = entry.path();
                            if let Ok(_) = fs::remove_file(&full_path) {
                                cleaned_files.push(full_path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        } else if Path::new(location).exists() {
            // 检查是否是旧版本
            if let Ok(output) = Command::new(location).arg("--version").output() {
                let version_output = String::from_utf8_lossy(&output.stdout);
                if version_output.contains("rambo") || version_output.contains("RAM Booster") {
                    // 这可能是旧版本，但要小心不要删除当前版本
                    let current_exe = std::env::current_exe().ok();
                    if let Some(current_path) = current_exe {
                        if current_path.to_string_lossy() != *location {
                            if let Ok(_) = fs::remove_file(location) {
                                cleaned_files.push(location.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(cleaned_files)
}

/// 执行更新
pub fn perform_update(force: bool) -> Result<(), UpdateError> {
    // 检查更新脚本是否存在
    let mut update_script_paths = vec![
        "update.sh".to_string(),
        "./update.sh".to_string(),
    ];

    if let Ok(home) = std::env::var("HOME") {
        update_script_paths.push(format!("{}/.local/bin/rb-update", home));
    }

    let mut update_script: Option<String> = None;
    for path in &update_script_paths {
        if Path::new(path.as_str()).exists() {
            update_script = Some(path.clone());
            break;
        }
    }

    let script_path = update_script.ok_or_else(|| {
        UpdateError::InstallationError("找不到更新脚本".to_string())
    })?;

    println!("🔄 开始更新 RAM Booster...");

    // 先清理旧版本
    match cleanup_old_versions() {
        Ok(cleaned) => {
            if !cleaned.is_empty() {
                println!("🧹 清理了旧版本文件:");
                for file in &cleaned {
                    println!("   - {}", file);
                }
            }
        }
        Err(e) => {
            println!("⚠️  清理旧版本时出现警告: {}", e);
        }
    }

    // 执行更新脚本
    let mut cmd = Command::new("bash");
    cmd.arg(&script_path);

    if force {
        cmd.env("FORCE_UPDATE", "1");
    }

    let status = cmd.status()?;

    if status.success() {
        println!("✅ 更新完成！");
        Ok(())
    } else {
        Err(UpdateError::InstallationError("更新脚本执行失败".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert_eq!(compare_versions("1.0.0", "1.0.0"), std::cmp::Ordering::Equal);
        assert_eq!(compare_versions("1.0.0", "1.0.1"), std::cmp::Ordering::Less);
        assert_eq!(compare_versions("1.0.1", "1.0.0"), std::cmp::Ordering::Greater);
        assert_eq!(compare_versions("1.2.0", "1.10.0"), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_get_current_version() {
        let version = get_current_version();
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }
}