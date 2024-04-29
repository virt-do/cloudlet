use crate::grpc::client::agent::ExecuteRequest;
use self::vmmorchestrator::{
    vmm_service_server::VmmService as VmmServiceTrait, Language, RunVmmRequest
};
use crate::VmmErrors;
use crate::{core::vmm::VMM, grpc::client::WorkloadClient};
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

    async fn run(&self, request: Request<RunVmmRequest>) -> Result<Self::RunStream> {
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        const HOST_IP: Ipv4Addr = Ipv4Addr::new(172, 29, 0, 1);
        const HOST_NETMASK: Ipv4Addr = Ipv4Addr::new(255, 255, 0, 0);
        const GUEST_IP: Ipv4Addr = Ipv4Addr::new(172, 29, 0, 2);

        // get current directory
        let mut curr_dir =
            current_dir().expect("Need to be able to access current directory path.");

        // define kernel path
        let mut kernel_entire_path = curr_dir.as_os_str().to_owned();
        kernel_entire_path
            .push("/tools/kernel/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin");

        // Check if the kernel is on the system, else build it
        let kernel_exists = Path::new(&kernel_entire_path).try_exists().expect(&format!(
            "Could not access folder {:?}",
            &kernel_entire_path
        ));

        if !kernel_exists {
            info!("Kernel not found, building kernel");
            // Execute the script using sh and capture output and error streams
            let output = Command::new("sh")
                .arg("./tools/kernel/mkkernel.sh")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Failed to execute the kernel build script");

            // Print output and error streams
            info!("Script output: {}", String::from_utf8_lossy(&output.stdout));
            error!("Script errors: {}", String::from_utf8_lossy(&output.stderr));
        };
        let kernel_path = Path::new(&kernel_entire_path);

        // define initramfs file placement
        let mut initramfs_entire_file_path = curr_dir.as_os_str().to_owned();
        initramfs_entire_file_path.push("/tools/rootfs/");

        // get request with the language
        let req: RunVmmRequest = request.into_inner();
        let language: Language = Language::from_i32(req.language).expect("Unknown language");

        let image = match language {
            Language::Rust => {
                initramfs_entire_file_path.push("rust.img");
                "rust:alpine"
            }
            Language::Python => {
                initramfs_entire_file_path.push("python.img");
                "python:alpine"
            }
            Language::Node => {
                initramfs_entire_file_path.push("node.img");
                "node:alpine"
            }
        };

        let rootfs_exists = Path::new(&initramfs_entire_file_path)
            .try_exists()
            .expect(&format!(
                "Could not access folder {:?}",
                &initramfs_entire_file_path
            ));
        if !rootfs_exists {
            // check if agent binary exists
            let agent_file_name = curr_dir.as_mut_os_string();
            agent_file_name.push("/target/x86_64-unknown-linux-musl/release/agent");

            // if agent hasn't been build, build it
            let agent_exists = Path::new(&agent_file_name)
                .try_exists()
                .expect(&format!("Could not access folder {:?}", &agent_file_name));
            if !agent_exists {
                //build agent
                info!("Building agent binary");
                // Execute the script using sh and capture output and error streams
                let output = Command::new("just")
                    .arg("build-musl-agent")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .expect("Failed to execute the just build script for the agent");

                // Print output and error streams
                info!("Script output: {}", String::from_utf8_lossy(&output.stdout));
                error!("Script errors: {}", String::from_utf8_lossy(&output.stderr));
                info!("Agent binary successfully built.")
            }

            info!("Building initramfs");
            // Execute the script using sh and capture output and error streams
            let output = Command::new("sh")
                .arg("./tools/rootfs/mkrootfs.sh")
                .arg(image)
                .arg(&agent_file_name)
                .arg(&initramfs_entire_file_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Failed to execute the initramfs build script");

            // Print output and error streams
            info!("Script output: {}", String::from_utf8_lossy(&output.stdout));
            error!("Script errors: {}", String::from_utf8_lossy(&output.stderr));
            info!("Initramfs successfully built.")
        }
        let initramfs_path = PathBuf::from(&initramfs_entire_file_path);

        let mut vmm = VMM::new(HOST_IP, HOST_NETMASK, GUEST_IP).map_err(VmmErrors::VmmNew)?;

        // Configure the VMM parameters might need to be calculated rather than hardcoded
        vmm.configure(1, 4000, kernel_path, &Some(initramfs_path))
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
            tokio::time::sleep(Duration::from_secs(2)).await;
            println!("Connecting to Agent service");

            WorkloadClient::new(GUEST_IP, 50051).await
        })
        .await
        .unwrap();

        // Send the grpc request to start the agent
        let vmm_request = request.into_inner();
        let agent_request = ExecuteRequest {
            workload_name: vmm_request.workload_name,
            language: match vmm_request.language {
                0 => "python".to_string(),
                1 => "node".to_string(),
                2 => "rust".to_string(),
                _ => unreachable!("Invalid language"),
            },
            action: 2, // Prepare and run
            code: vmm_request.code,
            config_str: "[build]\nrelease = true".to_string(),
        };

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
