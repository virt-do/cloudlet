use crate::loader::errors::ImageLoaderError;
use crate::loader::structs::{Image, Registry};
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
pub(super) fn get_registry_auth_data(
    client: &Client,
    image: &Image,
) -> Result<Registry, ImageLoaderError> {
    let manifest_url = format!(
        "https://{}/v2/{}/{}/manifests/{}",
        image.registry, image.repository, image.name, image.tag
    );

    let unauth_request = client
        .get(manifest_url)
        .send()
        .with_context(|| "Could not send request for unauthorized authentication".to_string())?;
    let auth_header: &str = unauth_request.headers()["www-authenticate"]
        .to_str()
        .map_err(|e| ImageLoaderError::Error { source: e.into() })?;
    let auth_data: Vec<&str> = auth_header.split('"').collect();
    if auth_data.len() != 7 {
        Err(ImageLoaderError::RegistryAuthDataNotFound(
            image.registry.clone(),
        ))?
    }
    Ok(Registry {
        name: image.registry.clone(),
        auth_link: auth_data[1].to_string(),
        auth_service: auth_data[3].to_string(),
    })
}

/// Get a token for anonymous authentication to Docker Hub.
pub(super) fn get_docker_download_token(client: &Client, image: &Image) -> Result<String> {
    let registry = get_registry_auth_data(client, image)?;

    let token_json: serde_json::Value = client
        .get(format!(
            "{}?service={}&scope=repository:{}/{}:pull",
            registry.auth_link, registry.auth_service, image.repository, image.name
        ))
        .send()
        .with_context(|| "Could not send request for anonymous authentication".to_string())?
        .json()
        .with_context(|| {
            "Failed to parse JSON response for anonymous authentication".to_string()
        })?;

    match token_json["token"]
        .as_str()
        .with_context(|| "Failed to get token from the anonymous auth response".to_string())
    {
        Ok(t) => Ok(t.to_owned()),
        Err(e) => Err(e),
    }
}
