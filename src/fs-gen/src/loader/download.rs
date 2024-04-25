use flate2::read::GzDecoder;
use reqwest::blocking::{Client, Response};
use std::fs::create_dir;
use std::path::PathBuf;
use tar::Archive;
use tracing::{debug, info};
use anyhow::{Context, Result};
use crate::loader::errors::ImageLoaderError;
use crate::loader::utils::{get_docker_download_token, unpack_tarball};

pub(crate) fn download_image_fs(
    image_name: &str,
    output_file: PathBuf,
) -> Result<Vec<PathBuf>, ImageLoaderError> {
    // Get image's name and tag
    let image_and_tag: Vec<&str> = image_name.split(':').collect();

    let tag = if image_and_tag.len() < 2 {
        "latest"
    } else {
        image_and_tag[1]
    };
    let image_name = image_and_tag[0];

    // Get download token
    let client = Client::new();
    let token = &get_docker_download_token(&client, image_name)?;

    // Download image manifest
    let mut manifest_json = download_manifest(&client, token, image_name, tag)
        .map_err(|e| ImageLoaderError::Error { source: e })?;

    // Verify if it's a manifest or a manifest list
    let mut layers = manifest_json["layers"].as_array();

    if layers.is_none() {
        let manifests = manifest_json["manifests"].as_array();
        match manifests {
            None => Err(ImageLoaderError::ManifestNotFound(image_name.to_string(), tag.to_string()))?,
            Some(m) => {
                debug!("Manifest list found. Looking for an amd64 manifest...");
                // TODO: implement other than amd64?
                // Get a manifest for amd64 architecture from the manifest list
                let amd64_manifest = m.iter().find(|manifest| {
                    manifest["platform"].as_object().unwrap()["architecture"]
                        .as_str()
                        .unwrap()
                        == "amd64"
                });

                match amd64_manifest {
                    None => Err(ImageLoaderError::UnsupportedArchitecture("amd64".to_string()))?,
                    Some(m) => {
                        info!("Downloading image...");
                        debug!("Downloading manifest for amd64 architecture...");

                        manifest_json = download_manifest(
                            &client,
                            token,
                            image_name,
                            m["digest"].as_str().unwrap()
                        ).map_err(|e| ImageLoaderError::Error { source: e })?;

                        layers = manifest_json["layers"].as_array();
                        if layers.is_none() {
                            Err(ImageLoaderError::LayersNotFound)?
                        }
                    }
                }
            }
        }
    }

    let _ = create_dir(&output_file);

    download_layers(
        layers.unwrap(),
        &client,
        token,
        image_name,
        &output_file
    ).map_err(|e| ImageLoaderError::Error { source: e })
}

fn download_manifest(client: &Client, token: &str, image_name: &str, digest: &str) -> Result<serde_json::Value> {
    // Query Docker Hub API to get the image manifest
    let manifest_url = format!(
        "https://registry-1.docker.io/v2/library/{}/manifests/{}",
        image_name, digest
    );

    let manifest: serde_json::Value = client
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
    layers: &Vec<serde_json::Value>,
    client: &Client,
    token: &str,
    image_name: &str,
    output_dir: &PathBuf,
) -> Result<Vec<PathBuf>> {
    let mut layer_paths = Vec::new();

    debug!("Downloading and unpacking layers...");

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

        let tar = GzDecoder::new(response);

        let mut output_path = PathBuf::new();
        output_path.push(output_dir);
        output_path.push(digest);

        unpack_tarball(tar, &output_path)?;
        debug!("layer '{}' unpacked", digest);
        layer_paths.push(output_path);
    }

    info!("Layers downloaded successfully!");

    Ok(layer_paths)
}
