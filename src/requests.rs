use std::u64;

use reqwest::{header::HeaderMap, Client, Response};
use crate::errors::RequestError;

pub async fn check_url(client: &Client, url: &str) -> Result<Response, RequestError> {
    match client.head(url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                Ok(res)
            } else {
                Err(RequestError::Status(res.status()))
            }
        },
        Err(e) => {
            Err(RequestError::Network(e))
        },
    }
}

pub async fn get_headers(client: &Client, url: &str, response: &Response) -> Result<Option<u64>, RequestError>{
    let header_content_length = extract_content_len_from_headers(response.headers());

    let content_length = match header_content_length {
        Some(0) => {
            // Site does not provide head
            // try get

            let get_result = client.get(url).send().await?;
            let get_content_length = get_result.content_length();

            get_content_length
        },
        Some(_) => {
            header_content_length
        },
        _ => {
            None
        }
    };
    Ok(content_length)
}

fn extract_content_len_from_headers(headers: &HeaderMap) -> Option<u64> {
    headers
        .get("content-length")
        .and_then(|ct_len| ct_len.to_str().ok())
        .and_then(|ct_len| ct_len.parse().ok())
}
