use agents::AgentOutput;
use std::fmt;

mod agents;
pub mod workload;

#[derive(Debug)]
pub enum AgentError {
    OpenConfigFileError(std::io::Error),
    ParseConfigError(toml::de::Error),
    InvalidLanguage(String),
    BuildFailed(AgentOutput),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::OpenConfigFileError(e) => write!(f, "Failed to open config file: {}", e),
            AgentError::ParseConfigError(e) => write!(f, "Failed to parse config file: {}", e),
            AgentError::BuildFailed(output) => write!(f, "Build failed: {:?}", output),
            AgentError::InvalidLanguage(e) => write!(f, "Invalid language: {}", e),
        }
    }
}

pub type AgentResult<T> = Result<T, AgentError>;

pub mod agent {
    tonic::include_proto!("cloudlet.agent");
}
