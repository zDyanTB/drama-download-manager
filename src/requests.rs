use std::u64;

use reqwest::{header::HeaderMap, Client, RequestBuilder, Response};
use crate::errors::LeviError;

pub async fn check_request(request: RequestBuilder) -> Result<bool, LeviError> {
    match request.send().await {
        Ok(res) => {
            if res.status().is_success() {
                Ok(true)
            } else {
                Err(LeviError::HttpStatus(res.status()))
            }
        },
        Err(e) => {
            Err(LeviError::Network(e))
        },
    }
}

pub async fn get_headers(client: &Client, url: &str, response: &Response) -> Result<Option<u64>, LeviError>{
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
