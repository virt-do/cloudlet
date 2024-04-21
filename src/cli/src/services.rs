use crate::utils::ConfigFileHandler;
use reqwest::Client;
use shared_models::{CloudletDtoRequest, YamlClientConfigFile};
use std::error::Error;
pub struct CloudletClient {}

impl CloudletClient {
    pub fn new_cloudlet_config(config: YamlClientConfigFile) -> CloudletDtoRequest {
        let code: String = ConfigFileHandler::read_file(&config.code_path)
            .expect("Error while reading the code file");
        let env = ConfigFileHandler::read_file(&config.env_path)
            .expect("Error while reading the environment file");
        let language = config.language;
        let log_level = config.log_level;
        CloudletDtoRequest {
            language,
            env,
            code,
            log_level,
        }
    }

    pub async fn run(request: CloudletDtoRequest) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        let json = serde_json::to_string(&request)?;
        println!("REQUEST : {:?}", request);
        let res = client
            .post("http://127.0.0.1:3000/run")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(json)
            .send()
            .await?;

        match res.status().as_u16() {
            200 => Ok(()),
            _ => Err("Error while making the request".into()),
        }
    }
}
