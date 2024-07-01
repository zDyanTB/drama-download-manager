use tokio::fs::{self, File};
use crate::errors::LeviError;

pub async fn create_file(file_path: &String, append: bool) -> Result<File, LeviError> {

    if append {
        let file = fs::OpenOptions::new()
            .append(append)
            .open(file_path)
            .await?;

        Ok(file)
    } else {

        Ok(File::create(file_path).await?)
    }
}

pub async fn get_file_size(file: &String) -> Result<Option<String>, LeviError>{
    let file_size = File::open(file)
        .await?
        .metadata()
        .await?
        .len();

    let range = format!("bytes={}-", file_size);

    Ok(Some(range))
}
