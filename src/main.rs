use core::panic;
use std::{
    error::Error,
    io::{Read, Write},
    net::TcpStream,
};

use serde_json::{json, Value};

struct Request<'a> {
    ip: &'a str,
}

struct Response {
    status: u16,
    body: Value,
}

impl<'a> Request<'a> {
    fn new(ip: &'a str) -> Self {
        Request { ip }
    }

    fn get(&self, json: Value) -> Result<Response, Box<dyn Error>> {
        let mut stream = TcpStream::connect(self.ip)?;

        let mut request = String::new();

        request.push_str("POST /test HTTP/1.1\r\n");
        request.push_str(&format!("Host: {} \r\n", self.ip));
        request.push_str("Content-Type: application/json\r\n");

        let body = &serde_json::to_string(&json)?;
        let size = body.as_bytes().len();
        request.push_str(&format!("Content-Length: {}\r\n\r\n", size));
        request.push_str(body);

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
    let response = Request::new("127.0.0.1:8000").get(json!({"test": "hello"}))?;

    println!("{:?}", response.body);

    Ok(())
}
