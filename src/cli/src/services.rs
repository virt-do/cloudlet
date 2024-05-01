use crate::utils::ConfigFileHandler;
use reqwest::Client;
use serde::Deserialize;
use shared_models::{BuildConfig, CloudletDtoRequest, Language, ServerConfig};
use std::error::Error;

#[derive(Deserialize, Debug)]
struct TomlConfig {
    #[serde(rename = "workload-name")]
    workload_name: String,
    language: Language,
    action: String,
    server: ServerConfig,
    build: BuildConfig,
}

pub struct CloudletClient {}

impl CloudletClient {
    pub fn new_cloudlet_config(config: String) -> CloudletDtoRequest {
        let config: TomlConfig =
            toml::from_str(&config).expect("Error while parsing the config file");

        let workload_name = config.workload_name;
        let code: String = ConfigFileHandler::read_file(&config.build.source_code_path)
            .expect("Error while reading the code file");

        let language = config.language;
        CloudletDtoRequest {
            workload_name,
            language,
            code,
            log_level: shared_models::LogLevel::INFO,
            server: config.server,
            build: config.build,
            action: config.action,
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
