mod rust;
mod agent;

use std::path::PathBuf;

use agent::{Action, Agent, AgentConfig, AgentResult};

pub struct AgentRunner {
    config: AgentConfig,
    agent: Box<dyn Agent>,
}

impl AgentRunner {
    pub fn new(config_path: String) -> Self {
        let config = AgentConfig::new_from_file(&PathBuf::from(config_path)).unwrap();

        let agent: Box<dyn Agent> = match config.language.as_str() {
            "rust" => Box::new(rust::RustAgent::new_from_config(config.clone())),
            _ => panic!("Unsupported language: {}", config.language),
        };

        AgentRunner {
            config,
            agent,
        }
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
