use clap::Parser;

use args::{CliArgs, Commands};
use request::run_request;
use request::HttpRunRequest;
use std::io::{self};
use types::Config;
use utils::{load_config, read_file};

mod args;
mod request;
mod types;
mod utils;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = CliArgs::parse();

    match args.command {
        Commands::Run { config_path } => {
            let yaml_config: Config =
                load_config(&config_path).expect("Error while loading the configuration file");

            let code =
                read_file(&yaml_config.code_path).expect("Error while reading the code file");
            println!("Code from file: \n{}", code);

            let env =
                read_file(&yaml_config.env_path).expect("Error while reading the environment file");
            println!("Env from file : \n{}", env);
            println!("Configuration from YAML file: \n {:#?}", yaml_config);

            let body = HttpRunRequest::new(yaml_config.language, env, code, yaml_config.log_level);
            let response = run_request(body).await;

            match response {
                Ok(_) => println!("Request successful"),
                Err(e) => eprintln!("Error while making the request: {}", e),
            }
        }
    }

    Ok(())
}
