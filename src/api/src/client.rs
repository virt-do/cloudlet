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

    pub async fn run_vmm(&mut self, request: vmmorchestrator::RunVmmRequest) {
        let request = tonic::Request::new(request);
        let response = self.client.run(request).await.unwrap();
        println!("RESPONSE={:?}", response);
    }
}
