use crate::errors::LeviError;
use std::str;

pub struct FileLink {
    pub url: String,
    pub filename: String,
    pub extension: Option<String>,
}

impl FileLink {

    pub fn new(url: &str) -> Result<FileLink, LeviError> {
        let trimmed = url.trim();

        if trimmed.is_empty() {
            return Err(LeviError::Url(format!("Url cannot be empty")));
        }

        if trimmed.ends_with('/') {
            return Err(LeviError::Url(format!("Url couldn't find an extension: {url}")));
        }

        let url_decoded = url_decode(url)?;
        let last_segment_rev : String = url_decoded
            .chars()
            .rev()
            .take_while(|c| c != &'/')
            .collect();

        let last_segment = last_segment_rev.chars().rev().collect::<String>();
        let (extension, filename) = Self::extract_extension_from_filename(&last_segment);

        let url = url.to_string();
        let file_link = FileLink {
            url,
            filename,
            extension,
        };
        Ok(file_link)

    }

    pub fn extract_extension_from_filename(filename: &str) -> (Option<String>, String) {
        if filename.contains('.') {
            let after_dot_rev: String = filename.chars().rev().take_while(|c| c != &'.').collect();

            let ext: String = after_dot_rev
                .chars()
                .rev()
                .take_while(|c| c != &'?')
                .collect();

            let tmp: String = filename
                .chars()
                .rev()
                .skip(after_dot_rev.len() + 1) // after_dot_rev to exclude query params and '+ 1' for the dot
                .collect();

            
            let filename: String = tmp.chars().rev().collect();
            ( Some(ext), filename )

        } else {
            // no extension found, the file name will be used
            // sanitize as it contains query params
            // which are not allowed in filenames on some OS
            let sanitized = filename.replace(['?', '&'], "-");
            (None, sanitized)
        }
    }

}

// taken from https://github.com/bt/rust_urlencoding/blob/master/src/lib.rs#L20
fn url_decode(data: &str) -> Result<String, LeviError> {
    let mut unescaped_bytes: Vec<u8> = Vec::new();
    let mut bytes = data.bytes();
    // If validate_urlencoded_str returned Ok, then we know
    // every '%' is followed by 2 hex characters
    while let Some(b) = bytes.next() {
        match b as char {
            '%' => {
                let bytes_to_decode = &[bytes.next().unwrap(), bytes.next().unwrap()];
                let hex_str = str::from_utf8(bytes_to_decode).unwrap();
                unescaped_bytes.push(u8::from_str_radix(hex_str, 16).unwrap());
            }
            _ => {
                // Assume whoever did the encoding intended what we got
                unescaped_bytes.push(b);
            }
        }
    }
    String::from_utf8(unescaped_bytes).map_err(|e| LeviError::Url (e.to_string()))
}

#[cfg(test)]
mod file_link_tests {
    use crate::links::*;

    #[test]
    fn empty_string() {
        match FileLink::new("") {
            Err(LeviError::Url( message )) => 
            assert_eq!(
                message,
                "Url cannot be empty".to_string()
            ),
            _ => assert_eq!(true, false),
        }
    }
    
    #[test]
    fn trailing_slash() {
        let url = "https://www.google.com/area51/";
        match FileLink::new(url) {
            Err(LeviError::Url( message )) => assert_eq!(
                message,
                "Url couldn't find an extension: https://www.google.com/area51/".to_string()
            ),
            _ => assert_eq!(true, false),
        }
    }

    #[test]
    fn happy_case() {
        let url = "https://www.google.com/area51.txt";
        match FileLink::new(url) {
            Ok(fl) => {
                assert_eq!(fl.url, url);
                assert_eq!(fl.filename, "area51".to_string());
                assert_eq!(fl.extension, Some("txt".to_string()));
            }
            _ => assert_eq!(true, false),
        }
    }

    #[test]
    fn no_extension() {
        let url = "https://www.google.com/area51";
        let fl = FileLink::new(url).unwrap();
        assert_eq!(fl.extension, None);
        assert_eq!(fl.filename, "area51");
        assert_eq!(fl.url, url);
    }

    #[test]
    fn no_extension_use_query_params() {
        let url = "https://oeis.org/search?q=id:A000001&fmt=json";
        let fl = FileLink::new(url).unwrap();
        assert_eq!(fl.extension, None);
        assert_eq!(
            fl.filename,
            "search-q=id:A000001-fmt=json"
        );
        assert_eq!(fl.url, url);
    }

    #[test]
    fn extract_extension_ok() {
        let (ext, filename) = FileLink::extract_extension_from_filename("area51.txt");
        assert_eq!(filename, "area51");
        assert_eq!(ext, Some("txt".to_string()));
    }

    #[test]
    fn extract_extension_with_query_param() {
        let url = "https://releases.ubuntu.com/21.10/ubuntu-21.10-desktop-amd64.iso?id=123";
        let fl = FileLink::new(url).unwrap();
        assert_eq!(fl.extension, Some("iso".to_string()));
        assert_eq!(fl.filename, "ubuntu-21.10-desktop-amd64");
        assert_eq!(fl.url, url);
    }

    #[test]
    fn extract_extension_with_query_param_bis() {
        let url = "https://atom-installer.github.com/v1.58.0/atom-amd64.deb?s=1627025597&ext=.deb";
        let fl = FileLink::new(url).unwrap();
        assert_eq!(fl.extension, Some("deb".to_string()));
        assert_eq!(fl.url, url);
        // FIXME
        // assert_eq!(fl.filename, "atom-amd64");
    }

    #[test]
    fn extract_extension_with_parts() {
        let url = "https://www.google.com/area51/alien-archive.tar.00";
        let fl = FileLink::new(url).unwrap();
        assert_eq!(fl.extension, Some("00".to_string()));
        // TODO fix this - should be alien-archive.tar.00 or parts will collide on tmp file
        assert_eq!(fl.filename, "alien-archive.tar");
    }
}
