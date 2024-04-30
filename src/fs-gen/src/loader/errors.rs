use crate::loader::structs::Image;
use anyhow::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ImageLoaderError {
    /// There is no existing manifest for the given image.
    #[error("Could not find Docker v2 or OCI manifest for `{0}`")]
    ManifestNotFound(Image),

    /// Image doesn't support the requested architecture.
    #[error("This image doesn't support {0} architecture")]
    UnsupportedArchitecture(String),

    /// The image manifest doesn't match the expected structure (no "layers" property).
    #[error("Could not find Docker v2 or OCI image manifest for `{0}`")]
    ImageManifestNotFound(Image),

    /// The requested container registry didn't provide the auth link and service in www-authenticate header
    #[error("Could not get the auth link and service for the `{0}` registry.")]
    RegistryAuthDataNotFound(String),

    /// Encountered an error during the flow.
    #[error("Image loading error: {}", .source)]
    Error { source: anyhow::Error },
}

impl From<anyhow::Error> for ImageLoaderError {
    fn from(value: Error) -> Self {
        Self::Error { source: value }
    }
}
