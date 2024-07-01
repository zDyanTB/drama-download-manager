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
    if let Err(e) = check_request(client.head(url)).await {
        return Err(e);
    }
    
    let file_link = FileLink::new(url)?;
    let (extension, filename) = match file_link.extension {
        Some(ext) => (ext, file_link.filename),
        None => {( "noext".to_string(), "file_link.filename".to_string() )},
    };

    
    let res = check_request(client.get(url)).await?;
    let total_size = match res.content_length() {
        Some(ct) => ct,
        None => 0,
    };

    let resumable = is_resumable(&res).await;
    println!("is resumable {resumable}");


    let file_destination = format!("{path}{filename}.{extension}");
    let file = create_file(&file_destination, resumable).await?;
    let file_size = get_file_size(&file_destination).await?;

    write_file(file, res, timeout_in_secs, total_size).await?;
    Ok(())
}

async fn write_file(mut file: File, mut res: Response, timeout_in_secs: usize, total_size: u64) -> Result<(), LeviError> {

    println!("Total size: {total_size}");
    let mut progress = 0;

    let chunk_timeout = Duration::from_secs(timeout_in_secs as u64);
    loop {
       select! {
            chunk = timeout(chunk_timeout, res.chunk()) => {
                let chunk = chunk??;
           
                if let Some(chunk) = chunk {
                    file.write_all(&chunk).await?;
                    file.flush().await?;

                    // progress += chunk.len() as u64;
                    // println!("{:.2}%", ( progress as f64 / total_size as f64 ) * 100.0);
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
