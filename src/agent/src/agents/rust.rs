use super::{Agent, AgentOutput};
use crate::{workload, AgentError, AgentResult};
use rand::distributions::{Alphanumeric, DistString};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

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
}

impl RustAgent {
    async fn build(
        &self,
        function_dir: &String,
        child_processes: &Arc<Mutex<HashSet<u32>>>,
    ) -> AgentResult<AgentOutput> {
        if self.rust_config.build.release {
            let child = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(function_dir)
                .spawn()
                .expect("Failed to build function");

            child_processes.lock().await.insert(child.id().unwrap());

            let output = child
                .wait_with_output()
                .await
                .expect("Failed to wait on child");

            Ok(AgentOutput {
                exit_code: output.status.code().unwrap(),
                stdout: std::str::from_utf8(&output.stdout).unwrap().to_string(),
                stderr: std::str::from_utf8(&output.stderr).unwrap().to_string(),
            })
        } else {
            let child = Command::new("cargo")
                .arg("build")
                .current_dir(function_dir)
                .spawn()
                .expect("Failed to build function");

            let output = child
                .wait_with_output()
                .await
                .expect("Failed to wait on child");

            Ok(AgentOutput {
                exit_code: output.status.code().unwrap(),
                stdout: std::str::from_utf8(&output.stdout).unwrap().to_string(),
                stderr: std::str::from_utf8(&output.stderr).unwrap().to_string(),
            })
        }
    }
}

// TODO should change with a TryFrom
impl From<workload::config::Config> for RustAgent {
    fn from(workload_config: workload::config::Config) -> Self {
        let rust_config: RustAgentConfig = toml::from_str(&workload_config.config_string).unwrap();

        Self {
            workload_config,
            rust_config,
        }
    }
}

impl Agent for RustAgent {
    fn prepare<'a>(
        &'a self,
        child_processes: &'a Arc<Mutex<HashSet<u32>>>,
    ) -> Pin<Box<dyn Future<Output = AgentResult<AgentOutput>> + Send + '_>> {
        Box::pin(async {
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
            edition = "2018"
        "#,
                self.workload_config.workload_name
            );

            std::fs::write(format!("{}/Cargo.toml", &function_dir), cargo_toml)
                .expect("Unable to write Cargo.toml file");

            let result = self.build(&function_dir, child_processes).await?;

            if result.exit_code != 0 {
                println!("Build failed: {:?}", result);
                return Err(AgentError::BuildFailed(AgentOutput {
                    exit_code: result.exit_code,
                    stdout: result.stdout,
                    stderr: result.stderr,
                }));
            }

            // Copy the binary to /tmp, we could imagine a more complex scenario where we would put this in an artifact repository (like S3)
            let binary_path = match self.rust_config.build.release {
                true => format!(
                    "{}/target/release/{}",
                    &function_dir, self.workload_config.workload_name
                ),
                false => format!(
                    "{}/target/debug/{}",
                    &function_dir, self.workload_config.workload_name
                ),
            };

            std::fs::copy(
                binary_path,
                format!("/tmp/{}", self.workload_config.workload_name),
            )
            .expect("Unable to copy binary");

            std::fs::remove_dir_all(&function_dir).expect("Unable to remove directory");

            Ok(AgentOutput {
                exit_code: result.exit_code,
                stdout: "Build successful".to_string(),
                stderr: "".to_string(),
            })
        })
    }

    fn run<'a>(
        &'a self,
        child_processes: &'a Arc<Mutex<HashSet<u32>>>,
    ) -> Pin<Box<dyn Future<Output = AgentResult<AgentOutput>> + Send + '_>> {
        Box::pin(async {
            let child = Command::new(format!("/tmp/{}", self.workload_config.workload_name))
                .spawn()
                .expect("Failed to run function");

            child_processes.lock().await.insert(child.id().unwrap());

            let output = child
                .wait_with_output()
                .await
                .expect("Failed to wait on child");

            let agent_output = AgentOutput {
                exit_code: output.status.code().unwrap(),
                stdout: std::str::from_utf8(&output.stdout).unwrap().to_string(),
                stderr: std::str::from_utf8(&output.stderr).unwrap().to_string(),
            };

            if !output.status.success() {
                println!("Run failed: {:?}", agent_output);
                return Err(AgentError::BuildFailed(agent_output));
            }

            Ok(agent_output)
        })
    }
}
