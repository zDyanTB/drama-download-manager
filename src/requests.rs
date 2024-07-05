use reqwest::{header, RequestBuilder, Response};
use crate::errors::LeviError;

// pub struct Requested {
//     pub response: Response,
//     pub content_length: Option<usize>,
//     pub resumable: bool,
// }

// impl Requested {
//     pub async fn new(request: RequestBuilder) -> Result<(), LeviError> {

//         let res = check_request(request).await?;
//         let content_len = res.content_length();

//         
//         
//         Ok(())
//     }
//     
//     
// }

pub async fn check_request(request: RequestBuilder) -> Result<Response, LeviError> {
    println!("Checking request: {:?}", request);
    match request.send().await {
        Ok(res) => {
            if res.status().is_success() {
                Ok(res)
            } else {
                Err(LeviError::HttpStatus(res.status()))
            }
        },
        Err(e) => {
            Err(LeviError::Network(e))
        },
    }
}

// Used to get content-length from HEAD request
// since .content_length() only returns from GET
pub async fn get_content_length(res: &Response) -> Option<u64> {
    let headers = res.headers();
    headers
        .get("content-length")
        .and_then(|ct_len| ct_len.to_str().ok())
        .and_then(|ct_len| ct_len.parse().ok())
}

pub async fn is_resumable(res: &Response) -> bool {
    println!("Cheking if request support range 'bytes'");
    match res.headers().get(header::ACCEPT_RANGES) {
        Some(_) => true,
        None => false,
    }
}
