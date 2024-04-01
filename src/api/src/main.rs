use actix_web::{App, HttpServer};
use api::services::{configuration, logs, metrics, run, shutdown};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 3000;

    println!("Starting server on port:  {}", port);
    HttpServer::new(|| {
        App::new()
            .service(configuration)
            .service(run)
            .service(logs)
            .service(metrics)
            .service(shutdown)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}