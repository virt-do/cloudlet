use std::{fs::create_dir_all, process::Command};

use rand::distributions::{Alphanumeric, DistString};
use serde::Deserialize;

use crate::{Agent, AgentConfig, AgentResult};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RustAgentBuildConfig {
    release: bool,
    source_code_path: String,
}

#[derive(Deserialize)]
struct RustAgentConfig {
    build: RustAgentBuildConfig,
}

pub struct RustAgent {
    agent_config: AgentConfig,
    rust_config: RustAgentConfig,
}

impl RustAgent {
    fn build(&self, function_dir: &String) -> AgentResult<String> {
        if self.rust_config.build.release {
            let output = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(function_dir)
                .output()
                .expect("Failed to build function");

            Ok(std::str::from_utf8(&output.stdout).unwrap().to_string())
        } else {
            let output = Command::new("cargo")
                .arg("build")
                .current_dir(function_dir)
                .output()
                .expect("Failed to build function");

            Ok(std::str::from_utf8(&output.stdout).unwrap().to_string())
        }
    }

    pub fn new_from_config(agent_config: AgentConfig) -> Self {
        let rust_config: RustAgentConfig = toml::from_str(&agent_config.config_string).unwrap();

        RustAgent {
            agent_config,
            rust_config,
        }
    }
}

impl Agent for RustAgent {
    fn prepare(&self) -> AgentResult<()> {
        let code = std::fs::read_to_string(&self.rust_config.build.source_code_path).unwrap();

        let function_dir = format!("/tmp/{}", Alphanumeric.sample_string(&mut rand::thread_rng(), 16));

        println!("Function directory: {}", function_dir);

        create_dir_all(format!("{}/src", &function_dir)).expect("Unable to create directory");

        std::fs::write(format!("{}/src/main.rs", &function_dir), code).expect("Unable to write main.rs file");
    
        let cargo_toml = format!(r#"
            [package]
            name = "{}"
            version = "0.1.0"
            edition = "2018"
        "#, self.agent_config.workload_name);

        std::fs::write(format!("{}/Cargo.toml",&function_dir), cargo_toml).expect("Unable to write Cargo.toml file");

        let result = self.build(&function_dir)?;

        println!("{}", result);

        // Copy the binary to /tmp, we could imagine a more complex scenario where we would put this in an artifact repository (like S3)
        let binary_path = match self.rust_config.build.release {
            true => format!("{}/target/release/{}", &function_dir, self.agent_config.workload_name),
            false => format!("{}/target/debug/{}", &function_dir, self.agent_config.workload_name),
        };

        std::fs::copy(binary_path, format!("/tmp/{}", self.agent_config.workload_name)).expect("Unable to copy binary");

        std::fs::remove_dir_all(&function_dir).expect("Unable to remove directory");

        Ok(())
    }

    fn run(&self) -> AgentResult<()> {
        let output = Command::new(format!("/tmp/{}", self.agent_config.workload_name))
            .output()
            .expect("Failed to run function");

        println!("{}", std::str::from_utf8(&output.stdout).unwrap());

        println!("{}", std::str::from_utf8(&output.stderr).unwrap());

        Ok(())
    }
}
