use nomad::Request;

fn main() {
    let response = Request::new("http://localhost:8000/test")
        .unwrap()
        .get()
        .unwrap();

    println!("{}", response.body)
}
