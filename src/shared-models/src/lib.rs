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
}

#[derive(Serialize, Deserialize, Debug)]

pub struct AgentExecuteDtoRequest {}
