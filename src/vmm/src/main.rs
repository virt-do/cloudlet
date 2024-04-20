use crate::args::CliArguments;
use clap::Parser;
use tracing::info;
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
        .with_max_level(args.convert_log_to_tracing())
        .init();

    info!(
        app_name = env!("CARGO_PKG_NAME"),
        app_version = env!("CARGO_PKG_VERSION"),
        "Starting application",
    );

    // Create a new VMM
    let mut vmm =
        VMM::new(args.network_host_ip, args.network_host_netmask).map_err(Error::VmmNew)?;

    vmm.configure(args.cpus, args.memory, &args.kernel, &args.initramfs)
        .map_err(Error::VmmConfigure)?;

    // Run the VMM
    vmm.run().map_err(Error::VmmRun)?;

    Ok(())
}
