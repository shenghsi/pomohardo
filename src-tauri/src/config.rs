use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::io;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyLimitState {
    pub locked_date: Option<String>, // YYYY-MM-DD (Local)
    pub locked_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub work_duration_minutes: u32,
    pub break_duration_minutes: u32,
    pub long_break_duration_minutes: u32,
    pub sessions_before_long_break: u32,
    pub emergency_skips_per_day: u32,
    pub break_debt_cap_minutes: u32,
    #[serde(default = "default_emergency_hold_seconds")]
    pub emergency_hold_seconds: u32,
    #[serde(default = "default_emergency_confirm_word")]
    pub emergency_confirm_word: String,
    #[serde(default = "default_emergency_confirm_timeout_seconds")]
    pub emergency_confirm_timeout_seconds: u32,
}

fn default_emergency_hold_seconds() -> u32 {
    4
}

fn default_emergency_confirm_word() -> String {
    "SKIP".to_string()
}

fn default_emergency_confirm_timeout_seconds() -> u32 {
    15
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_duration_minutes: 25,
            break_duration_minutes: 5,
            long_break_duration_minutes: 15,
            sessions_before_long_break: 4,
            emergency_skips_per_day: 2,
            break_debt_cap_minutes: 60,
            emergency_hold_seconds: 4,
            emergency_confirm_word: "SKIP".to_string(),
            emergency_confirm_timeout_seconds: 15,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pomohardo");
        
        fs::create_dir_all(&config_dir).ok();
        config_dir.join("config.toml")
    }

    pub fn state_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pomohardo");
        fs::create_dir_all(&config_dir).ok();
        config_dir.join("state.json")
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if !path.exists() {
            let default = Self::default();
            default.save()?;
            return Ok(default);
        }

        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

pub fn load_daily_limit_state() -> DailyLimitState {
    let path = Config::state_path();
    let Ok(contents) = fs::read_to_string(&path) else {
        return DailyLimitState::default();
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

pub fn save_daily_limit_state(state: &DailyLimitState) -> Result<(), io::Error> {
    let path = Config::state_path();
    let contents = serde_json::to_string_pretty(state).unwrap_or_else(|_| "{}".to_string());
    fs::write(&path, contents)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }

    Ok(())
}

