use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug)]
pub enum AgentError {
    OpenConfigFileError(std::io::Error),
    ParseConfigError(toml::de::Error),
}

pub type AgentResult<T> = std::result::Result<T, AgentError>;

pub trait Agent {
    fn prepare(&self) -> AgentResult<()>;
    fn run(&self) -> AgentResult<()>;
}

#[derive(Debug, Clone, Deserialize)]
pub enum Action {
    #[serde(rename = "prepare")]
    Prepare,
    #[serde(rename = "run")]
    Run,
    #[serde(rename = "prepare-and-run")]
    PrepareAndRun,
}

/// Generic agent configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AgentConfig {
    /// Name of the worklod, used to identify the workload.
    pub workload_name: String,
    /// Language of the workload.
    pub language: String,
    /// Action to perform.
    pub action: Action,
    /// Rest of the configuration as a string.
    #[serde(skip)]
    pub config_string: String,
}

impl AgentConfig {
    pub fn new_from_file(file_path: &PathBuf) -> AgentResult<Self> {
        let config = std::fs::read_to_string(file_path).map_err(AgentError::OpenConfigFileError)?;
        let mut config: AgentConfig = toml::from_str(&config).map_err(AgentError::ParseConfigError)?;

        let config_string = std::fs::read_to_string(file_path).map_err(AgentError::OpenConfigFileError)?;

        config.config_string = config_string;

        Ok(config)
    }
}