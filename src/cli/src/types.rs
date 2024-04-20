use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, ValueEnum, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    Node,
}

#[derive(Clone, Debug, ValueEnum, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Deserialize, Debug)]
pub struct YamlConfigFile {
    pub language: Language,
    pub env_path: PathBuf,
    pub code_path: PathBuf,
    pub log_level: LogLevel,
}
