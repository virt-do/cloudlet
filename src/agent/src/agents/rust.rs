use super::{Agent, AgentOutput};
use crate::agents::process_utils;
use crate::{workload, AgentError, AgentResult};
use async_trait::async_trait;
use rand::distributions::{Alphanumeric, DistString};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{
    broadcast,
    mpsc::{self, Receiver},
    Mutex,
};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RustAgentBuildConfig {
    release: bool,
}

#[derive(Deserialize)]
struct RustAgentConfig {
    build: RustAgentBuildConfig,
}

pub struct RustAgent {
    workload_config: workload::config::Config,
    rust_config: RustAgentConfig,
    build_notifier: broadcast::Sender<Result<(), ()>>,
}

// TODO should change with a TryFrom
impl From<workload::config::Config> for RustAgent {
    fn from(workload_config: workload::config::Config) -> Self {
        let rust_config: RustAgentConfig = toml::from_str(&workload_config.config_string).unwrap();

        Self {
            workload_config,
            rust_config,
            build_notifier: broadcast::channel::<Result<(), ()>>(1).0,
        }
    }
}

impl RustAgent {
    async fn get_build_child_process(
        &self,
        function_dir: &str,
        child_processes: Arc<Mutex<HashSet<u32>>>,
    ) -> Child {
        let mut command = Command::new("cargo");
        let command = if self.rust_config.build.release {
            command
                .stderr(Stdio::piped())
                .arg("build")
                .current_dir(function_dir)
                .arg("--release")
        } else {
            command
                .stderr(Stdio::piped())
                .arg("build")
                .current_dir(function_dir)
        };
        let child = command.spawn().expect("Failed to start build");

        {
            child_processes.lock().await.insert(child.id().unwrap());
        }

        child
    }
}

#[async_trait]
impl Agent for RustAgent {
    async fn prepare(
        &self,
        child_processes: Arc<Mutex<HashSet<u32>>>,
    ) -> AgentResult<Receiver<AgentOutput>> {
        let function_dir = format!(
            "/tmp/{}",
            Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
        );

        println!("Function directory: {}", function_dir);

        create_dir_all(format!("{}/src", &function_dir)).expect("Unable to create directory");

        std::fs::write(
            format!("{}/src/main.rs", &function_dir),
            &self.workload_config.code,
        )
        .expect("Unable to write main.rs file");

        let cargo_toml = format!(
            r#"
            [package]
            name = "{}"
            version = "0.1.0"
            edition = "2021"
        "#,
            self.workload_config.workload_name
        );

        std::fs::write(format!("{}/Cargo.toml", &function_dir), cargo_toml)
            .expect("Unable to write Cargo.toml file");

        let mut child = self
            .get_build_child_process(&function_dir, child_processes)
            .await;
        let workload_name = self.workload_config.workload_name.clone();
        let is_release = self.rust_config.build.release;
        let tx_build_notifier = self.build_notifier.clone();

        let (tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            let _ = process_utils::send_stderr_to_tx(child.stderr.take().unwrap(), tx.clone()).await.await;
            let build_result = process_utils::send_exit_status_to_tx(child, tx, false).await;
            // if error in build, short-circuit the execution
            if build_result.is_err() {
                let _ = tx_build_notifier.send(Err(()));
            } else {
                // Once finished: copy the binary to /tmp
                // We could imagine a more complex scenario where we would put this in an artifact repository (like S3)
                let binary_path = match is_release {
                    true => format!("{}/target/release/{}", &function_dir, workload_name),
                    false => format!("{}/target/debug/{}", &function_dir, workload_name),
                };

                std::fs::copy(binary_path, format!("/tmp/{}", workload_name))
                    .expect("Unable to copy binary");

                // notify when build is done
                let _ = tx_build_notifier.send(build_result);
            }

            std::fs::remove_dir_all(&function_dir).expect("Unable to remove directory");
        });

        Ok(rx)
    }

    async fn run(
        &self,
        child_processes: Arc<Mutex<HashSet<u32>>>,
    ) -> AgentResult<Receiver<AgentOutput>> {
        // wait for build to finish
        self.build_notifier
            .subscribe()
            .recv()
            .await
            .map_err(|_| AgentError::BuildNotifier)?
            .map_err(|_| AgentError::BuildFailed)?;

        println!("Starting run()");
        let mut child = Command::new(format!("/tmp/{}", self.workload_config.workload_name))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to run function");

        {
            child_processes.lock().await.insert(child.id().unwrap());
        }

        let (tx, rx) = mpsc::channel(10);
        let child_stdout = child.stdout.take().unwrap();
        let tx_stdout = tx.clone();
        let child_stderr = child.stderr.take().unwrap();
        let tx_stderr = tx;

        tokio::spawn(async move {
            let _ = process_utils::send_stdout_to_tx(child_stdout, tx_stdout.clone()).await.await;
            let _ = process_utils::send_exit_status_to_tx(child, tx_stdout, true).await;
        });

        tokio::spawn(async move {
            process_utils::send_stderr_to_tx(child_stderr, tx_stderr).await;
        });

        Ok(rx)
    }
}
