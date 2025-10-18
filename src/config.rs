use anyhow::Context;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub index: String,
    pub packages: Vec<String>,
}

pub fn load_config() -> anyhow::Result<Config> {
    let config_str =
        std::fs::read_to_string("/etc/pkg/config.toml").context("Failed to read config file")?;
    let config: Config = toml::from_str(&config_str).context("Invalid config file")?;
    Ok(config)
}
