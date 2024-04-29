use shared_models::YamlApiConfigFile;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

pub fn load_config(config_path: &PathBuf) -> io::Result<YamlApiConfigFile> {
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config_result = serde_yaml::from_str(&contents);
    if let Err(e) = config_result {
        return Err(io::Error::new(io::ErrorKind::InvalidData, e));
    }
    return Ok(config_result.unwrap());
}
