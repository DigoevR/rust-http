#[derive(std::fmt::Debug)]
pub enum HttpError {
    UnknownMethodError(String),
    UnknownHttpVersion(String),
}
impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownMethodError(method) => write!(f, "Unknown HTTP method: {}", method),
            Self::UnknownHttpVersion(version) => write!(f, "Unknown HTTP version: {}", version),
        }
    }
}
impl std::error::Error for HttpError {}
