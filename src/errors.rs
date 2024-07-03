use reqwest;
use thiserror::Error;
use tokio::io;
use tokio::time::error::Elapsed;

#[derive(Error, Debug)]
pub enum LeviError {

    #[error("HTTP request failed with status: {0:?}")]
    HttpStatus(reqwest::StatusCode),
   
    #[error("Network or request error: {0:?}")]
    Network(#[from] reqwest::Error),

    #[error("Url error: {0:?}")]
    Url(String),

    #[error("Elapsed timeout")]
    TimeoutElapsed(#[from] Elapsed),
   
    #[error("Error checking on file: {0:?}")]
    File(#[from] io::Error),
   
    #[error("File already exists and doesn't support resuming: {0:?}")]
    FileExists(String),
}
