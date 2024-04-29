use crate::utils::ConfigFileHandler;
use reqwest::Client;
use serde::Deserialize;
use shared_models::{CloudletDtoRequest, Language};
use std::{error::Error, path::PathBuf};

#[derive(Deserialize)]
struct TomlConfig {
    #[serde(rename = "workload-name")]
    workload_name: String,
    language: Language,
    action: String,
    server: ServerConfig,
    build: BuildConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    address: String,
    port: u16,
}

#[derive(Deserialize)]
struct BuildConfig {
    #[serde(rename = "source-code-path")]
    source_code_path: PathBuf,
    release: bool,
}

pub struct CloudletClient {}

impl CloudletClient {
    pub fn new_cloudlet_config(config: String) -> CloudletDtoRequest {
        let config: TomlConfig =
            toml::from_str(&config).expect("Error while parsing the config file");

        let code: String = ConfigFileHandler::read_file(&config.build.source_code_path)
            .expect("Error while reading the code file");
        let env = "";

        let language = config.language;
        CloudletDtoRequest {
            language,
            code,
            env: env.to_string(),
            log_level: shared_models::LogLevel::INFO,
        }
    }

    pub async fn run(request: CloudletDtoRequest) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        let json = serde_json::to_string(&request)?;
        let res = client
            .post("http://127.0.0.1:3000/run")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(json)
            .send()
            .await?;

        println!("Response: {:?}", res.text().await?);
        Ok(())
    }
}
