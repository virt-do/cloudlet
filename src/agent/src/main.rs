use agent::workload::{config::Config, runner::Runner};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = "/etc/cloudlet/agent/config.toml")]
    config: PathBuf,
}

fn main() {
    let args = Args::parse();

    let config = Config::from_file(&args.config).unwrap();
    let runner = Runner::new(config);

    runner.run().unwrap();
}
