use clap::Parser;
mod types;
mod utils;
use std::{
    io::{self},
    path::PathBuf,
};
use types::{Config, Language, LogLevel};
use utils::{load_config, read_file};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    Run {
        #[arg(short, long)]
        config_path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

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
            println!("Configuration from YAML file:");
            println!(
                "Language: {}",
                match yaml_config.language {
                    Language::Rust => "Rust",
                    Language::Python => "Python",
                    Language::Node => "Node",
                }
            );
            println!("Env Path: {}", yaml_config.env_path);
            println!(
                "Log Level: {}",
                match yaml_config.log_level {
                    LogLevel::Debug => "Debug",
                    LogLevel::Info => "Info",
                    LogLevel::Warn => "Warn",
                    LogLevel::Error => "Error",
                }
            );
            println!("Code Path: {}", yaml_config.code_path);
        }
    }

    Ok(())
}
