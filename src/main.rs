use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    let req = String::from_utf8_lossy(&buffer[..]).to_string();

    let uri = req.split(' ').nth(1).unwrap();

    //fs::read_dir();

    let response = format!("HTTP/1.1 200 OK\r\n\r\n {}", uri);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
