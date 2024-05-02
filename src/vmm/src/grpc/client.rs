use self::agent::{workload_runner_client::WorkloadRunnerClient, ExecuteRequest, SignalRequest};
use log::error;
use std::{error::Error, net::Ipv4Addr, time::Duration};
use tonic::{transport::Channel, IntoRequest, Streaming};
use super::server::vmmorchestrator::{ShutdownVmRequest, ShutdownVmResponse};

pub mod agent {
    tonic::include_proto!("cloudlet.agent");
}

pub struct WorkloadClient {
    client: WorkloadRunnerClient<Channel>,
}

impl WorkloadClient {
    pub async fn new(ip: Ipv4Addr, port: u16) -> Result<Self, tonic::transport::Error> {
        let delay = Duration::from_secs(2); // Setting initial delay to 2 seconds
        loop {
            match WorkloadRunnerClient::connect(format!("http://[{}]:{}", ip, port)).await {
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

    pub async fn shutdown(
        &mut self,
        _request: ShutdownVmRequest,
    ) -> Result<ShutdownVmResponse, tonic::Status> {
        const BROKEN_PIPE_ERROR: &str = "stream closed because of a broken pipe";

        let signal_request = SignalRequest::default();
        let response = self.client.signal(signal_request).await;

        if let Err(status) = response {
            let error = status.source().unwrap().source().unwrap().source().unwrap();
            if error.to_string().as_str().eq(BROKEN_PIPE_ERROR) {
                return Ok(ShutdownVmResponse {
                    success: true
                });
            }
        }

        Ok(ShutdownVmResponse {
            success: false
        })
    }
}
