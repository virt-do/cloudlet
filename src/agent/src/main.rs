use agent::AgentRunner;
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = "/etc/cloudlet/agent/config.toml")]
    config: String,
}

fn main() {
    let args = Args::parse();

    let agent_runner = AgentRunner::new(args.config);

    agent_runner.run().unwrap();
}
