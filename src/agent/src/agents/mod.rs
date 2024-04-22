use crate::AgentResult;
use serde::Deserialize;

pub mod rust;

#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait Agent {
    fn prepare(&self) -> AgentResult<AgentOutput>;
    fn run(&self) -> AgentResult<AgentOutput>;
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    Rust,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "rust"),
        }
    }
}
