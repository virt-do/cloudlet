use shared_models::YamlClientConfigFile;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

pub struct ConfigFileHandler {}

impl ConfigFileHandler {
    pub fn load_config(config_path: &PathBuf) -> io::Result<YamlClientConfigFile> {
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: YamlClientConfigFile = serde_yaml::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    pub fn read_file(file_path: &PathBuf) -> io::Result<String> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }
}
