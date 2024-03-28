use std::{env, path::PathBuf};

use clap::{command, Parser};
use regex::Regex;

use once_cell::sync::Lazy;
use validator::Validate;

// So, for any of you who may be scared, this is the regex from the OCI Distribution Sepcification for the image name + the tag
static RE_IMAGE_NAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*(\/[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*)*:[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}").unwrap()
});

/// Convert an OCI image into a CPIO file
#[derive(Parser, Debug, Validate)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// The name of the image to download

    #[arg(short, long)]
    #[validate(regex(path = *RE_IMAGE_NAME))]
    pub image_name: String,

    /// The path to the output file
    #[arg(short, long, default_value=get_default_log_path().into_os_string())]
    pub ouput_file: PathBuf,
}

impl CliArgs {
    /// Get the cli arguments with additional validation
    pub fn get_args() -> Self {
        let args = CliArgs::parse();

        let validation = args.validate();
        if validation.is_err() {
            panic!("Invalid arguments: {}", validation.expect_err("wut"));
        }

        args
    }
}

/// Get the default output path for the cpio file.
fn get_default_log_path() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.push("output.cpio");
    path
}
