//! Command-line arguments.
use std::path::PathBuf;

use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

// The Virtual Machine Manager for the Harkness serverless runtime.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArguments {
    /// Path to the image of the Linux kernel to boot.
    #[arg(short, long, env)]
    pub kernel: PathBuf,

    /// Number of virtual CPUs assigned to the guest.
    #[clap(short, long, env, default_value = "1")]
    pub cpus: u8,

    /// Memory amount (in MBytes) assigned to the guest.
    #[clap(short, long, env, default_value = "512")]
    pub memory: u32,

    /// Verbosity level.
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}
