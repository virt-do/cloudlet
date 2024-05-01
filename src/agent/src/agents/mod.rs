use crate::{AgentError, AgentResult};
use serde::Deserialize;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "debug-agent")]
pub mod debug;
pub mod rust;

#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait Agent {
    fn prepare<'a>(
        &'a self,
        child_processes: &'a Arc<Mutex<HashSet<u32>>>,
    ) -> Pin<Box<dyn Future<Output = AgentResult<AgentOutput>> + Send + '_>>;
    fn run<'a>(
        &'a self,
        child_processes: &'a Arc<Mutex<HashSet<u32>>>,
    ) -> Pin<Box<dyn Future<Output = AgentResult<AgentOutput>> + Send + '_>>;
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    Rust,
    #[cfg(feature = "debug-agent")]
    Debug,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "rust"),
            #[cfg(feature = "debug-agent")]
            Language::Debug => write!(f, "debug"),
        }
    }
}

impl TryFrom<&str> for Language {
    type Error = AgentError;

    fn try_from(value: &str) -> Result<Self, AgentError> {
        match value {
            "rust" => Ok(Language::Rust),
            #[cfg(feature = "debug-agent")]
            "debug" => Ok(Language::Debug),
            _ => Err(AgentError::InvalidLanguage(format!(
                "Invalid language: {}",
                value
            ))),
        }
    }
}
