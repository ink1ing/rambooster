use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::{MemStats, PressureLevel};
use chrono::prelude::*;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::fs::{create_dir_all, File, OpenOptions};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEvent {
    pub ts: String, // ISO 8601 format
    pub action: String,
    pub before: Option<MemStats>,
    pub after: Option<MemStats>,
    pub delta_mb: i64,
    pub pressure: PressureLevel,
    pub details: Value, // Flexible JSON object for action-specific data
}

fn get_log_file_path_for_date(date: &str) -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir().ok_or("Could not find data directory")?;
    let log_dir = data_dir.join("rambo").join("logs");
    create_dir_all(&log_dir).map_err(|e| format!("Could not create log directory: {}", e))?;

    Ok(log_dir.join(format!("{}.jsonl", date)))
}

fn get_log_file_path() -> Result<PathBuf, String> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    get_log_file_path_for_date(&today)
}

pub fn read_log_events(date: &str) -> Result<Vec<LogEvent>, String> {
    let file_path = get_log_file_path_for_date(date)?;
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(file_path).map_err(|e| format!("Could not open log file: {}", e))?;
    let reader = io::BufReader::new(file);

    let mut events = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Could not read line from log file: {}", e))?;
        if line.trim().is_empty() { continue; }
        let event: LogEvent = serde_json::from_str(&line).map_err(|e| format!("Could not parse log event: {}\nLine: {}", e, line))?;
        events.push(event);
    }

    Ok(events)
}

pub fn write_log_event(event: &LogEvent) -> Result<(), String> {
    let file_path = get_log_file_path()?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .map_err(|e| format!("Could not open log file: {}", e))?;

    let json = serde_json::to_string(event).map_err(|e| format!("Could not serialize log event: {}", e))?;

    writeln!(file, "{}", json).map_err(|e| format!("Could not write to log file: {}", e))
}

/// 获取日志目录路径
pub fn get_log_directory() -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir().ok_or("Could not find data directory")?;
    Ok(data_dir.join("rambo").join("logs"))
}

/// 清理过期日志文件
pub fn cleanup_old_logs(retention_days: u32) -> Result<u32, String> {
    use std::fs;

    let log_dir = get_log_directory()?;
    if !log_dir.exists() {
        return Ok(0);
    }

    let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);
    let mut deleted_count = 0;

    let entries = fs::read_dir(&log_dir)
        .map_err(|e| format!("Could not read log directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read directory entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                // 尝试解析文件名为日期 (YYYY-MM-DD)
                if let Ok(file_date) = NaiveDate::parse_from_str(file_stem, "%Y-%m-%d") {
                    let file_datetime = file_date.and_hms_opt(0, 0, 0).unwrap().and_utc();

                    if file_datetime < cutoff_date {
                        fs::remove_file(&path)
                            .map_err(|e| format!("Could not delete log file {:?}: {}", path, e))?;
                        deleted_count += 1;
                    }
                }
            }
        }
    }

    Ok(deleted_count)
}

/// 清理所有日志文件
pub fn clear_all_logs() -> Result<u32, String> {
    use std::fs;

    let log_dir = get_log_directory()?;
    if !log_dir.exists() {
        return Ok(0);
    }

    let mut deleted_count = 0;
    let entries = fs::read_dir(&log_dir)
        .map_err(|e| format!("Could not read log directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read directory entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            fs::remove_file(&path)
                .map_err(|e| format!("Could not delete log file {:?}: {}", path, e))?;
            deleted_count += 1;
        }
    }

    Ok(deleted_count)
}

/// 获取日志目录大小（字节）
pub fn get_logs_size() -> Result<u64, String> {
    use std::fs;

    let log_dir = get_log_directory()?;
    if !log_dir.exists() {
        return Ok(0);
    }

    let mut total_size = 0;
    let entries = fs::read_dir(&log_dir)
        .map_err(|e| format!("Could not read log directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read directory entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            let metadata = fs::metadata(&path)
                .map_err(|e| format!("Could not get metadata for {:?}: {}", path, e))?;
            total_size += metadata.len();
        }
    }

    Ok(total_size)
}

/// 列出所有日志文件
pub fn list_log_files() -> Result<Vec<(String, u64)>, String> {
    use std::fs;

    let log_dir = get_log_directory()?;
    if !log_dir.exists() {
        return Ok(Vec::new());
    }

    let mut log_files = Vec::new();
    let entries = fs::read_dir(&log_dir)
        .map_err(|e| format!("Could not read log directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read directory entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                let metadata = fs::metadata(&path)
                    .map_err(|e| format!("Could not get metadata for {:?}: {}", path, e))?;
                log_files.push((file_stem.to_string(), metadata.len()));
            }
        }
    }

    // 按日期排序
    log_files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(log_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn can_write_log_event() {
        let event = LogEvent {
            ts: Utc::now().to_rfc3339(),
            action: "test".to_string(),
            before: None,
            after: None,
            delta_mb: 0,
            pressure: PressureLevel::Normal,
            details: serde_json::json!({ "test": "data" }),
        };

        let result = write_log_event(&event);
        assert!(result.is_ok());

        // Verify file content
        let log_file = get_log_file_path().unwrap();
        let content = fs::read_to_string(log_file).unwrap();
        assert!(content.contains("\"action\":\"test\""));
    }
}