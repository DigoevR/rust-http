pub mod errors;
pub mod request;
pub mod response;

use errors::HttpError;

#[derive(Clone)]
pub enum HttpVersion {
    Http1_0,
    Http1_1,
    Http2_0,
}

impl HttpVersion {
    fn from_str(str: &str) -> Result<Self, HttpError> {
        match str {
            "HTTP/1.0" => Ok(Self::Http1_0),
            "HTTP/1.1" => Ok(Self::Http1_1),
            "HTTP/2.0" => Ok(Self::Http2_0),
            _ => Err(HttpError::UnknownHttpVersion(str.to_owned())),
        }
    }
}
