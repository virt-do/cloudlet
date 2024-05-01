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
        .with_context(|| format!("Could not send request to {}", image.registry))?;
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
pub(super) fn get_docker_download_token(
    client: &Client,
    image: &Image,
    username: Option<String>,
    password: Option<String>,
) -> Result<String> {
    let registry = get_registry_auth_data(client, image)?;

    let mut request = client.get(format!(
        "{}?service={}&scope=repository:{}/{}:pull",
        registry.auth_link, registry.auth_service, image.repository, image.name
    ));
    let mut auth_type = "";

    match username {
        Some(u) => {
            request = request.basic_auth(u, password);
        }
        None => {
            auth_type = "anonymous ";
        }
    };

    let token_json: serde_json::Value = request
        .send()
        .with_context(|| format!("Could not send request for {}authentication", auth_type))?
        .json()
        .with_context(|| {
            format!(
                "Failed to parse JSON response for {}authentication",
                auth_type
            )
        })?;

    match token_json["token"].as_str().with_context(|| {
        format!(
            "Failed to get token from the {}authentication response",
            auth_type
        )
    }) {
        Ok(t) => Ok(t.to_owned()),
        Err(e) => Err(e),
    }
}
