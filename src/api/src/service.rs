use crate::client::{
    vmmorchestrator::{ExecuteResponse, RunVmmRequest, ShutdownVmRequest, ShutdownVmResponse},
    VmmClient,
};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use actix_web_lab::sse;
use async_stream::stream;
use serde::Serialize;
use shared_models::{CloudletDtoRequest, Language};
use tokio_stream::StreamExt;
use tonic::Streaming;

#[post("/run")]
pub async fn run(req_body: web::Json<CloudletDtoRequest>) -> impl Responder {
    let req = req_body.into_inner();

    let mut client = VmmClient::new().await.unwrap();

    println!("Request: {:?}", req);

    let vmm_request = RunVmmRequest {
        workload_name: req.workload_name,
        code: req.code,
        language: match req.language {
            Language::RUST => 0,
            Language::PYTHON => 1,
            Language::NODE => 2,
        },
        log_level: req.log_level as i32,
    };

    println!("Request: {:?}", vmm_request);

    println!("Successfully connected to VMM service");

    let mut response_stream: Streaming<ExecuteResponse> =
        client.run_vmm(vmm_request).await.unwrap();
    println!("Response stream: {:?}", response_stream);

    let stream = stream! {
        while let Some(Ok(exec_response)) = response_stream.next().await {
            let json: ExecuteJsonResponse = exec_response.into();
            yield sse::Event::Data(sse::Data::new_json(json).unwrap());
        }
    };

    sse::Sse::from_infallible_stream(stream)
}

#[derive(Debug, Serialize)]
pub struct ExecuteJsonResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl From<ExecuteResponse> for ExecuteJsonResponse {
    fn from(value: ExecuteResponse) -> Self {
        Self {
            stdout: value.stdout,
            stderr: value.stderr,
            exit_code: value.exit_code,
        }
    }
}

#[post("/shutdown")]
pub async fn shutdown(request: HttpRequest) -> impl Responder {
    let req = request;

    let mut client = VmmClient::new().await.unwrap();

    println!("Request: {:?}", req);

    let shutdown_request = ShutdownVmRequest {};
    let response_result = client.shutdown_vm(shutdown_request).await;

    match response_result {
        Ok(response) => {
            let json_response: ShutdownJsonResponse = response.into();
            HttpResponse::Ok().body(serde_json::to_string(&json_response).unwrap())
        }
        Err(_) => {
            let json_response: ShutdownJsonResponse = ShutdownJsonResponse { success: false };
            return HttpResponse::Ok().body(serde_json::to_string(&json_response).unwrap());
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ShutdownJsonResponse {
    pub success: bool,
}

impl From<ShutdownVmResponse> for ShutdownJsonResponse {
    fn from(value: ShutdownVmResponse) -> Self {
        Self {
            success: value.success,
        }
    }
}
