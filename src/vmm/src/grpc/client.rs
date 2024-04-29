use self::agent::{workload_runner_client::WorkloadRunnerClient, ExecuteRequest};
use log::error;
use std::{net::Ipv4Addr, time::Duration};
use tonic::{transport::Channel, Streaming};

pub mod agent {
    tonic::include_proto!("cloudlet.agent");
}

pub struct WorkloadClient {
    client: WorkloadRunnerClient<Channel>,
}

impl WorkloadClient {
    pub async fn new(_ip: Ipv4Addr, _port: u16) -> Result<Self, tonic::transport::Error> {
        let delay = Duration::from_secs(2); // Setting initial delay to 2 seconds
        loop {
            match WorkloadRunnerClient::connect("http://[172.30.0.11]:50051".to_string()).await {
                Ok(client) => {
                    return Ok(WorkloadClient { client });
                }
                Err(err) => {
                    error!("Failed to connect to Agent service: {}", err);
                    error!("Retrying in {:?}...", delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    pub async fn execute(
        &mut self,
        request: ExecuteRequest,
    ) -> Result<Streaming<agent::ExecuteResponse>, tonic::Status> {
        let request = tonic::Request::new(request);
        let response_stream = self.client.execute(request).await?.into_inner();

        Ok(response_stream)
    }
}
