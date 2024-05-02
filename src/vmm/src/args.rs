//! Command-line arguments.
use std::{net::Ipv4Addr, path::PathBuf};

use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use tracing::level_filters;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    #[command(about = "Run a VMM instance.")]
    Cli(CliArguments),
    #[command(about = "Run a GRPC server listening for incoming requests.")]
    Grpc,
}

/// Run a VMM instance.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArguments {
    /// Path to the image of the Linux kernel to boot.
    #[arg(short, long, env, required = true)]
    pub kernel: PathBuf,

    /// Path to the cpio archive to use as the initramfs.
    #[arg(short, long, env, required = true)]
    pub initramfs: Option<PathBuf>,

    /// Number of virtual CPUs assigned to the guest.
    #[clap(short, long, env, default_value = "1")]
    pub cpus: u8,

    /// Memory amount (in MBytes) assigned to the guest.
    #[clap(short, long, env, default_value = "512")]
    pub memory: u32,

    /// IPv4 address of the host tap interface.
    #[clap(long, env, required = true)]
    pub iface_host_addr: Ipv4Addr,

    /// Network.
    #[clap(long, env, required = true)]
    pub network: Ipv4Addr,

    /// Subnet mask for network.
    #[clap(long, env, required = true)]
    pub netmask: Ipv4Addr,

    /// IPv4 address of the guest eth0 interface.
    #[clap(long, env, required = true)]
    pub iface_guest_addr: Ipv4Addr,

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
