use agents::AgentOutput;

mod agents;
pub mod workload;

#[derive(Debug)]
pub enum AgentError {
    OpenConfigFileError(std::io::Error),
    ParseConfigError(toml::de::Error),
    BuildFailed(AgentOutput),
}

pub type AgentResult<T> = Result<T, AgentError>;

pub mod agent {
    tonic::include_proto!("cloudlet.agent");
}
