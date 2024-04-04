use clap::Parser;
mod types;
mod utils;
use std::io::{self};
use types::{Config, Language, LogLevel};
use utils::load_config;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    Configure {
        #[arg(short, long)]
        config_path: String,
    },
    Status {},
    Apply {},
    Kill {},
}

#[tokio::main]

async fn main() -> io::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Configure { config_path } => {
            let config: Config = load_config(&config_path).unwrap();

            println!("Configuration from YAML file:");
            println!(
                "Language: {}",
                match config.language {
                    Language::Rust => "Rust",
                    Language::Python => "Python",
                    Language::Node => "Node",
                }
            );
            println!("Env Path: {}", config.env_path);
            println!("Code Path: {}", config.code_path);
            println!(
                "Log Level: {}",
                match config.log_level {
                    LogLevel::Debug => "Debug",
                    LogLevel::Info => "Info",
                    LogLevel::Warn => "Warn",
                    LogLevel::Error => "Error",
                }
            );
        }

        Commands::Status {} => {
            println!("Getting status");
        }

        Commands::Apply {} => {
            println!("Applying configuration");
        }

        Commands::Kill {} => {
            println!("Killing configuration");
        }
    }

    Ok(())
}
