use tonic::transport::Channel;
use vmmorchestrator::vmm_service_client::VmmServiceClient;

pub mod vmmorchestrator {
    tonic::include_proto!("vmmorchestrator");
}

pub struct VmmClient {
    client: VmmServiceClient<Channel>,
}

impl VmmClient {
    pub async fn new() -> Result<Self, tonic::transport::Error> {
        let client = VmmServiceClient::connect("http://[::1]:50051")
            .await
            .expect("Failed to connect to VMM service");

        Ok(VmmClient { client })
    }

    pub async fn run_vmm(&mut self) {
        let request = tonic::Request::new(vmmorchestrator::RunVmmRequest {
            language: vmmorchestrator::Language::Rust as i32,
            env: "fn main() { println!(\"Hello, World!\"); }".to_string(),
            code: "fn main() { println!(\"Hello, World!\"); }".to_string(),
            log_level: vmmorchestrator::LogLevel::Info as i32,
        });

        let response = self.client.run(request).await.unwrap();
        println!("RESPONSE={:?}", response);
    }
}
