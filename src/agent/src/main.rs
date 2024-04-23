use agent::{
    agent::workload_runner_server::WorkloadRunnerServer,
    workload::{config::Config, runner::Runner, service::WorkloadRunnerService},
};
use clap::Parser;
use std::{net::ToSocketAddrs, path::PathBuf};
use tonic::transport::Server;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = "/etc/cloudlet/agent/config.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = Config::from_file(&args.config).unwrap();

    let bind_address = format!("{}:{}", config.server.address, config.server.port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let runner = Runner::new(config);

    let server = WorkloadRunnerService::new(runner);

    Server::builder()
        .add_service(WorkloadRunnerServer::new(server))
        .serve(bind_address)
        .await
        .unwrap();

    Ok(())
}
