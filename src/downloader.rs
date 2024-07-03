use reqwest::{Client, Response};
use tokio::io::AsyncWriteExt;
use tokio::fs::File;
use tokio::select;
use tokio::time::{timeout, Duration};

use crate::links::FileLink;
use crate::errors::LeviError;
use crate::requests::{ check_request, is_resumable };
use crate::files::{create_file, get_file_size};

pub async fn download(client: &Client, timeout_in_secs: usize, path: &str, url: &str) -> Result<(), LeviError> {
    // https://github.com/agourlay/dlm/issues/293
    
    let head_req = check_request(client.head(url)).await?;
    let resumable = is_resumable(&head_req).await;


    let file_link = FileLink::new(url)?;
    let (extension, filename) = match file_link.extension {
        Some(ext) => (ext, file_link.filename),
        None => {( "noext".to_string(), "file_link.filename".to_string() )},
    };
    let file_destination = format!("{path}{filename}.{extension}");
    let file = create_file(&file_destination, resumable).await?;


    let file_size = get_file_size(&file_destination).await?;
    let mut req = client.get(url);
    if resumable {
            req = req.header("Range", format!("bytes={}-", file_size));
    }

    let response = check_request(req).await?;
    let total_size = match response.content_length() {
        Some(ct) => ct,
        None => return Err(LeviError::Url("Url doesn't provide content_length".to_string())),
    };

    write_file(file, response, timeout_in_secs, file_size, total_size).await?;
    Ok(())
}

async fn write_file(mut file: File, mut res: Response, timeout_in_secs: usize, file_size: u64, total_size: u64) -> Result<(), LeviError> {

    println!("Total size: {total_size}");
    let mut progress = file_size;

    let chunk_timeout = Duration::from_secs(timeout_in_secs as u64);
    loop {
       select! {
            chunk = timeout(chunk_timeout, res.chunk()) => {
                let chunk = chunk??;
           
                if let Some(chunk) = chunk {
                    file.write_all(&chunk).await?;
                    file.flush().await?;

                    progress += chunk.len() as u64;
                    println!("{:.2}%", ( progress as f64 / total_size as f64 ) * 100.0);
                } else {
                    // End of download
                    file.flush().await?;
                    break;
                }
            }
        }
 
    }
    Ok(())
}
