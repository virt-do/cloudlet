use std::{fs::remove_dir_all, path::Path};
use tracing::{debug, error, info, Level};
use anyhow::{Result, bail, Context};
use crate::cli_args::CliArgs;

use crate::initramfs_generator::{create_init_file, generate_initramfs, insert_agent};
use crate::image_builder::merge_layer;
use crate::loader::download::download_image_fs;

mod cli_args;
mod image_builder;
mod initramfs_generator;
mod loader;

fn run(
    args: CliArgs,
) -> Result<()> {
    let layers_subdir = args.temp_directory.join("layers/");
    let output_subdir = args.temp_directory.join("output/");
    let overlay_subdir = args.temp_directory.join("overlay/");

    let path = Path::new(output_subdir.as_path());

    // image downloading and unpacking
    let layers_paths = match download_image_fs(&args.image_name, &args.architecture, layers_subdir) {
        Err(e) => bail!(e),
        Ok(e) => e
    };
    debug!("Layers' paths: {:?}", layers_paths);

    // reconstructing image with overlayfs
    merge_layer(&layers_paths, path, &overlay_subdir)?;

    // building initramfs
    create_init_file(path, args.initfile_path)?;
    insert_agent(path, args.agent_host_path)?;
    generate_initramfs(path, Path::new(args.output_file.as_path()))?;

    // cleanup of temporary directory
    remove_dir_all(args.temp_directory.clone())
        .with_context(|| "Failed to remove temporary directory".to_string())?;

    Ok(())
}

fn main() -> Result<()> {
    let args = CliArgs::get_args();

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
        architecture = args.architecture,
        debug = args.debug,
        "arguments:",
    );

    if let Err(e) = run(args) {
        error!(error = ?e, "encountered error while running");
        Err(e)
    } else {
        info!("Finished successfully!");
        Ok(())
    }
}
