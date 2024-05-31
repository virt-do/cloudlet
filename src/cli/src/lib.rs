use api_client::ExecuteJsonResponse;
use args::{CliArgs, Commands};
use crossterm::style::Stylize;
use futures::TryStreamExt;
use reqwest_eventsource::Event;
use std::fmt::Display;

mod api_client;
pub mod args;
mod utils;

#[derive(Debug)]
pub enum Error {
    StdoutExecute(std::io::Error),
    ApiClient(api_client::Error),
    InvalidRequest(String),
    ProgramFailed,
}

pub async fn run_cli(base_url: &str, args: CliArgs) -> Result<i32, Error> {
    match args.command {
        Commands::Run { config_path } => {
            let body = api_client::new_cloudlet_request(&config_path).map_err(Error::ApiClient)?;
            let mut es = api_client::execute(base_url, body)
                .await
                .map_err(Error::ApiClient)?;

            let mut exit_code = 0;

            while let Ok(Some(event)) = es.try_next().await {
                match event {
                    Event::Open => { /* skip */ }
                    Event::Message(msg) => {
                        let exec_response = ExecuteJsonResponse::try_from(msg.data);
                        if let Ok(exec_response) = exec_response {
                            if let Some(stdout) = exec_response.stdout {
                                println!("{}", stylize(stdout, &exec_response.stage));
                            }
                            if let Some(stderr) = exec_response.stderr {
                                println!("{}", stylize(stderr, &exec_response.stage));
                            }
                            if let Some(code) = exec_response.exit_code {
                                exit_code = code;
                            }
                        }
                    }
                }
            }

            Ok(exit_code)
        }
        Commands::Shutdown {} => {
            api_client::shutdown(base_url)
                .await
                .map_err(Error::ApiClient)?;

            Ok(0)
        }
    }
}

fn stylize(output: String, stage: &api_client::Stage) -> impl Display {
    match stage {
        api_client::Stage::Building => output.yellow(),
        api_client::Stage::Failed => output.dark_red(),
        api_client::Stage::Debug => output.dark_blue(),
        _ => output.stylize(),
    }
}
