use crate::core::error::SchatError;
use rmcp::{
    RoleClient, ServiceExt,
    service::RunningService,
    transport::{ConfigureCommandExt, StreamableHttpClientTransport},
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as SerdeError};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    OpenRouter,
    DeepSeek,
    Gemini,
    Anthropic,
}

impl Provider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Some(Provider::OpenAI),
            "openrouter" => Some(Provider::OpenRouter),
            "deepseek" => Some(Provider::DeepSeek),
            "gemini" => Some(Provider::Gemini),
            "anthropic" => Some(Provider::Anthropic),
            _ => None,
        }
    }

    pub fn default_base_url(&self) -> &'static str {
        match self {
            Provider::OpenAI => "https://api.openai.com/v1",
            Provider::OpenRouter => "https://openrouter.ai/api/v1",
            Provider::DeepSeek => "https://api.deepseek.com/v1",
            Provider::Gemini => "https://generativelanguage.googleapis.com",
            Provider::Anthropic => "https://api.anthropic.com/v1",
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(flatten)]
    pub transport: McpTransportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum McpTransportConfig {
    Sse {
        url: String,
    },
    StreamableHttp {
        url: String,
    },
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        envs: HashMap<String, String>,
    },
}

impl McpTransportConfig {
    pub async fn start(&self) -> Result<RunningService<RoleClient, ()>, SchatError> {
        let client = match self {
            McpTransportConfig::Sse { url } => {
                let transport =
                    rmcp::transport::sse_client::SseClientTransport::start(url.to_owned())
                        .await
                        .map_err(|e| SchatError::McpConnection(format!("SSE transport: {}", e)))?;
                ().serve(transport)
                    .await
                    .map_err(|e| SchatError::McpConnection(format!("SSE serve: {}", e)))?
            }
            McpTransportConfig::StreamableHttp { url } => {
                let transport = StreamableHttpClientTransport::from_uri(url.to_owned());
                ().serve(transport)
                    .await
                    .map_err(|e| SchatError::McpConnection(format!("HTTP serve: {}", e)))?
            }
            McpTransportConfig::Stdio {
                command,
                args,
                envs,
            } => {
                let transport = rmcp::transport::child_process::TokioChildProcess::new(
                    tokio::process::Command::new(command).configure(|cmd| {
                        cmd.args(args);
                        cmd.envs(envs);
                        cmd.stderr(Stdio::null());
                        cmd.stdout(Stdio::null());
                    }),
                )?;
                ().serve(transport)
                    .await
                    .map_err(|e| SchatError::McpConnection(format!("Stdio serve: {}", e)))?
            }
        };
        Ok(client)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub active_provider: Option<Provider>,
    pub auto_confirm: bool,
    pub providers: HashMap<Provider, ProviderConfig>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_mcp_servers")]
    pub mcp_servers: Vec<McpServerConfig>,
}

fn deserialize_mcp_servers<'de, D>(deserializer: D) -> Result<Vec<McpServerConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let values: Vec<Value> = Vec::deserialize(deserializer)?;
    values
        .into_iter()
        .map(|mut v| {
            let obj = v
                .as_object_mut()
                .ok_or_else(|| SerdeError::custom("Expected a map"))?;
            if !obj.contains_key("type") {
                if obj.contains_key("url") {
                    obj.insert(
                        "type".to_string(),
                        Value::String("streamable-http".to_string()),
                    );
                } else if obj.contains_key("command") {
                    obj.insert("type".to_string(), Value::String("stdio".to_string()));
                }
            }
            McpServerConfig::deserialize(v).map_err(SerdeError::custom)
        })
        .collect()
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
            let contents = fs::read_to_string(&path)?;
            let mut config = serde_yml::from_str::<Config>(&contents)
                .map_err(|e| SchatError::Config(format!("Parse {}: {}", path.display(), e)))?;

            if config.providers.is_empty() {
                config.providers = HashMap::new();
            }
            return Ok(config);
        }

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let config = Config {
            active_provider: None,
            auto_confirm: false,
            providers: HashMap::new(),
            mcp_servers: Vec::new(),
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

    pub async fn create_mcp_clients(
        &self,
    ) -> Result<HashMap<String, RunningService<RoleClient, ()>>, SchatError> {
        let mut clients = HashMap::new();

        for server in &self.mcp_servers {
            if server.enabled {
                let client = server.transport.start().await?;
                clients.insert(server.name.clone(), client);
            }
        }

        Ok(clients)
    }
}
