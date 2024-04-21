use tonic::{transport::Server, Request, Response, Status};
use crate::core::vmm::{self, VMM};
use std::{convert::From, net::Ipv4Addr, path::{Path, PathBuf}, process::{Command, Stdio}};
use crate::VmmErrors;
use self::vmmorchestrator::{vmm_service_server::VmmService as VmmServiceTrait, RunVmmRequest, RunVmmResponse};
use tracing::info;

pub mod vmmorchestrator {
    tonic::include_proto!("vmmorchestrator");
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
    async fn run(
        &self,
        request: Request<RunVmmRequest>,
    ) -> Result<Response<RunVmmResponse>, Status> {
        let response = vmmorchestrator::RunVmmResponse {};

        const HOST_IP: Ipv4Addr = Ipv4Addr::new(172, 29, 0, 1);
        const HOST_NETMASK: Ipv4Addr = Ipv4Addr::new(255, 255, 0, 0);
        
        // Check if the kernel is on the system, else build it
        println!("BEFORE");

        if !Path::new("./tools/kernel/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin").exists() {
            info!("Kernel not found, building kernel");
            // Execute the script using sh and capture output and error streams
            let output = Command::new("sh")
                .arg("./tools/kernel/mkkernel.sh")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Failed to execute the kernel build script");

            // Print output and error streams
            println!("Script output: {}", String::from_utf8_lossy(&output.stdout));
            println!("Script errors: {}", String::from_utf8_lossy(&output.stderr));

        };

        let kernel_path = &Path::new("./tools/kernel/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin");
        let mut initramfs_path: PathBuf = PathBuf::new(); 

        // Todo - Check if the initramfs for the specified language is on the system, else build it
        initramfs_path.push("./initramfs.img"); // Append the path to the PathBuf

        println!("Kernel path: {:?}", kernel_path);

        // // Create a new VMM
        let mut vmm =
            VMM::new(HOST_IP, HOST_NETMASK).map_err(VmmErrors::VmmNew)?;

        vmm.configure(1, 512, &kernel_path, &Some(initramfs_path))
            .map_err(VmmErrors::VmmConfigure)?;
        // Run the VMM
        vmm.run().map_err(VmmErrors::VmmRun)?;

        Ok(Response::new(response))
    }
}
