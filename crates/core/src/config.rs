use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{env, fs};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub rss_threshold_mb: u64,
    pub log_backend: String,
    pub log_retention_days: u32,
    pub enable_process_termination: bool,
    pub throttle_interval_seconds: u64,
    pub whitelist_processes: Vec<String>,
    pub blacklist_processes: Vec<String>,
    pub hotkey: HotkeyConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HotkeyConfig {
    pub enabled: bool,
    pub key_combination: String,
    pub show_notification: bool,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            key_combination: "Control+R".to_string(),
            show_notification: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rss_threshold_mb: 50,
            log_backend: "jsonl".to_string(),
            log_retention_days: 30,
            enable_process_termination: false,
            throttle_interval_seconds: 300, // 5 minutes
            whitelist_processes: vec![
                "kernel_task".to_string(),
                "launchd".to_string(),
                "WindowServer".to_string(),
            ],
            blacklist_processes: vec![],
            hotkey: HotkeyConfig::default(),
        }
    }
}

fn get_config_dir() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let app_config_dir = config_dir.join("rambo");
    std::fs::create_dir_all(&app_config_dir).map_err(|e| format!("Could not create config directory: {}", e))?;
    Ok(app_config_dir)
}

pub fn get_config_path() -> Result<PathBuf, String> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("config.toml"))
}

