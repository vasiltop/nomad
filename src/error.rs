use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Seralize(serde_json::error::Error),
    Resolve(std::io::Error),
    Utf8(std::str::Utf8Error),
    ParseInt(std::num::ParseIntError),
    ParseUrl(url::ParseError),
    NoHostString,
    HttpParse,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::Seralize(e) => format!("Seralize error: {}", e),
            Error::Resolve(e) => format!("Resolve error: {}", e),
            Error::Utf8(e) => format!("Utf8 error: {}", e),
            Error::ParseInt(e) => format!("ParseInt error: {}", e),
            Error::ParseUrl(e) => format!("ParseUrl error: {}", e),
            Error::NoHostString => "No host string".to_string(),
            Error::HttpParse => "Http parse error".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(value: serde_json::error::Error) -> Self {
        Error::Seralize(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Resolve(value)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Error::Utf8(value)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Error::ParseInt(value)
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Error::ParseUrl(value)
    }
}
