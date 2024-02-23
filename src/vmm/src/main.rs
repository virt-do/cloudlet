use crate::args::CliArguments;
use clap::Parser;
use tracing::{info, Level};
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
        .with_max_level(Level::DEBUG)
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
