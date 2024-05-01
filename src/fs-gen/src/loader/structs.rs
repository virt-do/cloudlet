use serde::Deserialize;
use serde_json::Value;
use std::fmt;

// Any json returned by the request: image manifest, fat manifest, error...
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ManifestV2 {
    ImageManifest(ImageManifest),
    ManifestList(ManifestList),
    Other(Value),
}

// Docker v2 or OCI mage manifest containing image layers
#[derive(Debug, Deserialize)]
pub struct ImageManifest {
    pub layers: Vec<Layer>,
}

// Image layer
#[derive(Debug, Deserialize)]
pub struct Layer {
    pub digest: String,
}

// Docker v2 manifest list or OCI image index containing image manifests
#[derive(Debug, Deserialize)]
pub struct ManifestList {
    pub manifests: Vec<SubManifest>,
}

// SubManifest for a specific platform
#[derive(Debug, Deserialize)]
pub struct SubManifest {
    pub digest: String,
    pub platform: Platform,
}

// Supported image platform: architecture and OS
#[derive(Debug, Deserialize)]
pub struct Platform {
    pub architecture: String,
    pub os: String,
}

// Container image definition consisting of repository, name and tag
#[derive(Debug, Clone)]
pub struct Image {
    pub registry: String,
    pub repository: String,
    pub name: String,
    pub tag: String,
}

impl Image {
    // Get image's repository, name and tag
    pub fn from_str(image_name: &str) -> Image {
        const DEFAULT_REGISTRY: &str = "registry-1.docker.io";
        const DEFAULT_REPOSITORY: &str = "library";
        const DEFAULT_TAG: &str = "latest";

        let mut image_data: Vec<&str> = image_name
            .trim_start_matches("docker.io/")
            .splitn(3, '/')
            .collect();

        let registry = if image_data[0].contains('.') {
            image_data.remove(0).to_string()
        } else {
            DEFAULT_REGISTRY.to_string()
        };

        let (repository, name) = match image_data.len() {
            1 => (DEFAULT_REPOSITORY.to_string(), image_data[0].to_string()),
            2 => (image_data[0].to_string(), image_data[1].to_string()),
            _ => (
                image_data[0].to_string(),
                image_data[1].to_string() + "/" + image_data[2],
            ),
        };
        let image_and_tag: Vec<&str> = name.split(':').collect();
        let (name, tag) = if image_and_tag.len() < 2 {
            (image_and_tag[0].to_string(), DEFAULT_TAG.to_string())
        } else {
            (image_and_tag[0].to_string(), image_and_tag[1].to_string())
        };

        Image {
            registry,
            repository,
            name,
            tag,
        }
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}:{}", self.repository, self.name, self.tag)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Registry {
    pub name: String,
    pub auth_link: String,
    pub auth_service: String,
}
