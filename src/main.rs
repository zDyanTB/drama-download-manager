use errors::LeviError;
use reqwest::Client;
use futures::{select, stream, StreamExt};
use tokio::signal;
use tokio_util::sync::CancellationToken;

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
    let file_parts = 8;
    let urls = vec![
        // "https://github.com/audacity/audacity/releases/download/Audacity-3.5.1/audacity-linux-3.5.1-x64.AppImage",
        // "https://free.download.zorinos.com/17/Zorin-OS-17.1-Lite-64-bit-r2.iso",
        // "https://images6.alphacoders.com/136/1365892.png",
        "https://iso.pop-os.org/22.04/amd64/intel/41/pop-os_22.04_amd64_intel_41.iso",
    ];
    let timeout = 10;
    let token = CancellationToken::new();
    let token_clone = token.clone();

    let token_handle = tokio::spawn(async move {
        loop {
            signal::ctrl_c().await.expect("ctrl-c");
            token.cancel();
        }
    });

    let downloads = stream::iter(urls)
        .map(|url| downloader::download(&client, timeout, file_parts, path.to_string(), url, &token_clone))
        .buffer_unordered(concurrent_downloads)
        .collect::<Vec<_>>()
        .await;
    //
    // let mut tasks: Vec<JoinHandle<()>>= vec![];
    // let url = "https://images6.alphacoders.com/136/1365892.png";
    //     let client = client.clone();
    //     tasks.push(tokio::spawn(async move {
    //         let _ = downloader::download(&client, timeout, file_parts, path.to_string(), url).await;
    //     }));
    // 
    for file in downloads {
        match file {
            Ok(_) => println!("File downloaded successfully."),
            Err(e) => eprintln!("Error downloading file: {}", e),
        }
    }
    
    
    Ok(())
}
