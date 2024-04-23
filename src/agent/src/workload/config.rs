use crate::{agents::Language, AgentError, AgentResult};
use serde::Deserialize;
use std::path::PathBuf;

/// Generic agent configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Name of the worklod, used to identify the workload.
    pub workload_name: String,
    /// Language of the workload.
    pub language: Language,
    /// Action to perform.
    pub action: Action,
    /// Rest of the configuration as a string.
    #[serde(skip)]
    pub config_string: String,
}

impl Config {
    pub fn from_file(file_path: &PathBuf) -> AgentResult<Self> {
        let config = std::fs::read_to_string(file_path).map_err(AgentError::OpenConfigFileError)?;
        let mut config: Config = toml::from_str(&config).map_err(AgentError::ParseConfigError)?;

        let config_string =
            std::fs::read_to_string(file_path).map_err(AgentError::OpenConfigFileError)?;

        config.config_string = config_string;

        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    Prepare,
    Run,
    PrepareAndRun,
}
