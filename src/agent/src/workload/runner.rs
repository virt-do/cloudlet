use super::config::{Action, Config};
use crate::{
    agents::{rust, Agent, Language},
    AgentResult,
};

/// Runner for a workload.  
/// Will execute the workload based on the inner agent (language).
pub struct Runner {
    config: Config,
    agent: Box<dyn Agent>,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        let agent: Box<dyn Agent> = match config.language {
            Language::Rust => Box::new(rust::RustAgent::from(config.clone())),
        };

        Runner { config, agent }
    }

    pub fn run(&self) -> AgentResult<()> {
        match self.config.action {
            Action::Prepare => self.agent.prepare()?,
            Action::Run => self.agent.run()?,
            Action::PrepareAndRun => {
                self.agent.prepare()?;
                self.agent.run()?;
            }
        }

        Ok(())
    }
}
