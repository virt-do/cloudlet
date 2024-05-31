use crate::utils;
use serde::Deserialize;
use shared_models::{BuildConfig, CloudletDtoRequest, Language, ServerConfig};
use std::{fs, path::PathBuf};

pub mod execute;
pub mod shutdown;

pub use execute::*;
pub use shutdown::*;

#[derive(Debug)]
pub enum Error {
    ReadTomlConfigFile(std::io::Error),
    TomlConfigParse(toml::de::Error),
    ReadCodeFile(std::io::Error),
    ExecuteRequestBody,
    CreateEventSource(reqwest_eventsource::CannotCloneRequestError),
    ExecuteResponseDeserialize,
    ShutdownSendRequest(reqwest::Error),
    ShutdownResponse(reqwest::Error),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct TomlConfig {
    workload_name: String,
    language: Language,
    action: String,
    server: ServerConfig,
    build: BuildConfig,
}

pub fn new_cloudlet_request(config_path: &PathBuf) -> Result<CloudletDtoRequest, Error> {
    let toml_file = fs::read_to_string(config_path).map_err(Error::ReadTomlConfigFile)?;
    let config: TomlConfig = toml::from_str(&toml_file).map_err(Error::TomlConfigParse)?;

    let source_code_path = &config.build.source_code_path;
    let code: String = utils::read_file(source_code_path).map_err(Error::ReadCodeFile)?;

    Ok(CloudletDtoRequest {
        workload_name: config.workload_name,
        language: config.language,
        code,
        log_level: shared_models::LogLevel::INFO,
        server: config.server,
        build: config.build,
        action: config.action,
    })
}
