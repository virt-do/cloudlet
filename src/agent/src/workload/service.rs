use super::runner::Runner;
use crate::agent::{self, ExecuteRequest, ExecuteResponse, SignalRequest};
use agent::workload_runner_server::WorkloadRunner;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::{process, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};

type Result<T> = std::result::Result<Response<T>, tonic::Status>;

static CHILD_PROCESSES: Lazy<Arc<Mutex<HashSet<u32>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));

pub struct WorkloadRunnerService;

#[tonic::async_trait]
impl WorkloadRunner for WorkloadRunnerService {
    type ExecuteStream = ReceiverStream<std::result::Result<ExecuteResponse, tonic::Status>>;

    async fn execute(&self, req: Request<ExecuteRequest>) -> Result<Self::ExecuteStream> {
        let runner = Runner::new_from_execute_request(req.into_inner(), CHILD_PROCESSES.clone())
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let mut runner_rx = runner
            .run()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let (tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            while let Some(agent_output) = runner_rx.recv().await {
                println!("Sending to the gRPC client: {:?}", agent_output);
                let _ = tx.send(Ok(agent_output.into())).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn signal(&self, _: Request<SignalRequest>) -> Result<()> {
        let child_processes = CHILD_PROCESSES.lock().await;

        for &child_id in child_processes.iter() {
            match nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(child_id as i32),
                nix::sys::signal::Signal::SIGTERM,
            ) {
                Ok(_) => println!("Sent SIGTERM to child process {}", child_id),
                Err(e) => println!(
                    "Failed to send SIGTERM to child process {}: {}",
                    child_id, e
                ),
            }
        }

        process::exit(0);
    }
}
