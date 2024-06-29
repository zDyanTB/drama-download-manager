use errors::LeviError;
use reqwest::Client;

mod errors;
mod requests;
mod files;
mod downloads;
mod links;

#[tokio::main]
async fn main() -> Result<(), LeviError> {
    let client = Client::new();
    let path = "/home/zdyant/Downloads/file.iso";
    // let url = "https://zrn.co/17lite64";
    let url = "https://cdn.discordapp.com/attachments/729867043309879316/1256336596538494986/102763838.jpg?ex=66806622&is=667f14a2&hm=3e7edcb98e9b9789013c28255193c2df43cfec12277cd4f9cd516045b1cad0bd&";
    // let url = "https://free.download.zorinos.com/17/Zorin-OS-17.1-Lite-64-bit-r2.iso";
    let timeout = 30;

    downloads::download_file(&client, timeout, path, url).await?;
    
    Ok(())
}