pub fn load_config() -> Result<Config, String> {
    let mut config = Config::default();

    // 1. Load from config file
    let config_path = get_config_path()?;
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let file_config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config = file_config;
    }

    // 2. Override with environment variables
    if let Ok(val) = env::var("RAMBO_RSS_THRESHOLD_MB") {
        config.rss_threshold_mb = val.parse()
            .map_err(|_| "Invalid RAMBO_RSS_THRESHOLD_MB value")?;
    }

    if let Ok(val) = env::var("RAMBO_LOG_BACKEND") {
        config.log_backend = val;
    }

    if let Ok(val) = env::var("RAMBO_LOG_RETENTION_DAYS") {
        config.log_retention_days = val.parse()
            .map_err(|_| "Invalid RAMBO_LOG_RETENTION_DAYS value")?;
    }

    if let Ok(val) = env::var("RAMBO_ENABLE_PROCESS_TERMINATION") {
        config.enable_process_termination = val.parse()
            .map_err(|_| "Invalid RAMBO_ENABLE_PROCESS_TERMINATION value")?;
    }

    if let Ok(val) = env::var("RAMBO_THROTTLE_INTERVAL_SECONDS") {
        config.throttle_interval_seconds = val.parse()
            .map_err(|_| "Invalid RAMBO_THROTTLE_INTERVAL_SECONDS value")?;
    }

    if let Ok(val) = env::var("RAMBO_WHITELIST_PROCESSES") {
        config.whitelist_processes = val.split(',').map(|s| s.trim().to_string()).collect();
    }

    if let Ok(val) = env::var("RAMBO_BLACKLIST_PROCESSES") {
        config.blacklist_processes = val.split(',').map(|s| s.trim().to_string()).collect();
    }

    if let Ok(val) = env::var("RAMBO_HOTKEY_ENABLED") {
        config.hotkey.enabled = val.parse()
            .map_err(|_| "Invalid RAMBO_HOTKEY_ENABLED value")?;
    }

    if let Ok(val) = env::var("RAMBO_HOTKEY_COMBINATION") {
        config.hotkey.key_combination = val;
    }

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.rss_threshold_mb, 50);
        assert_eq!(config.log_backend, "jsonl");
        assert_eq!(config.log_retention_days, 30);
        assert!(!config.enable_process_termination);
        assert_eq!(config.throttle_interval_seconds, 300);
        assert!(config.whitelist_processes.contains(&"kernel_task".to_string()));
        assert!(config.whitelist_processes.contains(&"launchd".to_string()));
        assert!(config.whitelist_processes.contains(&"WindowServer".to_string()));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_content = toml::to_string(&config);
        assert!(toml_content.is_ok());

        let deserialized: Result<Config, _> = toml::from_str(&toml_content.unwrap());
        assert!(deserialized.is_ok());
        let deserialized_config = deserialized.unwrap();

        assert_eq!(config.rss_threshold_mb, deserialized_config.rss_threshold_mb);
        assert_eq!(config.log_backend, deserialized_config.log_backend);
    }

    #[test]
    fn test_env_variable_override() {
        // Save original values to restore later
        let original_threshold = env::var("RAMBO_RSS_THRESHOLD_MB").ok();
        let original_backend = env::var("RAMBO_LOG_BACKEND").ok();
        let original_termination = env::var("RAMBO_ENABLE_PROCESS_TERMINATION").ok();

        env::set_var("RAMBO_RSS_THRESHOLD_MB", "100");
        env::set_var("RAMBO_LOG_BACKEND", "sqlite");
        env::set_var("RAMBO_ENABLE_PROCESS_TERMINATION", "true");

        let config_result = load_config();

        // Clean up first
        if let Some(val) = original_threshold {
            env::set_var("RAMBO_RSS_THRESHOLD_MB", val);
        } else {
            env::remove_var("RAMBO_RSS_THRESHOLD_MB");
        }

        if let Some(val) = original_backend {
            env::set_var("RAMBO_LOG_BACKEND", val);
        } else {
            env::remove_var("RAMBO_LOG_BACKEND");
        }

        if let Some(val) = original_termination {
            env::set_var("RAMBO_ENABLE_PROCESS_TERMINATION", val);
        } else {
            env::remove_var("RAMBO_ENABLE_PROCESS_TERMINATION");
        }

        // Now check results
        if let Err(e) = &config_result {
            println!("Config load error: {}", e);
        }
        assert!(config_result.is_ok());
        let config = config_result.unwrap();

        assert_eq!(config.rss_threshold_mb, 100);
        assert_eq!(config.log_backend, "sqlite");
        assert!(config.enable_process_termination);
    }

    #[test]
    fn test_invalid_env_variables() {
        let original_threshold = env::var("RAMBO_RSS_THRESHOLD_MB").ok();

        env::set_var("RAMBO_RSS_THRESHOLD_MB", "invalid");

        let result = load_config();

        // Clean up
        if let Some(val) = original_threshold {
            env::set_var("RAMBO_RSS_THRESHOLD_MB", val);
        } else {
            env::remove_var("RAMBO_RSS_THRESHOLD_MB");
        }

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid RAMBO_RSS_THRESHOLD_MB"));
    }

    #[test]
    fn test_whitelist_blacklist_parsing() {
        // Save original values
        let original_whitelist = env::var("RAMBO_WHITELIST_PROCESSES").ok();
        let original_blacklist = env::var("RAMBO_BLACKLIST_PROCESSES").ok();

        env::set_var("RAMBO_WHITELIST_PROCESSES", "process1,process2, process3");
        env::set_var("RAMBO_BLACKLIST_PROCESSES", "bad1, bad2,bad3 ");

        let config_result = load_config();

        // Clean up first
        if let Some(val) = original_whitelist {
            env::set_var("RAMBO_WHITELIST_PROCESSES", val);
        } else {
            env::remove_var("RAMBO_WHITELIST_PROCESSES");
        }

        if let Some(val) = original_blacklist {
            env::set_var("RAMBO_BLACKLIST_PROCESSES", val);
        } else {
            env::remove_var("RAMBO_BLACKLIST_PROCESSES");
        }

        if let Err(e) = &config_result {
            println!("Config load error: {}", e);
        }
        assert!(config_result.is_ok());
        let config = config_result.unwrap();

        assert_eq!(config.whitelist_processes, vec!["process1", "process2", "process3"]);
        assert_eq!(config.blacklist_processes, vec!["bad1", "bad2", "bad3"]);
    }
}
