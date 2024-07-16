use futures::{ stream, StreamExt};
use reqwest::{Client, RequestBuilder};
use tokio::io::{BufReader, BufWriter, AsyncReadExt, AsyncWriteExt};
use tokio::fs::{self, File};
use tokio::select;
use tokio_util::sync::CancellationToken;
use tokio::time::{timeout, Duration};
use std::path::Path;

use crate::links::FileLink;
use crate::errors::LeviError;
use crate::requests::{ check_request, get_content_length, is_resumable };
use crate::files::{create_file, get_file_size};

// TODO debug this file
pub async fn download(client: &Client, timeout_in_secs: usize, num_parts: usize, file_path: String, url: &str, token: &CancellationToken) -> Result<String, LeviError> {
    // https://github.com/agourlay/dlm/issues/293
    let head_res = check_request(client.head(url)).await?;
    let resumable = is_resumable(&head_res).await;
    let total_size = match get_content_length(&head_res).await {
        Some(ct) => ct,
        None => return Err(LeviError::Url("Url doesn't provide content_length".to_string())),
    };
    println!("{total_size}");


    let file_link = FileLink::new(url)?;
    let (extension, filename) = match file_link.extension {
        Some(ext) => (ext, file_link.filename),
        None => {( "noext".to_string(), file_link.filename )},
    };
    let file_destination = format!("{}{}", file_path, filename);
    let final_file_path = format!("{}.{}", file_destination, extension);

    if Path::new(&final_file_path).try_exists()? {

        if let Ok(size_on_disk) =  get_file_size(&final_file_path).await {       
            if size_on_disk > 0 && size_on_disk == total_size {
                println!("File was already fully downloaded");
                return Ok("File already on disk".to_string());
            }
        }

    }

    // if resumable && Path::new(&final_file_path).try_exists()? {
    //     
    // }

    let part_size = total_size / num_parts as u64;
    let file_parts = stream::iter(0..num_parts).map(|part| {
        let file_destination = format!("{file_destination}.part{part}");
        let req = client.get(url);
        let token = token.clone();

        async move {
            let _ = download_part(file_destination, req, timeout_in_secs, part, part_size, num_parts, total_size, token).await;
        }
    });
    file_parts.buffer_unordered(num_parts).collect::<Vec<_>>().await;

    let final_file = create_file(&final_file_path, resumable).await?;
    merge_parts(final_file, file_destination, num_parts).await?;
    Ok("file downloaded".to_string())
}

async fn download_part(
    file_path: String,
    req: RequestBuilder,
    timeout_in_secs: usize,
    part_number: usize,
    part_size: u64,
    num_parts: usize,
    total_size: u64,
    token: CancellationToken,
) -> Result<(), LeviError> {

    let start = part_number as u64 * part_size;
    let end = if part_number == num_parts -1 {
        total_size
    } else {
        (part_number as u64 + 1) * part_size - 1
    };

    let req = req.header("Range", format!("bytes={}-{}", start, end));
    let mut res = check_request(req).await?;

    // println!("Total size: {total_size}");
    // let mut progress = file_size;

    let f = create_file(&file_path, true).await?;
    let mut file = BufWriter::new(f);
    let chunk_timeout = Duration::from_secs(timeout_in_secs as u64);
    loop {
        select! {
            _ = token.cancelled() => {
                file.flush().await?;
                println!("program stopped");
                return Ok(());
            }
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

// TODO use BufRead and BufWrite 
async fn merge_parts(mut final_file: File, parts_path: String, num_parts: usize) -> Result<(), LeviError> {

    for part in 0..num_parts {
        let part_path = format!("{}.part{}", parts_path, part);
        let part_file = File::open(&part_path).await?;

        let mut reader = BufReader::new(part_file);
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).await?;

        final_file.write_all(&buffer).await?;
        final_file.flush().await?;

        fs::remove_file(&part_path).await?;
    }
    Ok(())
}
