use self::vmmorchestrator::{
    vmm_service_server::VmmService as VmmServiceTrait, Language, RunVmmRequest, ShutdownVmRequest,
    ShutdownVmResponse,
};
use crate::grpc::client::agent::ExecuteRequest;
use crate::VmmErrors;
use crate::{core::vmm::VMM, grpc::client::WorkloadClient};
use std::ffi::OsStr;
use std::time::Duration;
use std::{
    convert::From,
    env::current_dir,
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
        // log the gRPC error before sending it
        error!("VMM error: {:?}", error);

        // You can create a custom Status variant based on the error
        match error {
            VmmErrors::VmmNew(_) => Status::internal("Error creating VMM"),
            VmmErrors::VmmConfigure(_) => Status::internal("Error configuring VMM"),
            VmmErrors::VmmRun(_) => Status::internal("Error running VMM"),
            VmmErrors::VmmBuildEnvironment(_) => {
                Status::internal("Error while compiling the necessary files for the VMM")
            }
        }
    }
}

#[derive(Default)]
pub struct VmmService;

impl VmmService {
    pub fn get_initramfs(
        &self,
        language: &str,
        curr_dir: &OsStr,
    ) -> std::result::Result<PathBuf, VmmErrors> {
        // define initramfs file placement
        let mut initramfs_entire_file_path = curr_dir.to_os_string();
        initramfs_entire_file_path.push(&format!("/tools/rootfs/{language}.img"));
        // set image name
        let image = format!("{language}:alpine");

        // check if an initramfs already exists
        let rootfs_exists = Path::new(&initramfs_entire_file_path)
            .try_exists()
            .map_err(VmmErrors::VmmBuildEnvironment)?;
        if !rootfs_exists {
            // build the agent
            let agent_file_name = self.get_path(
                curr_dir,
                "/target/x86_64-unknown-linux-musl/release/agent",
                "cargo",
                vec![
                    "build",
                    "--release",
                    "--bin",
                    "agent",
                    "--target=x86_64-unknown-linux-musl",
                ],
            )?;
            // build initramfs
            info!("Building initramfs");
            let _ = self
                .run_command(
                    "sh",
                    vec![
                        "./tools/rootfs/mkrootfs.sh",
                        &image,
                        &agent_file_name.to_str().unwrap(),
                        &initramfs_entire_file_path.to_str().unwrap(),
                    ],
                )
                .map_err(VmmErrors::VmmBuildEnvironment);
        }
        Ok(PathBuf::from(&initramfs_entire_file_path))
    }

    pub fn run_command(
        &self,
        command_type: &str,
        args: Vec<&str>,
    ) -> std::result::Result<(), std::io::Error> {
        // Execute the script using sh and capture output and error streams
        Command::new(command_type)
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to execute the script");
        Ok(())
    }

    pub fn get_path(
        &self,
        curr_dir: &OsStr,
        end_path: &str,
        command_type: &str,
        args: Vec<&str>,
    ) -> std::result::Result<PathBuf, VmmErrors> {
        // define file path
        let mut entire_path = curr_dir.to_os_string();
        entire_path.push(end_path);

        // Check if the file is on the system, else build it
        let exists = Path::new(&entire_path)
            .try_exists()
            .map_err(VmmErrors::VmmBuildEnvironment)?;

        if !exists {
            info!("File {:?} not found, building it", &entire_path);
            let _ = self
                .run_command(command_type, args)
                .map_err(VmmErrors::VmmBuildEnvironment);
            info!("File {:?} successfully build", &entire_path);
        };
        Ok(PathBuf::from(&entire_path))
    }

    pub fn get_agent_request(
        &self,
        vmm_request: RunVmmRequest,
        language: String,
    ) -> ExecuteRequest {
        // Send the grpc request to start the agent
        ExecuteRequest {
            workload_name: vmm_request.workload_name,
            language,
            action: 2, // Prepare and run
            code: vmm_request.code,
            config_str: "[build]\nrelease = true".to_string(),
        }
    }
}

#[tonic::async_trait]
impl VmmServiceTrait for VmmService {
    type RunStream =
        ReceiverStream<std::result::Result<vmmorchestrator::ExecuteResponse, tonic::Status>>;

    async fn shutdown(&self, request: Request<ShutdownVmRequest>) -> Result<ShutdownVmResponse> {
        const GUEST_IP: Ipv4Addr = Ipv4Addr::new(172, 29, 0, 2);

        let grpc_client = tokio::spawn(async move {
            // Wait 2 seconds
            tokio::time::sleep(Duration::from_secs(2)).await;
            println!("Connecting to Agent service");

            WorkloadClient::new(GUEST_IP, 50051).await
        })
        .await
        .unwrap();

        if let Ok(mut client) = grpc_client {
            info!("Attempting to shutdown the VM...");

            let response = client.shutdown(request.into_inner()).await.unwrap();

            return Ok(Response::new(response));
        } else if let Err(e) = grpc_client {
            error!("ERROR {:?}", e);
        }
        return Err(Status::internal("Failed to shutdown the VM"));
    }

    async fn run(&self, request: Request<RunVmmRequest>) -> Result<Self::RunStream> {
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        const HOST_IP: Ipv4Addr = Ipv4Addr::new(172, 29, 0, 1);
        const HOST_NETMASK: Ipv4Addr = Ipv4Addr::new(255, 255, 0, 0);
        const GUEST_IP: Ipv4Addr = Ipv4Addr::new(172, 29, 0, 2);

        // get current directory
        let curr_dir = current_dir()
            .map_err(VmmErrors::VmmBuildEnvironment)?
            .into_os_string();

        // build kernel if necessary
        let kernel_path: PathBuf = self.get_path(
            &curr_dir,
            "/tools/kernel/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin",
            "sh",
            vec!["./tools/kernel/mkkernel.sh"],
        )?;

        // get request with the language
        let vmm_request = request.into_inner();
        let language: String = Language::from_i32(vmm_request.language)
            .expect("Unknown language")
            .as_str_name()
            .to_lowercase();

        let initramfs_path = self.get_initramfs(&language, curr_dir.as_os_str())?;

        let mut vmm = VMM::new(HOST_IP, HOST_NETMASK, GUEST_IP).map_err(VmmErrors::VmmNew)?;

        // Configure the VMM parameters might need to be calculated rather than hardcoded
        vmm.configure(1, 4000, kernel_path, &Some(initramfs_path))
            .await
            .map_err(VmmErrors::VmmConfigure)?;

        // Run the VMM in a separate task
        tokio::spawn(async move {
            info!("Running VMM");
            if let Err(err) = vmm.run().map_err(VmmErrors::VmmRun) {
                error!("Error running VMM: {:?}", err);
            }
        });

        // run the grpc client
        let grpc_client = tokio::spawn(async move {
            // Wait 2 seconds
            tokio::time::sleep(Duration::from_secs(2)).await;
            info!("Connecting to Agent service");

            WorkloadClient::new(GUEST_IP, 50051).await
        })
        .await
        .unwrap();

        let agent_request = self.get_agent_request(vmm_request, language);

        match grpc_client {
            Ok(mut client) => {
                info!("Successfully connected to Agent service");

                // Start the execution
                let mut response_stream = client.execute(agent_request).await?;

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
