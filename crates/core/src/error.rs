/// Result alias that carries the custom [`MusicVizError`] type.
pub type Result<T> = std::result::Result<T, MusicVizError>;

/// Common error type for the core crate.
#[derive(Debug, thiserror::Error)]
pub enum MusicVizError {
    /// Placeholder variant used while concrete subsystems are under
    /// construction. It allows the higher level application to surface a
    /// readable message without committing to a particular error taxonomy yet.
    #[error("{0}")]
    Message(String),
    /// Wrapper around standard IO errors.
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl MusicVizError {
    /// Creates a new error that simply wraps the provided message.
    pub fn msg<T: Into<String>>(msg: T) -> Self {
        Self::Message(msg.into())
    }
}

impl From<&str> for MusicVizError {
    fn from(value: &str) -> Self {
        Self::msg(value)
    }
}

impl From<String> for MusicVizError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}
