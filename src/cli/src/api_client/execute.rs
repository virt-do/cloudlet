use super::Error;
use reqwest_eventsource::EventSource;
use serde::Deserialize;
use shared_models::CloudletDtoRequest;

pub async fn execute(base_url: &str, request: CloudletDtoRequest) -> Result<EventSource, Error> {
    let client = reqwest::Client::new()
        .post(format!("{base_url}/run"))
        .json(&request);

    EventSource::new(client).map_err(Error::CreateEventSource)
}

#[derive(Debug, Deserialize)]
pub struct ExecuteJsonResponse {
    pub stage: Stage,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub enum Stage {
    Pending,
    Building,
    Running,
    Done,
    Failed,
    Debug,
}

impl TryFrom<String> for ExecuteJsonResponse {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&value).map_err(|_| Error::ExecuteResponseDeserialize)
    }
}
