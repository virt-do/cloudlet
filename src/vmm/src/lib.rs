pub mod core;
pub mod service;

#[derive(Debug)]
pub enum VmmErrors {
    VmmNew(core::Error),
    VmmConfigure(core::Error),
    VmmRun(core::Error),
}
