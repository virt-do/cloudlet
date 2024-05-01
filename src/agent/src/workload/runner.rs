use crate::{
    agent::ExecuteRequest,
    agents::{rust, Agent, AgentOutput, Language},
    workload::config::Action,
    AgentResult,
};

#[cfg(feature = "debug-agent")]
use crate::agents::debug;

use super::config::Config;

/// Runner for a workload.  
/// Will execute the workload based on the inner agent (language).
pub struct Runner {
    config: Config,
    agent: Box<dyn Agent + Sync + Send>,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        let agent: Box<dyn Agent + Sync + Send> = match config.language {
            Language::Rust => Box::new(rust::RustAgent::from(config.clone())),
            #[cfg(feature = "debug-agent")]
            Language::Debug => Box::new(debug::DebugAgent::from(config.clone())),
        };

        Runner { config, agent }
    }

    pub fn new_from_execute_request(execute_request: ExecuteRequest) -> Self {
        let config = Config::new_from_execute_request(execute_request);
        Self::new(config)
    }

    pub fn run(&self) -> AgentResult<AgentOutput> {
        let result = match self.config.action {
            Action::Prepare => self.agent.prepare()?,
            Action::Run => self.agent.run()?,
            Action::PrepareAndRun => {
                let res = self.agent.prepare()?;
                println!("Prepare result {:?}", res);
                self.agent.run()?
            }
        };

        println!("Result: {:?}", result);

        Ok(result)
    }
}
