use errors::LeviError;
use reqwest::Client;

mod errors;
mod requests;

#[tokio::main]
async fn main() -> Result<(), RequestError> {
    let client = Client::new();

    let urls = vec![
        "https://google.com",
        "https://youtube.com",
    ];

    for url in urls {
        let response = client.head(url).send().await?;

        requests::check_url(&client, url).await?;
        requests::get_headers(&client, url, &response).await?;
    }

    Ok(())
}
