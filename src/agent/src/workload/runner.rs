use crate::{
    agent::ExecuteRequest,
    agents::{rust, Agent, AgentOutput, Language},
    workload::config::Action,
    AgentError, AgentResult,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "debug-agent")]
use crate::agents::debug;

use super::config::Config;

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

        Runner {
            config,
            agent,
            child_processes,
        }
    }

    pub fn new_from_execute_request(execute_request: ExecuteRequest) -> Result<Self, AgentError> {
        let config = Config::new_from_execute_request(execute_request)?;
        Ok(Self::new(config))
    }

    pub async fn run(&self) -> AgentResult<AgentOutput> {
        let result = match self.config.action {
            Action::Prepare => self.agent.prepare(&self.child_processes).await?,
            Action::Run => self.agent.run(&self.child_processes).await?,
            Action::PrepareAndRun => {
                let res = self.agent.prepare(&self.child_processes).await?;
                println!("Prepare result {:?}", res);
                self.agent.run(&self.child_processes).await?
            }
        };

        println!("Result: {:?}", result);

        Ok(result)
    }
}
