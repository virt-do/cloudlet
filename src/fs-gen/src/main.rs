use std::{fs, path::{Path, PathBuf}, str::FromStr};

use image_builder::merge_layer;
use crate::initramfs_generator::{create_init_file, generate_initramfs};

mod cli_args;
mod image_builder;
mod image_loader;
mod initramfs_generator;

fn main() {
    let args = cli_args::CliArgs::get_args();
    println!("Hello, world!, {:?}", args);

    // let paths: Vec<PathBuf> =
    //     vec![PathBuf::from_str("/home/spse/Downloads/image-gen/layer").unwrap()];

    // merge_layer(&paths, &PathBuf::from_str("./titi").unwrap());

    match image_loader::download_image_fs(&args.image_name, args.output_file.clone()) {
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        },
        Ok(layers_paths) => {
            println!("Image downloaded successfully! Layers' paths:");
            for path in &layers_paths {
                println!(" - {}", path.display());
            }

            let path = Path::new("/tmp/cloudlet");

            merge_layer(&layers_paths, path);
            create_init_file(path);
            generate_initramfs(path, Path::new("/tmp/rusty.img"));
        }
    }
}
