use crate::client::VmmClient;
use actix_web::{post, HttpResponse, Responder};

#[post("/run")]
pub async fn run(req_body: String) -> impl Responder {
    let grpc_client = VmmClient::new().await;

    match grpc_client {
        Ok(mut client) => {
            client.run_vmm().await;
            HttpResponse::Ok().body(req_body)
        }
        Err(e) => HttpResponse::InternalServerError()
            .body("Failed to connect to VMM service with error: ".to_string() + &e.to_string()),
    }
}

#[post("/shutdown")]
pub async fn shutdown(req_body: String) -> impl Responder {
    // TODO: Get the id from the body and shutdown the vm
    HttpResponse::Ok().body(req_body)
}
