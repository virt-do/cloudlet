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
pub(super) fn get_docker_download_token(
    client: &Client,
    image: &Image,
    registry: &Registry,
) -> Result<String> {
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

// Get image's repository, name and tag
pub(super) fn split_image_name(image_name: &str) -> Image {
    let repo_and_image: Vec<&str> = image_name.splitn(2, '/').collect();

    let (repository, name) = if repo_and_image.len() < 2 {
        ("library".to_string(), repo_and_image[0].to_string())
    } else {
        (repo_and_image[0].to_string(), repo_and_image[1].to_string())
    };
    let image_and_tag: Vec<&str> = name.split(':').collect();
    let (name, tag) = if image_and_tag.len() < 2 {
        (image_and_tag[0].to_string(), "latest".to_string())
    } else {
        (image_and_tag[0].to_string(), image_and_tag[1].to_string())
    };
    Image {
        repository,
        name,
        tag,
    }
}

pub(super) fn get_registry(name: &str) -> Result<Registry, ImageLoaderError> {
    let registries_file = include_str!("../../resources/registries.json");
    let registries: Vec<Registry> = serde_json::from_str(registries_file)
        .with_context(|| "Failed to parse registries JSON file")?;
    match registries.iter().find(|reg| reg.name == name) {
        Some(reg) => Ok(reg.clone()),
        _ => Err(ImageLoaderError::RegistryNotFound(name.to_string())),
    }
}
