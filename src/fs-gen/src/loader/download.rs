use reqwest::blocking::Client;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use anyhow::{Context, Result};
use serde_json::Value;
use crate::loader::errors::ImageLoaderError;
use crate::loader::utils::{get_docker_download_token, unpack_tarball};

pub(crate) fn download_image_fs(
    image_name: &str,
    output_file: PathBuf,
) -> Result<Vec<PathBuf>, ImageLoaderError> {
    info!("Downloading image...");

    // Get image's name and tag
    let image_and_tag: Vec<&str> = image_name.split(':').collect();
    let image_name = image_and_tag[0];
    let tag = if image_and_tag.len() < 2 { "latest" } else { image_and_tag[1] };

    // Get download token and download manifest
    let client = Client::new();
    let token = &get_docker_download_token(&client, image_name)?;
    let manifest = download_manifest(&client, token, image_name, tag)
        .map_err(|e| ImageLoaderError::Error { source: e })?;


    if let Some(layers) = manifest["layers"].as_array() {
        // We have layers already, no need to look into sub-manifests.
        info!("Found layers in manifest");
        create_dir_all(&output_file).with_context(|| "Could not create output directory for image downloading")?;
        return download_layers(
            layers,
            &client,
            token,
            image_name,
            &output_file
        ).map_err(|e| ImageLoaderError::Error { source: e })
    }

    // Below, we assume there are no layers found.
    // We dig into sub-manifests to try and find a suitable one to download, with hopefully layers inside.

    let manifest_list = match manifest["manifests"].as_array() {
        // No sub-manifests found, we throw an error.
        None => Err(ImageLoaderError::ManifestNotFound(image_name.to_string(), tag.to_string()))?,
        Some(m) => m
    };
    info!(architecture = "amd64", "Manifest list found. Looking for an architecture-specific manifest...");

    // TODO: implement other than amd64?
    let amd64_submanifest = manifest_list.iter().find(|manifest| {
        manifest["platform"].as_object().unwrap()["architecture"]
            .as_str()
            .unwrap()
            == "amd64"
    });

    let submanifest = match amd64_submanifest {
        None => Err(ImageLoaderError::UnsupportedArchitecture("amd64".to_string()))?,
        Some(m) => {
            debug!("Downloading architecture-specific manifest");

            

            download_manifest(
                &client,
                token,
                image_name,
                m["digest"].as_str().unwrap()
            ).map_err(|e| ImageLoaderError::Error { source: e })?
        }
    };
    
    match submanifest["layers"].as_array() {
        None => Err(ImageLoaderError::LayersNotFound)?,
        Some(layers) => {
            create_dir_all(&output_file).with_context(|| "Could not create output directory for image downloading")?;
            download_layers(
                layers,
                &client,
                token,
                image_name,
                &output_file
            ).map_err(|e| ImageLoaderError::Error { source: e })
        }
    }
}

fn download_manifest(client: &Client, token: &str, image_name: &str, digest: &str) -> Result<Value> {
    // Query Docker Hub API to get the image manifest
    let manifest_url = format!(
        "https://registry-1.docker.io/v2/library/{}/manifests/{}",
        image_name, digest
    );

    let manifest: Value = client
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
        .bearer_auth(token)
        .send().with_context(|| "Could not send request to get manifest data".to_string())?
        .json().with_context(|| "Failed to parse manifest to JSON".to_string())?;

    debug!(
        manifest = ?manifest,
        "downloaded manifest: "
    );

    Ok(manifest)
}

fn download_layers(
    layers: &Vec<Value>,
    client: &Client,
    token: &str,
    image_name: &str,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    info!("Downloading and unpacking layers...");

    let mut layer_paths = Vec::new();

    // Download and unpack each layer
    for layer in layers {
        let digest = layer["digest"].as_str()
            .with_context(|| "Failed to get digest for layer".to_string())?;
        let layer_url = format!(
            "https://registry-1.docker.io/v2/library/{}/blobs/{}",
            image_name, digest
        );

        let response = client.get(&layer_url).bearer_auth(token)
            .send().with_context(|| format!("Could not send request for layer digest '{digest}'"))?;

        debug!("starting to decode layer with digest '{}'", digest);

        let output_path = output_dir.join(digest);

        unpack_tarball(response, &output_path)?;
        debug!("layer '{}' unpacked", digest);
        layer_paths.push(output_path);
    }

    info!("Layers downloaded successfully!");

    Ok(layer_paths)
}
