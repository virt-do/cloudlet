use clap::Parser;

use crate::types::YamlConfigFile;
use args::{CliArgs, Commands};

use services::HttpVmmRequest;
use std::io::{self};
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
            let yaml_config: YamlConfigFile =
                load_config(&config_path).expect("Error while loading the configuration file");
            let body = HttpVmmRequest::new(yaml_config);
            let response = HttpVmmRequest::post(body).await;

            match response {
                Ok(_) => println!("Request successful"),
                Err(e) => eprintln!("Error while making the request: {}", e),
            }
        }
    }

    Ok(())
}
