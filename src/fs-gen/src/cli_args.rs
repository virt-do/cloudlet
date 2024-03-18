use std::{env, path::PathBuf};

use clap::{command, Parser};
/// Convert an OCI image into a CPIO file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// The name of the image to download
    #[arg(short, long)]
    pub image_name: String,

    /// The path to the output file
    #[arg(short, long, default_value=get_default_log_path().into_os_string())]
    pub ouput_file: PathBuf,
}

/// Get the default output path for the cpio file.
fn get_default_log_path() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.push("output.cpio");
    path
}
