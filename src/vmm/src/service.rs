use tonic::{transport::Server, Request, Response, Status};
use crate::core::vmm::{self, VMM};
use std::{convert::From, net::Ipv4Addr, path::{Path, PathBuf}};
use crate::VmmErrors;
use self::vmmorchestrator::{vmm_service_server::VmmService as VmmServiceTrait, RunVmmRequest, RunVmmResponse};

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

        const host_ip: Ipv4Addr = Ipv4Addr::new(172, 29, 0, 1);
        const host_netmask: Ipv4Addr = Ipv4Addr::new(255, 255, 0, 0);
        // Check if the kernel is on the system, else download it

        let kernel_path: &Path = &Path::new("../../../initramfs.img");
        let initramfs_path: PathBuf = PathBuf::new(); // Create an owned PathBuf
        // // Create a new VMM
        let mut vmm =
            VMM::new(host_ip, host_netmask).map_err(VmmErrors::VmmNew)?;

        vmm.configure(1, 512, &kernel_path, &Some(initramfs_path))
            .map_err(VmmErrors::VmmConfigure)?;
        // Run the VMM
        vmm.run().map_err(VmmErrors::VmmRun)?;

        Ok(Response::new(response))
    }
}
