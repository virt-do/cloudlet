use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;

use crate::{cli_args::CliArgs, image_builder::build_new_image};

mod cli_args;
mod image_builder;

fn main() {
    let args = CliArgs::get_args();
    println!("Hello, world!, {:?}", args);

    let paths: Vec<PathBuf> =
        vec![PathBuf::from_str("../../image-gen/blobs/sha256/layer_1").unwrap()];

    build_new_image(&paths, &PathBuf::from_str("./titi").unwrap());
}
