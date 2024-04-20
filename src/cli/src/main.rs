use clap::Parser;

use args::{CliArgs, Commands};
use services::run_request;
use services::HttpRunRequest;
use std::io::{self};
use types::Config;
use utils::load_config;

mod args;
mod services;
mod types;
mod utils;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = CliArgs::parse();

    match args.command {
        Commands::Run { config_path } => {
            let yaml_config: Config =
                load_config(&config_path).expect("Error while loading the configuration file");
            let body = HttpRunRequest::new(yaml_config);
            let response = run_request(body).await;

            match response {
                Ok(_) => println!("Request successful"),
                Err(e) => eprintln!("Error while making the request: {}", e),
            }
        }
    }

    Ok(())
}
