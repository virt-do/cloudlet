use std::fmt;

mod agents;
pub mod workload;

#[derive(Debug)]
pub enum AgentError {
    OpenConfigFileError(std::io::Error),
    ParseConfigError(toml::de::Error),
    InvalidLanguage(String),
    BuildNotifier,
    BuildFailed,
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::OpenConfigFileError(e) => write!(f, "Failed to open config file: {}", e),
            AgentError::ParseConfigError(e) => write!(f, "Failed to parse config file: {}", e),
            AgentError::InvalidLanguage(e) => write!(f, "Invalid language: {}", e),
            AgentError::BuildNotifier => {
                write!(f, "Could not get notification from build notifier")
            }
            AgentError::BuildFailed => write!(f, "Build has failed"),
        }
    }
}

pub type AgentResult<T> = Result<T, AgentError>;

pub mod agent {
    tonic::include_proto!("cloudlet.agent");
}
