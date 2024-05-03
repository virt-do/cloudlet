pub mod core;
pub mod grpc {
    pub mod client;
    pub mod server;
}

#[derive(Debug)]
pub enum VmmErrors {
    VmmNew(core::Error),
    VmmConfigure(core::Error),
    VmmRun(core::Error),
    VmmBuildEnvironment(std::io::Error),
}
