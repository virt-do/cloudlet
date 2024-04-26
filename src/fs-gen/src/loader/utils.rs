use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use reqwest::blocking::{Client, Response};
use std::path::Path;
use tar::Archive;

/// Unpack the tarball to a given directory.
pub(super) fn unpack_tarball(response: Response, output_dir: &Path) -> Result<()> {
    Archive::new(GzDecoder::new(response))
        .unpack(output_dir)
        .with_context(|| format!("Failed to unpack tarball to {}", output_dir.display()))?;
    Ok(())
}

/// Get a token for anonymous authentication to Docker Hub.
pub(super) fn get_docker_download_token(client: &Client, image_name: &str) -> Result<String> {
    let token_json: serde_json::Value = client
        .get(format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/{image_name}:pull"))
        .send().with_context(|| "Could not send request for anonymous authentication".to_string())?
        .json().with_context(|| "Failed to parse JSON response for anonymous authentication".to_string())?;

    match token_json["token"]
        .as_str()
        .with_context(|| "Failed to get token from anon auth response".to_string())
    {
        Ok(t) => Ok(t.to_owned()),
        Err(e) => Err(e),
    }
}
