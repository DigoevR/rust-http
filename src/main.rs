use std::io::{BufReader, Error, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(std::fmt::Debug)]
enum HttpError {
    UnknownMethodError,
    UnknownHttpVersion(String),
}
impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownMethodError => write!(f, "Unknown HTTP method"),
            Self::UnknownHttpVersion(version) => write!(f, "Unknown HTTP version: {}", version),
        }
    }
}
impl std::error::Error for HttpError {}

#[derive(Clone)]
enum HttpVersion {
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

enum HttpStatus {
    Ok,
    NotFound,
}

struct HttpResponseStatusLine {
    version: HttpVersion,
    status: HttpStatus,
}

impl HttpResponseStatusLine {
    fn new(version: HttpVersion, status: HttpStatus) -> Self {
        Self { version, status }
    }

    fn to_string(&self) -> String {
        match self.version {
            HttpVersion::Http1_0 => "HTTP/1.0",
            HttpVersion::Http1_1 => "HTTP/1.1",
            HttpVersion::Http2_0 => "HTTP/2.0",
        }
        .to_string()
            + " "
            + match self.status {
                HttpStatus::Ok => "200 OK",
                HttpStatus::NotFound => "404 Not Found",
            }
    }
}

struct HttpResponse {
    status_line: HttpResponseStatusLine,
    headers: Vec<(String, String)>,
    content: String,
}

impl HttpResponse {
    fn new(version: HttpVersion) -> Self {
        Self {
            status_line: HttpResponseStatusLine::new(version, HttpStatus::Ok),
            content: "".to_owned(),
            headers: Vec::new(),
        }
    }

    fn add_header(&mut self, header_name: String, header_value: String) -> &mut Self {
        self.headers.push((header_name, header_value));
        self
    }

    fn set_status(&mut self, status: HttpStatus) -> &mut Self {
        self.status_line.status = status;
        self
    }

    fn add_content(&mut self, content: String) -> &mut Self {
        self.content = content;
        self
    }

    fn to_string(&self) -> String {
        let mut response = self.status_line.to_string() + "\r\n";

        for (name, value) in &self.headers {
            response += &format!("{name}: {value}\r\n",)
        }

        response += "\r\n";
        response += &self.content;
        response
    }
}

enum HttpRequestMethod {
    Get,
}
impl HttpRequestMethod {
    fn from_str(str: &str) -> Result<Self, HttpError> {
        match str {
            "GET" => Ok(HttpRequestMethod::Get),
            _ => Err(HttpError::UnknownMethodError),
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

struct HttpRequest {
    request_line: HttpRequestLine,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl HttpRequest {
    fn from_stream(stream: &mut BufReader<&TcpStream>) -> Result<Self, Box<dyn std::error::Error>> {
        let request_line = HttpRequestLine::from_stream(stream)?;
        let mut headers = Vec::new();
        let header_str = String::from_utf8(parse_stream_untill_sequence(stream, b"\r\n")?)?;
        let header_str = header_str.trim();
        if header_str.len() > 0 {
            let mut parts = header_str.split(": ");
            let header_name = parts.next().unwrap();
            let header_value = parts.next().unwrap();
            headers.push((header_name.to_string(), header_value.to_string()));
        }
        loop {
            let header_str = String::from_utf8(parse_stream_untill_sequence(stream, b"\r\n")?)?;
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
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buf = BufReader::new(&stream);
                let request =
                    HttpRequest::from_stream(&mut buf).expect("Failed to parse the request.");

                let response = HttpResponse::new(request.request_line.version.clone());
                let response = handle_request(request, response);

                stream.write_all(response.to_string().as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_request(request: HttpRequest, mut response: HttpResponse) -> HttpResponse {
    let target = &request.request_line.target[..];
    match target {
        "/" => {}
        target if target.starts_with("/echo/") => {
            let message = target.trim_start_matches("/echo/");
            response
                .add_header("Content-Type".to_string(), "text/plain".to_string())
                .add_header("Content-Length".to_string(), message.len().to_string())
                .add_content(message.to_string());
        }
        _ => {
            response.set_status(HttpStatus::NotFound);
        }
    }
    response
}
