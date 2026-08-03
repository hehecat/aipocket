use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("outbound request denied: {0}")]
    AuthorizationDenied(&'static str),
}
