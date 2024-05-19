use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

use http::response::HttpStatus;

use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;

mod http;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("accepted new connection");
    let request = HttpRequest::from_stream(&stream).expect("Failed to parse the request.");
    let response = handle_request(request);
    stream.write_all(response.to_string().as_bytes()).unwrap();
}

fn handle_request(request: HttpRequest) -> HttpResponse {
    let mut response = HttpResponse::new(request.get_http_version().to_owned());

    match request.get_path() {
        "/" => {}
        "/user-agent" => {
            response.write_text(request.get_header("User-Agent").unwrap());
        }
        path if path.starts_with("/echo/") => {
            response.write_text(path.trim_start_matches("/echo/"));
        }
        _ => {
            response.set_status(HttpStatus::NotFound);
        }
    }
    response
}
