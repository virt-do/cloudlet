use self::vmmorchestrator::{vmm_service_server::VmmService as VmmServiceTrait, RunVmmRequest};
use crate::grpc::client::agent::ExecuteRequest;
use crate::VmmErrors;
use crate::{core::vmm::VMM, grpc::client::WorkloadClient};
use std::time::Duration;
use std::{
    convert::From,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info};

type Result<T> = std::result::Result<Response<T>, tonic::Status>;

pub mod vmmorchestrator {
    tonic::include_proto!("vmmorchestrator");
}

pub mod agent {
    tonic::include_proto!("cloudlet.agent");
}

// Implement the From trait for VmmErrors into Status
impl From<VmmErrors> for Status {
    fn from(error: VmmErrors) -> Self {
        // You can create a custom Status variant based on the error
        match error {
            VmmErrors::VmmNew(_) => Status::internal("Error creating VMM"),
            VmmErrors::VmmConfigure(_) => Status::internal("Error configuring VMM"),
            VmmErrors::VmmRun(_) => Status::internal("Error running VMM"),
        }
    }
}

#[derive(Default)]
pub struct VmmService;

#[tonic::async_trait]
impl VmmServiceTrait for VmmService {
    type RunStream =
        ReceiverStream<std::result::Result<vmmorchestrator::ExecuteResponse, tonic::Status>>;

    async fn run(&self, _request: Request<RunVmmRequest>) -> Result<Self::RunStream> {
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        const HOST_IP: Ipv4Addr = Ipv4Addr::new(172, 29, 0, 1);
        const VM_IP: Ipv4Addr = Ipv4Addr::new(172, 30, 0, 11);
        const HOST_NETMASK: Ipv4Addr = Ipv4Addr::new(255, 255, 0, 0);

        // Check if the kernel is on the system, else build it
        if !Path::new("./tools/kernel/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin")
            .exists()
        {
            info!("Kernel not found, building kernel");
            // Execute the script using sh and capture output and error streams
            let output = Command::new("sh")
                .arg("./tools/kernel/mkkernel.sh")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Failed to execute the kernel build script");

            // Print output and error streams
            error!("Script output: {}", String::from_utf8_lossy(&output.stdout));
            error!("Script errors: {}", String::from_utf8_lossy(&output.stderr));
        };

        let kernel_path = &Path::new(
            "./tools/kernel/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin",
        );
        let mut initramfs_path: PathBuf = PathBuf::new();

        // Todo - Check if the initramfs for the specified language is on the system, else build it
        initramfs_path.push("./tools/rootfs/initramfs.img");

        // // Create a new VMM
        let mut vmm = VMM::new(HOST_IP, HOST_NETMASK).map_err(VmmErrors::VmmNew)?;

        // Configure the VMM parameters might need to be calculated rather than hardcoded
        vmm.configure(1, 512, kernel_path, &Some(initramfs_path))
            .map_err(VmmErrors::VmmConfigure)?;

        // Run the VMM in a separate task
        tokio::spawn(async move {
            info!("Running VMM");
            if let Err(err) = vmm.run().map_err(VmmErrors::VmmRun) {
                error!("Error running VMM: {:?}", err);
            }
        });

        let grpc_client = tokio::spawn(async move {
            // Wait 2 seconds
            tokio::time::sleep(Duration::from_secs(1)).await;
            info!("Connecting to Agent service");

            // Set up the IP address
            

            WorkloadClient::new(VM_IP, 50051).await
        })
        .await
        .unwrap();

        // Send the grpc request to start the agent

        let execute_request = ExecuteRequest {};

        match grpc_client {
            Ok(mut client) => {
                info!("Successfully connected to Agent service");

                // Start the execution
                let mut response_stream = client.execute(execute_request).await?;

                // Process each message as it arrives
                while let Some(response) = response_stream.message().await? {
                    let vmm_response = vmmorchestrator::ExecuteResponse {
                        stdout: response.stdout,
                        stderr: response.stderr,
                        exit_code: response.exit_code,
                    };
                    tx.send(Ok(vmm_response)).await.unwrap();
                }
            }
            Err(e) => {
                error!("ERROR {:?}", e);
            }
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
