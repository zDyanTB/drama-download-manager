use reqwest;
use std::fmt;

#[derive(Debug)]
pub enum RequestError {
    Status(reqwest::StatusCode),
    Network(reqwest::Error),
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::Status(code) => write!(f, "HTTP request failed with status: {}", code),
            RequestError::Network(err) => write!(f, "Network or request error: {}", err),
        }
    }
}

impl std::error::Error for RequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RequestError::Status(_) => None,
            RequestError::Network(err) => Some(err),
        }
    }
}

impl From<reqwest::Error> for RequestError {
    fn from(err: reqwest::Error) -> Self {
        RequestError::Network(err)
    }
}
