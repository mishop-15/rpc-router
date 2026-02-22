use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Provider {
    pub name: String,
     pub url: String,
     pub weight: u64,
}

#[derive(Debug, Deserialize)]
pub struct Config {
     pub settings: Settings,
    pub providers: Vec<Provider>,
}

pub fn load_config() -> anyhow::Result<Config> {
    let content = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}