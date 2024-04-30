use clap::Parser;

use args::{CliArgs, Commands};

use services::CloudletClient;
use std::{fs, io, process::exit};

mod args;
mod services;
mod utils;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = CliArgs::parse();

    match args.command {
        Commands::Run { config_path } => {
            let toml_file = match fs::read_to_string(config_path.clone()) {
                Ok(c) => c,
                Err(_) => {
                    eprintln!("Could not read file `{:?}`", config_path);
                    exit(1);
                }
            };
            let body = CloudletClient::new_cloudlet_config(toml_file);
            let response = CloudletClient::run(body).await;

            match response {
                Ok(_) => println!("Request successful {:?}", response),
                Err(e) => eprintln!("Error while making the request: {}", e),
            }
        }
    }

    Ok(())
}
