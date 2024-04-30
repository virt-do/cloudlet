use agent::{
    agent::workload_runner_server::WorkloadRunnerServer, workload::service::WorkloadRunnerService,
};
use clap::Parser;
use std::net::ToSocketAddrs;
use tonic::transport::Server;

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, env, default_value = "localhost")]
    grpc_server_address: String,
    #[clap(long, env, default_value = "50051")]
    grpc_server_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let bind_address = format!("{}:{}", args.grpc_server_address, args.grpc_server_port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let server = WorkloadRunnerService;

    Server::builder()
        .add_service(WorkloadRunnerServer::new(server))
        .serve(bind_address)
        .await
        .unwrap();

    Ok(())
}
