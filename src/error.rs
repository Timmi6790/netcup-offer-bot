use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Tracing error")]
    Logger(#[from] tracing::metadata::ParseLevelError),
    #[error("Config error: {0}")]
    ConfigVar(String),
    #[error("Parser error")]
    ParseError(#[from] std::num::ParseIntError),
    #[error("Rss: {0}")]
    Rss(String),
    #[error("Reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("Reqwest middleware error")]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),
    #[error("Tokio error")]
    TokioJoinError(#[from] tokio::task::JoinError),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("Serde error")]
    Serde(#[from] serde_json::Error),
    #[error("Custom: {0}")]
    Custom(String),
}

impl Error {
    pub fn custom(msg: String) -> Self {
        Self::Custom(msg)
    }
}

impl From<rss::Error> for Error {
    fn from(err: rss::Error) -> Self {
        Self::Rss(err.to_string())
    }
}

impl From<rss::validation::ValidationError> for Error {
    fn from(err: rss::validation::ValidationError) -> Self {
        Self::Rss(err.to_string())
    }
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Self::ConfigVar(err.to_string())
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::Custom(err.to_string())
    }
}
