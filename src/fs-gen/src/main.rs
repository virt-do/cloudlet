use anyhow::{bail, Context, Result};
use std::{fs::remove_dir_all, path::Path};
use tracing::level_filters::LevelFilter;
use tracing::{debug, error, info};
use tracing_subscriber::filter::EnvFilter;

use crate::cli_args::CliArgs;
use crate::image_builder::merge_layer;
use crate::initramfs_generator::{create_init_file, generate_initramfs, insert_agent};
use crate::loader::download::download_image_fs;

mod cli_args;
mod image_builder;
mod initramfs_generator;
mod loader;

fn run(args: CliArgs) -> Result<()> {
    let layers_subdir = args.temp_directory.join("layers/");
    let overlay_subdir = args.temp_directory.join("overlay/");
    let _binding = args.temp_directory.join("output/");
    let output_subdir = _binding.as_path();

    // image downloading and unpacking
    let layers_paths = match download_image_fs(&args.image_name, &args.architecture, layers_subdir)
    {
        Err(e) => bail!(e),
        Ok(e) => e,
    };
    debug!("Layers' paths: {:?}", layers_paths);

    // reconstructing image with overlayfs
    merge_layer(&layers_paths, output_subdir, &overlay_subdir)?;

    // building initramfs
    create_init_file(output_subdir, args.initfile_path)?;
    insert_agent(output_subdir, args.agent_host_path)?;
    generate_initramfs(output_subdir, Path::new(args.output_file.as_path()))?;

    // cleanup of temporary directory
    remove_dir_all(args.temp_directory.clone())
        .with_context(|| "Failed to remove temporary directory".to_string())?;

    Ok(())
}

fn main() -> Result<()> {
    let args = CliArgs::get_args();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(
                    (if args.debug {
                        LevelFilter::DEBUG
                    } else {
                        LevelFilter::INFO
                    })
                    .into(),
                )
                .from_env()?
                .add_directive("fuse_backend_rs=warn".parse()?),
        )
        .init();

    info!(
        "Cloudlet initramfs generator: '{}' v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
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
