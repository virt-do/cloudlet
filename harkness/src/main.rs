use clap::Parser;
use harkness_vmm::VMM;
use tracing::{event, Level};
use tracing_log::AsTrace;

use crate::args::CliArguments;

mod args;

#[derive(Debug)]
pub enum Error {
    VmmNew(harkness_vmm::Error),
    VmmConfigure(harkness_vmm::Error),
    VmmRun(harkness_vmm::Error),
}

/// The application entry point.
fn main() -> Result<(), Error> {
    // Parse the configuration and configure logger verbosity
    let args = CliArguments::parse();

    tracing_subscriber::fmt()
        .with_max_level(args.verbose.log_level_filter().as_trace())
        .init();

    event!(
        Level::INFO,
        app_name = env!("CARGO_PKG_NAME"),
        app_version = env!("CARGO_PKG_VERSION"),
        "Starting application"
    );

    // Create a new VMM
    let mut vmm = VMM::new().map_err(Error::VmmNew)?;

    // Configure the VMM:
    // * Number of virtual CPUs
    // * Memory size (in MB)
    // * Path to a Linux kernel
    vmm.configure(args.cpus, args.memory, &args.kernel)
        .map_err(Error::VmmConfigure)?;

    // Run the VMM
    vmm.run().map_err(Error::VmmRun)?;

    Ok(())
}
