use crate::types::{Language, LogLevel};
use reqwest::Client;
use serde::Serialize;
use std::error::Error;

#[derive(Serialize, Debug)]
pub struct HttpRunRequest {
    pub language: Language,
    pub env_content: String,
    pub code_content: String,
    pub log_level: LogLevel,
}

impl HttpRunRequest {
    pub fn new(
        language: Language,
        env_content: String,
        code_content: String,
        log_level: LogLevel,
    ) -> Self {
        HttpRunRequest {
            language,
            env_content,
            code_content,
            log_level,
        }
    }
}

pub async fn run_request(request: HttpRunRequest) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let res = client
        .post("http://127.0.0.1:3000/run")
        .body(serde_json::to_string(&request)?)
        .send()
        .await?;
    println!("Response Status: {}", res.status());
    let body = res.text().await?;
    println!("Response body: {}", body);
    Ok(())
}
