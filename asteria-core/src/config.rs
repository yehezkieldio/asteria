use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub trait LoadableConfig: Sized + Default + for<'de> Deserialize<'de> {
    fn file_name() -> &'static str;

    fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content: String = std::fs::read_to_string(&config_path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir: PathBuf = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("asteria").join(Self::file_name()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub network: NetworkConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
        }
    }
}

impl LoadableConfig for ServerConfig {
    fn file_name() -> &'static str {
        "server.toml"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub network: NetworkConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
        }
    }
}

impl LoadableConfig for ClientConfig {
    fn file_name() -> &'static str {
        "client.toml"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub host: String,
    pub port: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3100,
        }
    }
}
