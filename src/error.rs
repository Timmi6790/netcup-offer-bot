use std::env::VarError;

use log::error;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Logger: {0}")]
    Logger(String),
    #[error("Config var: {0}")]
    ConfigVar(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Rss: {0}")]
    Rss(String),
    #[error("Reqwest: {0}")]
    Reqwest(String),
    #[error("tokio join error: {0}")]
    TokioJoinError(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serde: {0}")]
    Serde(String),
    #[error("Prometheus: {0}")]
    Prometheus(String),
    #[error("Custom: {0}")]
    Custom(String),
}

impl Error {
    pub fn custom(msg: String) -> Self {
        Self::Custom(msg)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err.to_string())
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::TokioJoinError(err.to_string())
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

impl From<log::SetLoggerError> for Error {
    fn from(err: log::SetLoggerError) -> Self {
        Self::Logger(err.to_string())
    }
}

impl From<VarError> for Error {
    fn from(err: VarError) -> Self {
        Self::ConfigVar(err.to_string())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::ParseError(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serde(err.to_string())
    }
}

impl From<prometheus::Error> for Error {
    fn from(err: prometheus::Error) -> Self {
        Self::Prometheus(err.to_string())
    }
}

impl From<prometheus_exporter::Error> for Error {
    fn from(err: prometheus_exporter::Error) -> Self {
        Self::Prometheus(err.to_string())
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::Custom(err.to_string())
    }
}
