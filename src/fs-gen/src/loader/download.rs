use crate::loader::errors::ImageLoaderError;
use crate::loader::structs::{Layer, ManifestV2, Registry};
use crate::loader::utils::{
    get_docker_download_token, get_registry, split_image_name, unpack_tarball,
};
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use super::structs::Image;

pub(crate) fn download_image_fs(
    image_name: &str,
    architecture: &str,
    output_file: PathBuf,
    registry_name: &str,
) -> Result<Vec<PathBuf>, ImageLoaderError> {
    info!("Downloading image...");
    let registry = get_registry(registry_name)?;
    let image = split_image_name(image_name);

    // Get download token and download manifest
    let client = Client::new();
    let token = &get_docker_download_token(&client, &image, &registry)?;
    let manifest = download_manifest(&client, token, &image, &image.tag, &registry)
        .map_err(|e| ImageLoaderError::Error { source: e })?;

    if let ManifestV2::ImageManifest(m) = manifest {
        // We directly get the image manifest rather than a list of manifests (fat manifest)
        info!("Found layers in manifest");
        warn!(
            "{}:{} is not a multi-platform image, the initramfs is not guaranteed to work correctly on the architecture {}",
            image.name, image.tag, architecture
        );
        create_dir_all(&output_file)
            .with_context(|| "Could not create output directory for image downloading")?;
        return download_layers(&m.layers, &client, token, &image, &output_file, &registry)
            .map_err(|e| ImageLoaderError::Error { source: e });
    }

    // Below, we assume that the image is multi-platform and we received a list of manifests (fat manifest).
    // We dig into sub-manifests to try and find a suitable one to download, with hopefully layers inside.

    let manifest_list = match manifest {
        // The manifest structure doesn't correspond to a fat manifest, we throw an error.
        ManifestV2::ManifestList(m) => m,
        _ => Err(ImageLoaderError::ManifestNotFound(image.clone()))?,
    };
    info!(
        architecture,
        "Manifest list found. Looking for an architecture-specific manifest..."
    );

    let arch_specific_manifest = manifest_list
        .manifests
        .iter()
        .find(|manifest| manifest.platform.architecture == architecture);

    let submanifest = match arch_specific_manifest {
        None => Err(ImageLoaderError::UnsupportedArchitecture(
            architecture.to_string(),
        ))?,
        Some(m) => {
            debug!("Downloading architecture-specific manifest");

            download_manifest(&client, token, &image, &m.digest, &registry)
                .map_err(|e| ImageLoaderError::Error { source: e })?
        }
    };

    match submanifest {
        // The submanifest structure doesn't correspond to an image manifest, we throw an error.
        ManifestV2::ImageManifest(m) => {
            create_dir_all(&output_file)
                .with_context(|| "Could not create output directory for image downloading")?;
            download_layers(&m.layers, &client, token, &image, &output_file, &registry)
                .map_err(|e| ImageLoaderError::Error { source: e })
        }
        _ => Err(ImageLoaderError::ImageManifestNotFound(image.clone()))?,
    }
}

fn download_manifest(
    client: &Client,
    token: &str,
    image: &Image,
    digest: &str,
    registry: &Registry,
) -> Result<ManifestV2> {
    // Query Docker Hub API to get the image manifest
    let manifest_url = format!(
        "{}/v2/{}/{}/manifests/{}",
        registry.api_v2_link, image.repository, image.name, digest
    );

    let manifest: ManifestV2 = client
        .get(manifest_url)
        .header(
            "Accept",
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .header(
            "Accept",
            "application/vnd.docker.distribution.manifest.list.v2+json",
        )
        .header("Accept", "application/vnd.oci.image.manifest.v1+json")
        .header("Accept", "application/vnd.oci.image.index.v1+json")
        .bearer_auth(token)
        .send()
        .with_context(|| "Could not send request to get manifest data".to_string())?
        .json()
        .with_context(|| "Failed to parse manifest to JSON".to_string())?;

    debug!(
        manifest = ?manifest,
        "downloaded manifest: "
    );

    Ok(manifest)
}

fn download_layers(
    layers: &Vec<Layer>,
    client: &Client,
    token: &str,
    image: &Image,
    output_dir: &Path,
    registry: &Registry,
) -> Result<Vec<PathBuf>> {
    info!("Downloading and unpacking layers...");

    let mut layer_paths = Vec::new();

    // Download and unpack each layer
    for layer in layers {
        let digest = &layer.digest;
        let layer_url = format!(
            "{}/v2/{}/{}/blobs/{}",
            registry.api_v2_link, image.repository, image.name, digest
        );

        let response = client
            .get(&layer_url)
            .bearer_auth(token)
            .send()
            .with_context(|| format!("Could not send request for layer digest '{digest}'"))?;

        debug!("starting to decode layer with digest '{}'", digest);

        let output_path = output_dir.join(digest);

        unpack_tarball(response, &output_path)?;
        debug!("layer '{}' unpacked", digest);
        layer_paths.push(output_path);
    }

    info!("Layers downloaded successfully!");

    Ok(layer_paths)
}
