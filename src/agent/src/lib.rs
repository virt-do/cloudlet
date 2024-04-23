use agents::AgentOutput;

mod agents;
pub mod workload {
    pub mod config;
    pub mod runner;
}

#[derive(Debug)]
pub enum AgentError {
    OpenConfigFileError(std::io::Error),
    ParseConfigError(toml::de::Error),
    BuildFailed(AgentOutput),
}

pub type AgentResult<T> = Result<T, AgentError>;
