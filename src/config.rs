use std::fs;

use eyre::Context;
use minisign_verify::PublicKey;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub index: String,
    pub keys: Vec<String>,
    pub packages: Vec<String>,
}

pub fn load_config(root: &str) -> eyre::Result<Config> {
    let config_str = fs::read_to_string(&format!("{}/etc/meow/meow.toml", root))
        .context("Failed to read config file")?;
    let config: Config = toml::from_str(&config_str).context("Invalid config file")?;
    Ok(config)
}

pub fn load_keys(config: &Config) -> eyre::Result<Vec<PublicKey>> {
    let mut keys = vec![];
    for key in &config.keys {
        let key = PublicKey::from_base64(key).context("Unable to load keys from config.")?;
        keys.push(key);
    }
    Ok(keys)
}
