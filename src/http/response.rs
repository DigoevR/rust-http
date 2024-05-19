use super::HttpVersion;

pub enum HttpStatus {
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

pub struct HttpResponse {
    status_line: HttpResponseStatusLine,
    headers: Vec<(String, String)>,
    content: String,
}

impl HttpResponse {
    pub fn new(version: HttpVersion) -> Self {
        Self {
            status_line: HttpResponseStatusLine::new(version, HttpStatus::Ok),
            content: "".to_owned(),
            headers: Vec::new(),
        }
    }

    pub fn add_header(&mut self, header_name: &str, header_value: &str) -> &mut Self {
        self.headers
            .push((header_name.to_string(), header_value.to_string()));
        self
    }

    pub fn set_status(&mut self, status: HttpStatus) -> &mut Self {
        self.status_line.status = status;
        self
    }

    pub fn add_content(&mut self, content: &str) -> &mut Self {
        self.content = content.to_string();
        self
    }

    pub fn write_text(&mut self, text: &str) -> &mut Self {
        self.add_header("Content-Type", "text/plain")
            .add_header("Content-Length", &text.len().to_string())
            .add_content(text)
    }

    pub fn to_string(&self) -> String {
        let mut response = self.status_line.to_string() + "\r\n";

        for (name, value) in &self.headers {
            response += &format!("{name}: {value}\r\n",)
        }

        response += "\r\n";
        response += &self.content;
        response
    }
}
