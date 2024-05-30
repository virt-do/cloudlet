use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, ValueEnum, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    RUST,
    PYTHON,
    NODE,
}

#[derive(Clone, Debug, ValueEnum, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

#[derive(Deserialize, Debug)]
pub struct TomlClientConfigFile {
    pub worklaod_name: String,
    pub language: Language,
    pub code_path: PathBuf,
    pub log_level: LogLevel,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CloudletDtoRequest {
    pub workload_name: String,
    pub language: Language,
    pub code: String,
    pub log_level: LogLevel,
    pub action: String,
    pub server: ServerConfig,
    pub build: BuildConfig,
}

#[derive(Debug, Deserialize)]
pub struct CloudletShutdownResponse {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            address: String::from("localhost"),
            port: 50051
        }

    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BuildConfig {
    #[serde(rename = "source-code-path")]
    pub source_code_path: PathBuf,
    pub release: bool,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct AgentExecuteDtoRequest {}
