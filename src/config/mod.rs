use crate::core::error::SchatError;
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
    fn config_dir() -> PathBuf {
        #[cfg(windows)]
        {
            dirs::home_dir().expect("Could not find home directory")
        }
        #[cfg(not(windows))]
        {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        }
    }

    fn config_path() -> PathBuf {
        Self::config_dir().join(".schat").join("config.yaml")
    }

    pub fn load() -> Result<Config, SchatError> {
        let path = Self::config_path();
        let config_dir = path.parent().unwrap();

        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(contents) => match serde_yml::from_str::<Config>(&contents) {
                    Ok(mut config) => {
                        if config.providers.is_empty() {
                            config.providers = HashMap::new();
                        }
                        return Ok(config);
                    }
                    Err(e) => {
                        return Err(SchatError::Config(format!(
                            "Failed to parse config file {}: {}",
                            path.display(),
                            e
                        )));
                    }
                },
                Err(e) => {
                    return Err(SchatError::Io { source: e });
                }
            }
        }

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| SchatError::Io { source: e })?;
        }

        let config = Config {
            active_provider: None,
            auto_confirm: false,
            providers: HashMap::new(),
        };

        let _ = config.save();

        Ok(config)
    }

    pub fn save(&self) -> Result<(), SchatError> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let yaml_content = serde_yml::to_string(self)?;
        fs::write(&path, yaml_content)?;
        Ok(())
    }

    pub fn history_dir() -> PathBuf {
        Self::config_dir().join(".schat").join("history")
    }
}
