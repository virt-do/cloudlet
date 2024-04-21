use crate::client::vmmorchestrator::RunVmmRequest;
use crate::client::VmmClient;
use actix_web::{post, web, HttpResponse, Responder};
use shared_models::CloudletDtoRequest;

#[post("/run")]
pub async fn run(req_body: web::Json<CloudletDtoRequest>) -> impl Responder {
    let req = req_body.into_inner();
    let grpc_client = VmmClient::new().await;

    let vmm_request = RunVmmRequest {
        code: req.code,
        env: req.env,
        language: req.language as i32,
        log_level: req.log_level as i32,
    };

    match grpc_client {
        Ok(mut client) => {
            println!("Successfully connected to VMM service");
            client.run_vmm(vmm_request).await;
            HttpResponse::Ok().body("Successfully ran VMM")
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
