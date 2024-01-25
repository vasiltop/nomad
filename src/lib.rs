pub mod error;

use error::Error;
use std::{
    io::{Read, Write},
    net::TcpStream,
};
use url::Url;

use serde_json::Value;

pub struct Request {
    url: Url,
    kind: Method,
}

pub struct Response {
    pub status: u16,
    pub body: Value,
}

#[derive(Eq, PartialEq)]
pub enum Method {
    Get,
    Post,
}

impl Method {
    fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
        }
    }
}

impl Request {
    pub fn new(ip: &str) -> Result<Self, Error> {
        let url = Url::parse(ip)?;

        Ok(Request {
            url,
            kind: Method::Get,
        })
    }

    pub fn get(&self) -> Result<Response, Error> {
        self.send(None)
    }

    pub fn post(&mut self, json: Value) -> Result<Response, Error> {
        self.kind = Method::Post;
        self.send(Some(json))
    }

    fn send(&self, json: Option<Value>) -> Result<Response, Error> {
        let mut stream = TcpStream::connect(self.url.socket_addrs(|| Some(8000))?.as_slice())?;

        write!(
            stream,
            "{} {} HTTP/1.1\r\n",
            self.kind.as_str(),
            self.url.path()
        )?;

        match self.url.host_str() {
            Some(host) => write!(stream, "Host: {}\r\n", host)?,
            None => Err(Error::NoHostString)?,
        }

        write!(stream, "Content-Type: application/json\r\n")?;

        let mut body = None;
        let mut size = 0;

        if self.kind == Method::Post {
            let s = serde_json::to_string(&json)?;
            size = s.as_bytes().len();
            body = Some(s);
        }

        write!(stream, "Content-Length: {}\r\n\r\n", size)?;
        if let Some(body) = body {
            write!(stream, "{}", body)?;
        }

        let buf = &mut [0; 1024];
        let mut data = Vec::new();

        loop {
            let n = stream.read(buf)?;

            data.extend_from_slice(&buf[..n]);

            if n < 1024 {
                break;
            }
        }

        self.parse_response(data)
    }

    fn parse_response(&self, data: Vec<u8>) -> Result<Response, Error> {
        if !data.starts_with(b"HTTP/1.1 ") {
            return Err(Error::HttpParse);
        }

        let mut slice = data.as_slice();
        slice = &slice[b"HTTP/1.1 ".len()..];

        let (value, mut slice) = Self::slice_until_byte(slice, b' ');
        let status: u16 = std::str::from_utf8(value)?.parse()?;

        slice = Self::slice_until_byte(slice, b'\n').1;

        while slice[1] != b'\n' {
            slice = Self::slice_until_byte(slice, b'\n').1;
        }

        slice = Self::slice_until_byte(slice, b'\n').1;

        let body = serde_json::from_slice(slice)?;

        Ok(Response { status, body })
    }

    fn slice_until_byte(data: &[u8], byte: u8) -> (&[u8], &[u8]) {
        let mut i = 0;

        for c in data {
            if *c == byte {
                break;
            }

            i += 1;
        }

        (&data[..i], &data[i + 1..])
    }
}
