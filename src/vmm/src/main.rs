use crate::args::{CliArgs, Commands};
use clap::Parser;
use tonic::transport::Server;
use tracing::info;
use vmm::{
    core::vmm::VMM,
    grpc::server::{vmmorchestrator, VmmService},
    VmmErrors,
};
mod args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the configuration and configure logger verbosity
    let args = CliArgs::parse();

    info!(
        app_name = env!("CARGO_PKG_NAME"),
        app_version = env!("CARGO_PKG_VERSION"),
        "Starting application",
    );

    let addr = "[::1]:50051".parse().unwrap();
    let vmm_service = VmmService;

    // check if the args is grpc or command
    match args.command {
        Commands::Grpc => {
            tracing_subscriber::fmt().init();
            Server::builder()
                .add_service(vmmorchestrator::vmm_service_server::VmmServiceServer::new(
                    vmm_service,
                ))
                .serve(addr)
                .await?;
        }
        Commands::Cli(cli_args) => {
            tracing_subscriber::fmt()
                .with_max_level(cli_args.convert_log_to_tracing())
                .init();

            // Create a new VMM
            let mut vmm = VMM::new(
                cli_args.iface_host_addr,
                cli_args.netmask,
                cli_args.iface_guest_addr,
            )
            .await
            .map_err(VmmErrors::VmmNew)
            .unwrap();

            vmm.configure(
                cli_args.cpus,
                cli_args.memory,
                cli_args.kernel,
                &cli_args.initramfs,
            )
            .await
            .map_err(VmmErrors::VmmConfigure)
            .unwrap();

            // Run the VMM
            vmm.run().map_err(VmmErrors::VmmRun).unwrap();
        }
    }

    Ok(())
}
