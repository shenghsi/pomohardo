use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub work_duration_minutes: u32,
    pub break_duration_minutes: u32,
    pub long_break_duration_minutes: u32,
    pub sessions_before_long_break: u32,
    pub emergency_skips_per_day: u32,
    pub break_debt_cap_minutes: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_duration_minutes: 25,
            break_duration_minutes: 5,
            long_break_duration_minutes: 15,
            sessions_before_long_break: 4,
            emergency_skips_per_day: 3,
            break_debt_cap_minutes: 60,
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

