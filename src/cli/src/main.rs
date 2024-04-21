use clap::Parser;

use args::{CliArgs, Commands};
use shared_models::YamlClientConfigFile;

use services::CloudletClient;
use std::io::{self};
use utils::ConfigFileHandler;

mod args;
mod services;
mod utils;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = CliArgs::parse();

    match args.command {
        Commands::Run { config_path } => {
            let yaml_config: YamlClientConfigFile = ConfigFileHandler::load_config(&config_path)
                .expect("Error while loading the configuration file");
            let body = CloudletClient::new_cloudlet_config(yaml_config);
            let response = CloudletClient::run(body).await;

            match response {
                Ok(_) => println!("Request successful"),
                Err(e) => eprintln!("Error while making the request: {}", e),
            }
        }
    }

    Ok(())
}
