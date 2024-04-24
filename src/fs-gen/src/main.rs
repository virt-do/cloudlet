use std::{fs::remove_dir_all, path::Path};
use tracing::{debug, error, info, Level};
use anyhow::{Result, Error, bail, Context};

use crate::initramfs_generator::{create_init_file, generate_initramfs, insert_agent};
use crate::image_builder::merge_layer;

mod cli_args;
mod image_builder;
mod image_loader;
mod initramfs_generator;
mod errors;

fn main() -> Result<()> {
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
            error!(error = ?e, "image loader error");
            bail!(e)
        }
        Ok(layers_paths) => {
            debug!("Layers' paths: {:?}", layers_paths);

            let path = Path::new(output_subdir.as_path());

            merge_layer(&layers_paths, path, &overlay_subdir).expect("Merging layers failed");

            if let Err(e) = create_init_file(path, args.initfile_path) {
                error!(error = ?e, "while creating init file");
                bail!(e)
            }

            if let Err(e) = insert_agent(path, args.agent_host_path) {
                error!(error = ?e, "while inserting agent");
                bail!(e)
            }

            if let Err(e) = generate_initramfs(path, Path::new(args.output_file.as_path())) {
                error!(error = ?e, "while generating initramfs");
                bail!(e)
            }

            // cleanup of temporary directory
            if let Err(e) = remove_dir_all(args.temp_directory.clone())
                .with_context(|| "Failed to remove temporary directory".to_string()) {
                error!(?e, "");
                bail!(e)
            }

            Ok(())
        }
    }
}
