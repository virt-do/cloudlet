use crate::{
    agent::{execute_request, ExecuteRequest},
    agents::Language,
    AgentError, AgentResult,
};
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
    /// Code
    pub code: String,
    /// Rest of the configuration as a string.
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

    pub fn new_from_execute_request(execute_request: ExecuteRequest) -> Result<Self, AgentError> {
        Ok(Self {
            workload_name: execute_request.workload_name.clone(),
            // TODO: Fix this unwrap
            language: Language::try_from(execute_request.language.clone().as_str())?,
            action: execute_request.action().into(),
            config_string: execute_request.config_str,
            code: execute_request.code,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    Prepare,
    Run,
    PrepareAndRun,
}

impl From<execute_request::Action> for Action {
    fn from(value: execute_request::Action) -> Self {
        match value {
            execute_request::Action::Prepare => Action::Prepare,
            execute_request::Action::Run => Action::Run,
            execute_request::Action::PrepareAndRun => Action::PrepareAndRun,
        }
    }
}
