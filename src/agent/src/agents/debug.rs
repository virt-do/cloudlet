use super::AgentOutput;
use crate::agent::execute_response::Stage;
use crate::agents::Agent;
use crate::{workload, AgentResult};
use async_trait::async_trait;
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::Mutex;

pub struct DebugAgent {
    workload_config: workload::config::Config,
}

impl From<workload::config::Config> for DebugAgent {
    fn from(workload_config: workload::config::Config) -> Self {
        Self { workload_config }
    }
}

#[async_trait]
impl Agent for DebugAgent {
    async fn prepare(&self, _: Arc<Mutex<HashSet<u32>>>) -> AgentResult<Receiver<AgentOutput>> {
        let dir = format!("/tmp/{}", self.workload_config.workload_name);

        println!("Function directory: {}", dir);

        create_dir_all(&dir).expect("Unable to create directory");

        std::fs::write(
            format!("{}/debug.txt", &dir),
            format!(
                "Debug agent for {} - written at {:?}",
                self.workload_config.workload_name,
                SystemTime::now(),
            ),
        )
        .expect("Unable to write debug.txt file");

        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let _ = tx
                .send(AgentOutput {
                    stage: Stage::Building,
                    stdout: Some("Build successfully!".into()),
                    stderr: None,
                    exit_code: None,
                })
                .await;
        });

        Ok(rx)
    }

    async fn run(&self, _: Arc<Mutex<HashSet<u32>>>) -> AgentResult<Receiver<AgentOutput>> {
        let dir = format!("/tmp/{}", self.workload_config.workload_name);

        let content = std::fs::read_to_string(format!("{}/debug.txt", &dir));

        std::fs::remove_dir_all(dir).expect("Unable to remove directory");

        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            if let Ok(content) = content {
                let _ = tx
                    .send(AgentOutput {
                        stage: Stage::Done,
                        stdout: Some(content),
                        stderr: None,
                        exit_code: Some(0),
                    })
                    .await;
            }

            let _ = tx
                .send(AgentOutput {
                    stage: Stage::Failed,
                    stdout: None,
                    stderr: Some("unable to read debug.txt".into()),
                    exit_code: Some(1),
                })
                .await;
        });

        Ok(rx)
    }
}
