use crate::args::CliArguments;
use clap::Parser;
use tracing::{info, level_filters};
use vmm::core::{self, vmm::VMM};

mod args;

#[derive(Debug)]
pub enum Error {
    VmmNew(core::Error),
    VmmConfigure(core::Error),
    VmmRun(core::Error),
}

/// The application entry point.
fn main() -> Result<(), Error> {
    // Parse the configuration and configure logger verbosity
    let args = CliArguments::parse();

    tracing_subscriber::fmt()
        .with_max_level(match args.verbose.log_level_filter() {
            log::LevelFilter::Off => level_filters::LevelFilter::OFF,
            log::LevelFilter::Error => level_filters::LevelFilter::ERROR,
            log::LevelFilter::Warn => level_filters::LevelFilter::WARN,
            log::LevelFilter::Info => level_filters::LevelFilter::INFO,
            log::LevelFilter::Debug => level_filters::LevelFilter::DEBUG,
            log::LevelFilter::Trace => level_filters::LevelFilter::TRACE,
        })
        .init();

    info!(
        app_name = env!("CARGO_PKG_NAME"),
        app_version = env!("CARGO_PKG_VERSION"),
        "Starting application",
    );

    // Create a new VMM
    let mut vmm = VMM::new().map_err(Error::VmmNew)?;

    vmm.configure(args.cpus, args.memory, &args.kernel)
        .map_err(Error::VmmConfigure)?;

    // Run the VMM
    vmm.run().map_err(Error::VmmRun)?;

    Ok(())
}
