use clap::Parser;

use crate::cli_args::CliArgs;

mod cli_args;

fn main() {
    let args = CliArgs::get_args();
    println!("Hello, world!, {:?}", args);
}
