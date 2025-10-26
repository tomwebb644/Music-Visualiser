use thiserror::Error;

/// Convenient result alias for the crate.
pub type Result<T, E = MusicVizError> = std::result::Result<T, E>;

/// Unified error type for the early scaffolding of the music visualiser.
#[derive(Debug, Error)]
pub enum MusicVizError {
    /// Raised when a feature is not yet implemented but the code path already
    /// exists to make future work easier to integrate.
    #[error("feature not yet implemented: {0}")]
    Unimplemented(&'static str),

    /// Raised when input data is invalid or unexpected.
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),

    /// Catch all variant for ad-hoc error messages.
    #[error("{0}")]
    Message(String),
}

impl MusicVizError {
    /// Creates an ad-hoc error with the provided message.
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl From<realfft::FftError> for MusicVizError {
    fn from(err: realfft::FftError) -> Self {
        Self::Message(err.to_string())
    }
}
