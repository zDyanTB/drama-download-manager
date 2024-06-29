use tokio::fs::File;
use std::path::Path;
use crate::errors::LeviError;

pub async fn create_file(file_path: &str) -> Result<File, LeviError> {

    if Path::new(file_path).try_exists()? {
        Err(LeviError::FileExists(file_path.to_string()))
    } else {
        Ok(File::create(file_path).await?)
    }

}
