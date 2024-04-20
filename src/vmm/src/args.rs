//! Command-line arguments.
use std::{net::Ipv4Addr, path::PathBuf};

use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use tracing::level_filters;

/// The Virtual Machine Manager for the Cloudlet serverless runtime.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArguments {
    /// Path to the image of the Linux kernel to boot.
    #[arg(short, long, env)]
    pub kernel: PathBuf,

    /// Path to the cpio archive to use as the initramfs.
    #[arg(short, long, env)]
    pub initramfs: Option<PathBuf>,

    /// Number of virtual CPUs assigned to the guest.
    #[clap(short, long, env, default_value = "1")]
    pub cpus: u8,

    /// Memory amount (in MBytes) assigned to the guest.
    #[clap(short, long, env, default_value = "512")]
    pub memory: u32,

    /// IPv4 address of the host tap interface.
    #[clap(long, env)]
    pub network_host_ip: Ipv4Addr,

    /// Subnet mask of the host tap interface.
    #[clap(long, env)]
    pub network_host_netmask: Ipv4Addr,

    /// Verbosity level.
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}

impl CliArguments {
    /// Get the log level filter.
    pub fn convert_log_to_tracing(&self) -> level_filters::LevelFilter {
        match self.verbose.log_level_filter() {
            log::LevelFilter::Off => level_filters::LevelFilter::OFF,
            log::LevelFilter::Error => level_filters::LevelFilter::ERROR,
            log::LevelFilter::Warn => level_filters::LevelFilter::WARN,
            log::LevelFilter::Info => level_filters::LevelFilter::INFO,
            log::LevelFilter::Debug => level_filters::LevelFilter::DEBUG,
            log::LevelFilter::Trace => level_filters::LevelFilter::TRACE,
        }
    }
}
