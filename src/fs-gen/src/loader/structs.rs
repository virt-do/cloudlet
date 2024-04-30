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
    pub repository: String,
    pub name: String,
    pub tag: String,
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
