use tokio::fs::{self, File};
use crate::errors::LeviError;
use std::path::Path;

pub async fn create_file(file_path: &String, append: bool) -> Result<File, LeviError> {

    if append {
        println!("Creating file on append mode");
        let file = fs::OpenOptions::new()
            .append(append)
            .create(true)
            .open(file_path)
            .await?;

        Ok(file)
    } else {
        if Path::new(file_path).try_exists()? {
            return Err(LeviError::FileExists(file_path.to_string()));
        }

        println!("Creating file on overwrite mode");
        Ok(File::create(file_path).await?)
    }
}

pub async fn get_file_size(file: &String) -> Result<u64, LeviError>{
    println!("Retrieving file size");
    let range = File::open(file)
        .await?
        .metadata()
        .await?
        .len();

    Ok(range)
}
