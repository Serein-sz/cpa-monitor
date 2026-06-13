use std::path::{Path, PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub redis: Option<RedisConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub channel: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from_dir(config_dir())
    }

    pub fn load_from_dir(config_dir: impl AsRef<Path>) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::from(config_dir.as_ref().join("default.toml")))
            .add_source(File::from(config_dir.as_ref().join("local.toml")).required(false))
            .add_source(
                Environment::with_prefix("CPA")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()?
            .try_deserialize()
    }

    pub fn db_config(&self) -> cpa_store::DbConfig {
        cpa_store::DbConfig {
            database_url: self.database.url.clone(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("CPA_CONFIG_DIR") {
        return PathBuf::from(path);
    }

    #[cfg(any(target_os = "macos", windows))]
    if let Some(path) = dirs::home_dir() {
        return path.join(".config").join("cpa-monitor");
    }

    if let Some(path) = dirs::config_dir() {
        return path.join("cpa-monitor");
    }

    PathBuf::from("config")
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::AppConfig;

    #[test]
    fn loads_default_config_file() {
        let config =
            AppConfig::load_from_dir(sample_config_dir()).expect("default config should load");

        assert!(!config.database.url.is_empty());
        assert!(config.redis.is_some());
    }

    fn sample_config_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .map(|dir| dir.join("config"))
            .find(|dir| dir.join("default.toml").exists())
            .expect("sample config directory should exist")
    }
}
