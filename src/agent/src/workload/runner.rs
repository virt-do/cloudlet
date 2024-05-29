use super::config::Config;
use crate::{
    agent::ExecuteRequest,
    agents::{rust, Agent, AgentOutput, Language},
    workload::config::Action,
    AgentError, AgentResult,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc::Receiver, Mutex};

#[cfg(feature = "debug-agent")]
use crate::agents::debug;

/// Runner for a workload.  
/// Will execute the workload based on the inner agent (language).
pub struct Runner {
    config: Config,
    agent: Box<dyn Agent + Sync + Send>,
    child_processes: Arc<Mutex<HashSet<u32>>>,
}

impl Runner {
    pub fn new(config: Config, child_processes: Arc<Mutex<HashSet<u32>>>) -> Self {
        let agent: Box<dyn Agent + Sync + Send> = match config.language {
            Language::Rust => Box::new(rust::RustAgent::from(config.clone())),
            #[cfg(feature = "debug-agent")]
            Language::Debug => Box::new(debug::DebugAgent::from(config.clone())),
        };

        Self {
            config,
            agent,
            child_processes,
        }
    }

    pub fn new_from_execute_request(
        execute_request: ExecuteRequest,
        child_processes: Arc<Mutex<HashSet<u32>>>,
    ) -> Result<Self, AgentError> {
        let config = Config::new_from_execute_request(execute_request)?;
        Ok(Self::new(config, child_processes))
    }

    pub async fn run(&self) -> AgentResult<Receiver<AgentOutput>> {
        let rx = match self.config.action {
            Action::Prepare => {
                self.agent
                    .prepare(Arc::clone(&self.child_processes))
                    .await?
            }
            Action::Run => self.agent.run(Arc::clone(&self.child_processes)).await?,
            Action::PrepareAndRun => {
                // should merge with run rx?
                let _ = self
                    .agent
                    .prepare(Arc::clone(&self.child_processes))
                    .await?;

                self.agent.run(Arc::clone(&self.child_processes)).await?
            }
        };

        Ok(rx)
    }
}
