use crate::error::Result;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliConfig {
    pub edgar_email: Option<String>,
}

pub struct ConfigStorage {
    file_path: PathBuf,
}

impl ConfigStorage {
    pub fn new() -> Result<Self> {
        let file_path = Self::default_path()?;
        Ok(Self { file_path })
    }

    fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("fq");
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("config.json"))
    }

    pub fn load(&self) -> Result<CliConfig> {
        if !self.file_path.exists() {
            return Ok(CliConfig::default());
        }

        let content = fs::read_to_string(&self.file_path).context("Failed to read config file")?;
        let config = serde_json::from_str(&content).context("Failed to parse config file")?;
        Ok(config)
    }

    pub fn save(&self, config: &CliConfig) -> Result<()> {
        let content =
            serde_json::to_string_pretty(config).context("Failed to serialize config file")?;
        fs::write(&self.file_path, content).context("Failed to write config file")?;
        Ok(())
    }
}

pub fn resolve_edgar_email(cli_email: Option<String>) -> Result<String> {
    let storage = ConfigStorage::new()?;
    let mut config = storage.load()?;

    if let Some(email) = cli_email {
        let email = email.trim().to_string();
        if email.is_empty() {
            return Err(crate::error::CliError::InvalidArgument(
                "EDGAR email cannot be empty".to_string(),
            ));
        }
        config.edgar_email = Some(email.clone());
        storage.save(&config)?;
        return Ok(email);
    }

    if let Some(email) = config.edgar_email.clone() {
        return Ok(email);
    }

    let mut input = String::new();
    print!("EDGAR email not set. Enter email: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let email = input.trim().to_string();

    if email.is_empty() {
        return Err(crate::error::CliError::InvalidArgument(
            "EDGAR email required to access SEC filings".to_string(),
        ));
    }

    config.edgar_email = Some(email.clone());
    storage.save(&config)?;
    Ok(email)
}
