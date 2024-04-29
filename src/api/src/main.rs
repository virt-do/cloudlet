use actix_web::{App, HttpServer};
use api::services::{run, shutdown};
use api::args::ApiArgs;
use api::config::load_config;
use clap::Parser;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = ApiArgs::parse();
    let mut port = 3000;
    let mut bind_ip = "127.0.0.1".to_string();

    if let Some(config_path) = args.config_path.as_deref() {
        let load_result = load_config(&config_path.to_path_buf());
        if let Err(e) = load_result {
            eprintln!("Failed to load configuration file, using default configuration: \n\t{e}")
        }else if let Ok(config) = load_result {
            port = config.bind_port;
            bind_ip = config.bind_ip;
        }
    }
    
    println!("Starting server on port:  {}", port);
    HttpServer::new(|| App::new().service(run).service(shutdown))
        .bind((bind_ip, port))?
        .run()
        .await
}
