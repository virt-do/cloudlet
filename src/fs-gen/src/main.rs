use std::{fs::remove_dir_all, path::Path};
use tracing::{debug, error, info, Level};
use tracing_subscriber;

use crate::initramfs_generator::{create_init_file, generate_initramfs, insert_agent};
use image_builder::merge_layer;

mod cli_args;
mod image_builder;
mod image_loader;
mod initramfs_generator;

fn main() {
    let args = cli_args::CliArgs::get_args();

    tracing_subscriber::fmt()
        .with_max_level(if args.debug { Level::DEBUG } else { Level::INFO })
        .init();

    info!("Cloudlet initramfs generator v{}", env!("CARGO_PKG_VERSION"));
    info!("Generating for image '{}'", args.image_name);

    debug!(
        image_name = args.image_name,
        agent_host_path = ?args.agent_host_path,
        output_file = ?args.output_file,
        temp_dir = ?args.temp_directory,
        initfile_path = ?args.initfile_path,
        debug = args.debug,
        "arguments:",
    );

    let layers_subdir = args.temp_directory.clone().join("layers/");
    let output_subdir = args.temp_directory.clone().join("output/");
    let overlay_subdir = args.temp_directory.clone().join("overlay/");

    match image_loader::download_image_fs(&args.image_name, layers_subdir) {
        Err(e) => {
            error!(%e, "Received error while downloading image");
            return;
        }
        Ok(layers_paths) => {
            debug!("Layers' paths: {:?}", layers_paths);

            let path = Path::new(output_subdir.as_path());

            merge_layer(&layers_paths, path, &overlay_subdir).expect("Merging layers failed");

            create_init_file(path, args.initfile_path);
            insert_agent(path, args.agent_host_path);

            generate_initramfs(path, Path::new(args.output_file.as_path()));
        }
    }

    // cleanup of temporary directory
    remove_dir_all(args.temp_directory.clone()).expect("Could not remove temporary directory");
}
