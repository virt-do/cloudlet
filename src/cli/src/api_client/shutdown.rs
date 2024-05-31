use super::Error;
use shared_models::CloudletShutdownResponse;

pub async fn shutdown(base_url: &str) -> Result<CloudletShutdownResponse, Error> {
    let client = reqwest::Client::new();

    client
        .post(format!("{base_url}/shutdown"))
        .send()
        .await
        .map_err(Error::ShutdownSendRequest)?
        .json::<CloudletShutdownResponse>()
        .await
        .map_err(Error::ShutdownSendRequest)
}
