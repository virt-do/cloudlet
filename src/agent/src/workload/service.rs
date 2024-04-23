use crate::agent::{self, ExecuteRequest, ExecuteResponse, SignalRequest};

use agent::workload_runner_server::WorkloadRunner;

use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};

type Result<T> = std::result::Result<Response<T>, tonic::Status>;

pub struct WorkloadRunnerService {}

#[tonic::async_trait]
impl WorkloadRunner for WorkloadRunnerService {
    type ExecuteStream = ReceiverStream<std::result::Result<ExecuteResponse, tonic::Status>>;

    async fn execute(&self, _: Request<ExecuteRequest>) -> Result<Self::ExecuteStream> {
        unreachable!("Not implemented")
    }

    async fn signal(&self, _: Request<SignalRequest>) -> Result<()> {
        unreachable!("Not implemented")
    }
}
