use core::panic;
use std::{
    error::Error,
    io::{Read, Write},
    net::TcpStream,
};
use url::Url;

use serde_json::{json, Value};

struct Request {
    url: Url,
    kind: RequestKind,
}

struct Response {
    status: u16,
    body: Value,
}

#[derive(Eq, PartialEq)]
enum RequestKind {
    Get,
    Post,
}

impl RequestKind {
    fn as_str(&self) -> &'static str {
        match self {
            RequestKind::Get => "GET",
            RequestKind::Post => "POST",
        }
    }
}

impl Request {
    fn new(ip: &str) -> Self {
        let url = Url::parse(ip).unwrap();

        Request {
            url,
            kind: RequestKind::Get,
        }
    }

    fn get(&self) -> Result<Response, Box<dyn Error>> {
        let response = self.send(None)?;

        Ok(response)
    }

    fn post(&mut self, json: Value) -> Result<Response, Box<dyn Error>> {
        self.kind = RequestKind::Post;
        let response = self.send(Some(json))?;

        Ok(response)
    }

    fn send(&self, json: Option<Value>) -> Result<Response, Box<dyn Error>> {
        let mut stream = TcpStream::connect(self.url.socket_addrs(|| Some(8000))?.as_slice())?;

        let mut request = String::new();

        request.push_str(&format!(
            "{} {} HTTP/1.1\r\n",
            self.kind.as_str(),
            self.url.path()
        ));

        match self.url.host_str() {
            Some(host) => request.push_str(&format!("Host: {} \r\n", host)),
            None => panic!("no host"),
        }

        request.push_str("Content-Type: application/json\r\n");

        let mut body = None;
        let mut size = 0;

        if self.kind == RequestKind::Post {
            let s = serde_json::to_string(&json)?;
            size = s.as_bytes().len();
            body = Some(s);
        }

        request.push_str(&format!("Content-Length: {}\r\n\r\n", size));
        if let Some(body) = body {
            request.push_str(&body);
        }

        stream.write_all(request.as_bytes())?;
        let buf = &mut [0; 1024];
        let mut data = Vec::new();

        loop {
            let n = stream.read(buf)?;

            data.extend_from_slice(&buf[..n]);

            if n < 1024 {
                break;
            }
        }

        Ok(self.parse_response(data))
    }

    fn parse_response(&self, data: Vec<u8>) -> Response {
        if !data.starts_with(b"HTTP/1.1 ") {
            panic!("could not parse")
        }

        let mut slice = data.as_slice();
        slice = &slice[b"HTTP/1.1 ".len()..];

        let (value, mut slice) = Self::slice_until_byte(slice, b' ');
        let status: u16 = std::str::from_utf8(value).unwrap().parse().unwrap();
        slice = Self::slice_until_byte(slice, b'\n').1;

        while slice[1] != b'\n' {
            slice = Self::slice_until_byte(slice, b'\n').1;
        }

        slice = Self::slice_until_byte(slice, b'\n').1;

        let body: Value = serde_json::from_slice(slice).unwrap();

        Response { status, body }
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

fn main() -> Result<(), Box<dyn Error>> {
    let res = Request::new("http://localhost:8000/test").post(json!({"test": "adadsadasda"}))?;
    //let res = Request::new("http://localhost:8000/test").get()?;

    println!("{}, {}", res.body, res.status);

    Ok(())
}
