use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

pub struct ConfigFileHandler {}

impl ConfigFileHandler {
    pub fn read_file(file_path: &PathBuf) -> io::Result<String> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }
}
