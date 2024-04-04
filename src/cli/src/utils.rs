use crate::types::Config;
use std::fs::File;
use std::io::{self, Read};

pub fn load_config(config_path: &str) -> io::Result<Config> {
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_yaml::from_str(&contents).unwrap();
    Ok(config)
}
