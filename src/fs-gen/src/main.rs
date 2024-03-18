use clap::Parser;

use crate::cli_args::CliArgs;

mod cli_args;

fn main() {
    let args = CliArgs::parse();
    println!("Hello, world!, {:?}", args);
}
