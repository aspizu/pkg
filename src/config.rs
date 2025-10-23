use std::fs;

use eyre::Context;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub index: String,
    pub packages: Vec<String>,
}

pub fn load_config(root: &str) -> eyre::Result<Config> {
    let config_str = fs::read_to_string(&format!("{}/etc/meow/meow.toml", root))
        .context("Failed to read config file")?;
    let config: Config = toml::from_str(&config_str).context("Invalid config file")?;
    Ok(config)
}
