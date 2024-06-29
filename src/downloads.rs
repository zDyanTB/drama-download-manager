use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::time::{timeout, Duration};

use crate::errors::LeviError;
use crate::requests::check_request;
use crate::files::create_file;

pub async fn download_file(client: &Client, timeout_in_secs: usize, path: &str, url: &str) -> Result<(), LeviError> {

    // https://github.com/agourlay/dlm/issues/293
    if let Err(e) = check_request(client.head(url)).await {
        return Err(e);
    }

    let mut file = create_file(path).await?;
    let mut res = client.get(url).send().await?;

    let chunk_timeout = Duration::from_secs(timeout_in_secs as u64);
    loop {
       select! {
            chunk = timeout(chunk_timeout, res.chunk()) => {
                let chunk = chunk??;
               
                if let Some(chunk) = chunk {
                    file.write_all(&chunk).await?;
                    file.flush().await?;
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
