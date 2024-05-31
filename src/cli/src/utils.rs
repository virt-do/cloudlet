use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn read_file(file_path: &PathBuf) -> std::io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
