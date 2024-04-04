use std::{fs, path::PathBuf, str::FromStr};

use image_builder::merge_layer;

mod cli_args;
mod image_builder;

fn main() {
    // let args = CliArgs::get_args();
    // println!("Hello, world!, {:?}", args);

    let paths: Vec<PathBuf> =
        vec![PathBuf::from_str("/home/spse/Downloads/image-gen/layer").unwrap()];

    merge_layer(&paths, &PathBuf::from_str("./titi").unwrap());
}
