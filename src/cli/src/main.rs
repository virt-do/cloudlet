use clap::Parser;
use cli::args::CliArgs;
use std::process::exit;

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    let api_url = std::env::var("API_URL").unwrap_or("localhost:3000".into());
    let api_url = format!("http://{api_url}");

    let result = cli::run_cli(&api_url, args).await;
    match result {
        Ok(exit_code) => exit(exit_code),
        Err(e) => {
            eprintln!("Could not execute the command:\n{:?}", e);
            exit(1);
        }
    }
}
