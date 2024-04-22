use flate2::read::GzDecoder;
use reqwest::blocking::{Client, Response};
use std::error::Error;
use std::fs::create_dir;
use std::path::PathBuf;
use tar::Archive;

pub fn download_image_fs(
    image_name: &str,
    output_file: PathBuf,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    // Get image's name and tag
    let image_and_tag: Vec<&str> = image_name.split(':').collect();

    let tag = if image_and_tag.len() < 2 {
        "latest"
    } else {
        image_and_tag[1]
    };
    let image_name = image_and_tag[0];

    // Download image manifest
    let mut manifest_json = download_manifest(image_name, tag)?;

    // Verify if it's a manifest or a manifest list
    let mut layers = manifest_json["layers"].as_array();

    if layers.is_none() {
        let manifests = manifest_json["manifests"].as_array();
        match manifests {
            None => Err(format!(
                "Couldn't find a Docker V2 or OCI manifest for {}:{}",
                image_name, tag
            ))?,
            Some(m) => {
                println!("Manifest list found. Looking for an amd64 manifest...");
                // Get a manifest for amd64 architecture from the manifest list
                let amd64_manifest = m.iter().find(|manifest| {
                    manifest["platform"].as_object().unwrap()["architecture"]
                        .as_str()
                        .unwrap()
                        == "amd64"
                });

                match amd64_manifest {
                    None => Err("This image doesn't support amd64 architecture")?,
                    Some(m) => {
                        println!("Downloading manifest for amd64 architecture...");
                        manifest_json =
                            download_manifest(image_name, m["digest"].as_str().unwrap())?;
                        layers = manifest_json["layers"].as_array();
                        if layers.is_none() {
                            Err("Couldn't find image layers in the manifest.")?
                        }
                    }
                }
            }
        }
    }

    let _ = create_dir(&output_file);

    download_layers(layers.unwrap(), image_name, &output_file)
}

fn download_manifest(image_name: &str, digest: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    // Create a reqwest HTTP client
    let client = Client::new();

    // Get a token for anonymous authentication to Docker Hub
    let token_json: serde_json::Value = client
        .get(format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/{image_name}:pull"))
        .send()?.json()?;

    let token = token_json["token"].as_str().unwrap();

    // Query Docker Hub API to get the image manifest
    let manifest_url = format!(
        "https://registry-1.docker.io/v2/library/{}/manifests/{}",
        image_name, digest
    );

    let manifest_response = client
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
        .send()?;

    let manifest_json: serde_json::Value = manifest_response.json()?;

    println!("{}", manifest_json);

    Ok(manifest_json)
}

fn unpack_tarball(tar: GzDecoder<Response>, output_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut ar = Archive::new(tar);
    ar.unpack(output_dir)?;
    Ok(())
}

fn download_layers(
    layers: &Vec<serde_json::Value>,
    image_name: &str,
    output_dir: &PathBuf,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let client = Client::new();

    // Get a token for anonymous authentication to Docker Hub
    let token_json: serde_json::Value = client
        .get(format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/{image_name}:pull"))
        .send()?.json()?;

    let token = token_json["token"].as_str().unwrap();

    let mut layer_paths = Vec::new();

    println!("Downloading and unpacking layers:");

    // Download and unpack each layer
    for layer in layers {
        let digest = layer["digest"].as_str().unwrap();
        let layer_url = format!(
            "https://registry-1.docker.io/v2/library/{}/blobs/{}",
            image_name, digest
        );

        let response = client.get(&layer_url).bearer_auth(token).send()?;

        print!(" - {}", digest);

        let tar = GzDecoder::new(response);

        let mut output_path = PathBuf::new();
        output_path.push(output_dir);
        output_path.push(digest);

        unpack_tarball(tar, &output_path)?;
        println!(" - unpacked");
        layer_paths.push(output_path);
    }
    Ok(layer_paths)
}
