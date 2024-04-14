use std::{fs, path::PathBuf, str::FromStr};

use image_builder::merge_layer;

mod cli_args;
mod image_builder;
mod image_loader;

fn main() {
    let args = cli_args::CliArgs::get_args();
    println!("Hello, world!, {:?}", args);

    // let paths: Vec<PathBuf> =
    //     vec![PathBuf::from_str("/home/spse/Downloads/image-gen/layer").unwrap()];

    // merge_layer(&paths, &PathBuf::from_str("./titi").unwrap());

    match image_loader::download_image_fs(&args.image_name, args.output_file) {
        Err(e) => eprintln!("Error: {}", e),
        Ok(layers_paths) => {
            println!("Image downloaded successfully! Layers' paths:");
            for path in layers_paths {
                println!(" - {}", path.display());
            }
        }
    }
}
