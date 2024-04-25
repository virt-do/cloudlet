use anyhow::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ImageLoaderError {
    /// There is no existing manifest for the given image.
    #[error("Could not find Docker v2 or OCI manifest for `{0}:{1}`")]
    ManifestNotFound(String, String),

    /// Image doesn't support the requested architecture.
    #[error("This image doesn't support {0} architecture")]
    UnsupportedArchitecture(String),

    /// The manifest doesn't contain any layers to unpack.
    #[error("Could not find image layers in the manifest")]
    LayersNotFound,

    /// Encountered an error during the flow.
    #[error("Image loading error: {}", .source)]
    Error {
        source: anyhow::Error
    }
}

impl From<anyhow::Error> for ImageLoaderError {
    fn from(value: Error) -> Self {
        Self::Error { source: value }
    }
}
