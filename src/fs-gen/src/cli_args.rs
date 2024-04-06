use std::{env, path::PathBuf};

use clap::{command, error::ErrorKind, CommandFactory, Parser};
use regex::Regex;

use once_cell::sync::Lazy;

// So, for any of you who may be scared, this is the regex from the OCI Distribution Sepcification for the image name + the tag
static RE_IMAGE_NAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*(\/[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*)*:[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}").unwrap()
});

/// Convert an OCI image into a CPIO file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// The name of the image to download
    pub image_name: String,

    /// The path to the output file
    #[arg(short, long, default_value=get_default_log_path().into_os_string())]
    pub ouput_file: PathBuf,

    /// The host path to the guest agent binary
    pub agent_host_path: PathBuf,

    /// The target path of the guest agent binary
    #[arg(short, long, default_value=get_default_target_agent_path().into_os_string())]
    pub agent_target_path: PathBuf,
}

impl CliArgs {
    /// Get the cli arguments with additional validation
    pub fn get_args() -> Self {
        let args = CliArgs::parse();

        args.validate_image();
        args.validate_host_path();

        args
    }

    fn validate_image(&self) {
        if !RE_IMAGE_NAME.is_match(&self.image_name) {
            let mut cmd = CliArgs::command();
            cmd.error(
                ErrorKind::InvalidValue,
                format!("Invalid image name: \"{}\"", self.image_name),
            )
            .exit();
        }
    }

    fn validate_host_path(&self) {
        if !self.agent_host_path.exists() {
            let mut cmd = CliArgs::command();
            cmd.error(
                ErrorKind::InvalidValue,
                format!(
                    "File not found for agent binary: \"{}\"",
                    self.agent_host_path.to_string_lossy()
                ),
            )
            .exit();
        }
    }
}

/// Get the default output path for the cpio file.
fn get_default_log_path() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.push("output.cpio");
    path
}

fn get_default_target_agent_path() -> PathBuf {
    PathBuf::from("/usr/bin/agent")
}
