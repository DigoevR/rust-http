use std::io::Write;
use std::net::TcpListener;

enum HttpVersion {
    Http1_1,
}

enum HttpStatus {
    Ok,
}

struct HttpResponseStatusLine {
    version: HttpVersion,
    status: HttpStatus,
}

impl HttpResponseStatusLine {
    fn new(version: HttpVersion, status: HttpStatus) -> Self {
        HttpResponseStatusLine { version, status }
    }

    fn to_string(&self) -> String {
        match self.version {
            HttpVersion::Http1_1 => "HTTP/1.1",
        }
        .to_string()
            + " "
            + match self.status {
                HttpStatus::Ok => "200 OK",
            }
    }
}

struct HttpResponse {
    status_line: HttpResponseStatusLine,
    content: String,
}

impl HttpResponse {
    fn new(version: HttpVersion, status: HttpStatus, content: String) -> Self {
        HttpResponse {
            status_line: HttpResponseStatusLine::new(version, status),
            content,
        }
    }

    fn to_string(&self) -> String {
        let mut response = self.status_line.to_string() + "\r\n";

        response += "\r\n";
        response += &self.content;
        response
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let response =
                    HttpResponse::new(HttpVersion::Http1_1, HttpStatus::Ok, "".to_string());
                stream.write_all(response.to_string().as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
