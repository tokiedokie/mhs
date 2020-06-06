use std::env;
use std::net::TcpListener;
use std::process;

use mhs::*;

fn main() {
    // default port is 7878
    let port = parse_ages(env::args()).unwrap_or(7878);

    let listener = TcpListener::bind(&format!("127.0.0.1:{}", port)).unwrap_or_else(|_| {
        eprintln!("Can't open port: {}", port);
        process::exit(1);
    });

    println!("mhs has started");
    println!("http://127.0.0.1:{}", port);
    println!("http://localhost:{}", port);

    for stream in listener.incoming() {
        let stream = stream.unwrap_or_else(|err| {
            eprintln!("error: {}", err);
            process::exit(1);
        });

        handle_connection(stream).unwrap_or_else(|err| {
            eprintln!("error: {}", err);
        });
    }
}
