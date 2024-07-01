use errors::LeviError;
use reqwest::Client;
use futures::{stream, StreamExt};

mod errors;
mod requests;
mod files;
mod downloader;
mod links;

#[tokio::main]
async fn main() -> Result<(), LeviError> {
    let client = Client::new();
    let concurrent_downloads = 8;
    let path = "/home/zdyant/Downloads/";
    let urls = vec![
        "https://github.com/audacity/audacity/releases/download/Audacity-3.5.1/audacity-linux-3.5.1-x64.AppImage",
        "https://free.download.zorinos.com/17/Zorin-OS-17.1-Lite-64-bit-r2.iso",
        "https://iso.pop-os.org/22.04/amd64/intel/41/pop-os_22.04_amd64_intel_41.iso",
    ];

    let timeout = 10;


    let downloads = stream::iter(urls)
        .map(|url| downloader::download(&client, timeout, path, url))
        .buffer_unordered(concurrent_downloads)
        .collect::<Vec<_>>()
        .await;
    
    for file in downloads {
        match file {
            Ok(_) => println!("File downloaded successfully."),
            Err(e) => eprintln!("Error downloading file: {}", e),
        }
    }
    
    
    Ok(())
}
