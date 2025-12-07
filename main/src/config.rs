use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub gemini_api_key: String,
    pub resend_api_key: String,
    pub from_email: String,
    pub to_email: String,
    pub gemini_model: Option<String>,
    pub feeds: Vec<FeedConfig>,
}

#[derive(Deserialize)]
pub struct FeedConfig {
    pub name: String,
    pub url: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn feeds_as_tuples(&self) -> Vec<(&str, &str)> {
        self.feeds
            .iter()
            .map(|f| (f.url.as_str(), f.name.as_str()))
            .collect()
    }
}
