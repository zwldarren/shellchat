use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    OpenRouter,
}

impl Provider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Some(Provider::OpenAI),
            "openrouter" => Some(Provider::OpenRouter),
            _ => None,
        }
    }

    pub fn default_base_url(&self) -> &'static str {
        match self {
            Provider::OpenAI => "https://api.openai.com/v1",
            Provider::OpenRouter => "https://openrouter.ai/api/v1",
        }
    }
}

impl Default for Provider {
    fn default() -> Self {
        Provider::OpenAI
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub active_provider: Option<Provider>,
    pub auto_confirm: bool,
    pub providers: HashMap<Provider, ProviderConfig>,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        #[cfg(windows)]
        {
            dirs::home_dir().expect("Could not find home directory")
        }
        #[cfg(not(windows))]
        {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        }
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join(".schat").join("config.yaml")
    }

    pub fn load() -> Config {
        let config_path = Self::config_path();
        let config_dir = Self::config_dir().join(".schat");

        // If config file exists and can be parsed, return it
        if config_path.exists() {
            if let Ok(contents) = fs::read_to_string(&config_path) {
                match serde_yml::from_str::<Config>(&contents) {
                    Ok(mut config) => {
                        // Ensure providers HashMap exists
                        if config.providers.is_empty() {
                            config.providers = HashMap::new();
                        }
                        return config;
                    }
                    Err(e) => {
                        eprintln!("Error parsing config file: {}", e);
                        // Fall through to create new config
                    }
                }
            }
        }

        // Ensure config directory exists
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }

        // Create new config with defaults
        let config = Config {
            active_provider: None,
            auto_confirm: false,
            providers: HashMap::new(),
        };
        let _ = config.save(); // Ignore save errors
        config
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = Self::config_dir();
        let config_path = Self::config_path();

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        // Serialize config to YAML
        let yaml_content = serde_yml::to_string(self)?;

        // Write to file
        fs::write(&config_path, yaml_content)?;

        Ok(())
    }
}
