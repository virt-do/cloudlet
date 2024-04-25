use std::{process, sync::Arc};

use crate::agent::{self, ExecuteRequest, ExecuteResponse, SignalRequest};

use agent::workload_runner_server::WorkloadRunner;

use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};

use super::runner::Runner;

type Result<T> = std::result::Result<Response<T>, tonic::Status>;

pub struct WorkloadRunnerService {
    runner: Arc<Mutex<Runner>>,
}

impl WorkloadRunnerService {
    pub fn new(runner: Runner) -> Self {
        WorkloadRunnerService {
            runner: Arc::new(Mutex::new(runner)),
        }
    }
}

#[tonic::async_trait]
impl WorkloadRunner for WorkloadRunnerService {
    type ExecuteStream = ReceiverStream<std::result::Result<ExecuteResponse, tonic::Status>>;

    async fn execute(&self, _: Request<ExecuteRequest>) -> Result<Self::ExecuteStream> {
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        // We assume there's only one request at a time
        let runner = match self.runner.try_lock() {
            Ok(runner) => runner,
            Err(_) => {
                return Err(tonic::Status::unavailable("Runner is busy"));
            }
        };

        let res = match runner.run() {
            Ok(res) => res,
            Err(err) => {
                return Err(tonic::Status::internal(err.to_string()));
            }
        };

        let _ = tx
            .send(Ok(ExecuteResponse {
                stdout: res.stdout,
                stderr: res.stderr,
                exit_code: res.exit_code,
            }))
            .await
            .map_err(|e| {
                println!("Failed to send response: {:?}", e);
                tonic::Status::internal("Failed to send response")
            })?;

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn signal(&self, _: Request<SignalRequest>) -> Result<()> {
        process::exit(0)
    }
}
