use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DEFAULT_DAILY_BUDGET: u64 = 1_000_000;
const DEFAULT_BASE_URL: &str = "https://api.kimi.com/coding/v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_daily_budget")]
    pub daily_budget: u64,

    #[serde(default)]
    pub api_key: Option<String>,

    #[serde(default = "default_base_url")]
    pub base_url: String,
}

fn default_daily_budget() -> u64 {
    DEFAULT_DAILY_BUDGET
}

fn default_base_url() -> String {
    DEFAULT_BASE_URL.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daily_budget: DEFAULT_DAILY_BUDGET,
            api_key: None,
            base_url: default_base_url(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut config = config_path()
            .filter(|p| p.exists())
            .and_then(|p| std::fs::read_to_string(&p).ok())
            .and_then(|c| toml::from_str::<Config>(&c).ok())
            .unwrap_or_default();

        // Allow API key override via environment variable.
        if let Ok(env_key) = std::env::var("KIMI_CODE_API_KEY")
            && !env_key.is_empty()
        {
            config.api_key = Some(env_key);
        }

        config
    }

    pub fn percentage(&self, today_tokens: u64) -> u8 {
        if self.daily_budget == 0 {
            return 0;
        }
        let pct = (today_tokens as f64 / self.daily_budget as f64) * 100.0;
        pct.min(255.0) as u8
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path().ok_or("could not determine config directory")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }

    pub fn set_api_key(&mut self, key: String) -> Result<(), Box<dyn std::error::Error>> {
        self.api_key = Some(key);
        self.save()
    }
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("kimi-usage-widget").join("config.toml"))
}

pub fn ensure_default_config() {
    if let Some(path) = config_path().filter(|p| !p.exists()) {
        let _ = std::fs::create_dir_all(path.parent().unwrap_or(Path::new("")));
        let default = Config::default();
        let _ = std::fs::write(&path, toml::to_string_pretty(&default).unwrap_or_default());
    }
}
