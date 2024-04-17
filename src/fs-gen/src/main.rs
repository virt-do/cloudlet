//! Main module for the initramfs tarball generation

use crate::{cli_args::CliArgs, image_builder::build_new_image};
use clap::Parser;
use std::{path::PathBuf, str::FromStr};

mod cli_args;
mod image_builder;

fn main() {
    let args = CliArgs::get_args();
}
