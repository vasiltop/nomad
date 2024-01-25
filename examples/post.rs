use nomad::Request;
use serde_json::json;

fn main() {
    let response = Request::new("http://localhost:8000/test")
        .post(json!({"test": "aa"}))
        .unwrap();

    println!("{}", response.body)
}
