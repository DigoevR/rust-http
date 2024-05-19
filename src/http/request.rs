use std::{
    io::{BufReader, Error, ErrorKind, Read},
    net::TcpStream,
};

use super::{errors::HttpError, HttpVersion};

#[derive(Clone)]
pub enum HttpRequestMethod {
    Get,
    Post,
    Delete,
    Patch,
    Put,
}
impl HttpRequestMethod {
    fn from_str(str: &str) -> Result<Self, HttpError> {
        match str {
            "GET" => Ok(HttpRequestMethod::Get),
            "POST" => Ok(HttpRequestMethod::Post),
            "DELETE" => Ok(HttpRequestMethod::Delete),
            "PATCH" => Ok(HttpRequestMethod::Patch),
            "PUT" => Ok(HttpRequestMethod::Put),

            _ => Err(HttpError::UnknownMethodError(str.to_string())),
        }
    }
}

struct HttpRequestLine {
    version: HttpVersion,
    target: String,
    method: HttpRequestMethod,
}

impl HttpRequestLine {
    fn new(version: HttpVersion, target: String, method: HttpRequestMethod) -> Self {
        Self {
            version,
            target,
            method,
        }
    }

    fn from_stream(stream: &mut BufReader<&TcpStream>) -> Result<Self, Box<dyn std::error::Error>> {
        let buffer = parse_stream_untill_sequence(stream, b"\r\n")?;
        let mut words = buffer.split(|byte| byte == &b' ');

        let mut method = String::new();
        let mut target = String::new();
        let mut version = String::new();

        words.next().unwrap().read_to_string(&mut method)?;
        words.next().unwrap().read_to_string(&mut target)?;
        words.next().unwrap().read_to_string(&mut version)?;

        let method = HttpRequestMethod::from_str(&method)?;
        let version = HttpVersion::from_str(&version)?;
        Ok(Self::new(version, target, method))
    }
}

fn parse_stream_untill_sequence(
    stream: &mut BufReader<&TcpStream>,
    sequence: &[u8],
) -> Result<Vec<u8>, Error> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut sequence_buffer: Vec<u8> = Vec::with_capacity(sequence.len());

    for byte in stream.bytes() {
        let byte = byte?;

        buffer.push(byte);
        sequence_buffer.push(byte);

        if sequence_buffer.len() > sequence.len() {
            sequence_buffer.remove(0);
        }

        if sequence_buffer == sequence {
            buffer.truncate(buffer.len() - sequence.len());

            return Ok(buffer);
        }
    }
    return Err(Error::new(ErrorKind::Other, "No sequence found in stream"));
}

pub struct HttpRequest {
    request_line: HttpRequestLine,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl HttpRequest {
    pub fn from_stream(stream: &TcpStream) -> Result<Self, Box<dyn std::error::Error>> {
        let mut stream = BufReader::new(stream);

        let request_line = HttpRequestLine::from_stream(&mut stream)?;
        let mut headers = Vec::new();
        let header_str = String::from_utf8(parse_stream_untill_sequence(&mut stream, b"\r\n")?)?;
        let header_str = header_str.trim();
        if header_str.len() > 0 {
            let mut parts = header_str.split(": ");
            let header_name = parts.next().unwrap();
            let header_value = parts.next().unwrap();
            headers.push((header_name.to_string(), header_value.to_string()));
        }
        loop {
            let header_str =
                String::from_utf8(parse_stream_untill_sequence(&mut stream, b"\r\n")?)?;
            let header_str = header_str.trim();
            if header_str.len() == 0 {
                break;
            }
            let mut parts = header_str.split(": ");
            let header_name = parts.next().unwrap();
            let header_value = parts.next().unwrap();
            headers.push((header_name.to_string(), header_value.to_string()));
        }
        Ok(Self {
            request_line,
            headers,
            body: None,
        })
    }

    pub fn get_header(&self, header_name: &str) -> Option<&String> {
        self.headers
            .iter()
            .find(|(name, _)| header_name == name)
            .and_then(|(_, value)| Some(value))
    }

    pub fn get_path(&self) -> &str {
        &self.request_line.target
    }

    pub fn get_method(&self) -> &HttpRequestMethod {
        &self.request_line.method
    }

    pub fn get_http_version(&self) -> &HttpVersion {
        &self.request_line.version
    }
}
