use actix_web::{App, HttpServer};
use api::service::{run, shutdown};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 3000;

    println!("Starting server on port:  {}", port);
    HttpServer::new(|| App::new().service(run).service(shutdown))
        .bind(("127.0.0.1", port))?
        .run()
        .await
}
