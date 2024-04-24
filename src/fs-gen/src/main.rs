use std::{fs::remove_dir_all, path::Path};
use std::path::PathBuf;
use tracing::{debug, error, info, Level};
use anyhow::{Result, Error, bail, Context};
use crate::cli_args::CliArgs;

use crate::initramfs_generator::{create_init_file, generate_initramfs, insert_agent};
use crate::image_builder::merge_layer;

mod cli_args;
mod image_builder;
mod image_loader;
mod initramfs_generator;
mod errors;

fn run(
    args: CliArgs,
    layers_subdir: PathBuf,
    output_subdir: PathBuf,
    overlay_subdir: PathBuf,
) -> Result<()> {
    let path = Path::new(output_subdir.as_path());

    // image downloading and unpacking
    let layers_paths = image_loader::download_image_fs(&args.image_name, layers_subdir)?;
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
        debug = args.debug,
        "arguments:",
    );

    if let Err(e) = run(
        args,
        args.temp_directory.clone().join("layers/"),
        args.temp_directory.clone().join("output/"),
        args.temp_directory.clone().join("overlay/")
    ) {
        error!(error = ?e, "encountered error while running");
        Err(e)
    } else {
        info!("Finished successfully!");
        Ok(())
    }
}
