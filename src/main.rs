use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;

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

    let uri = req.split(' ').nth(1).unwrap_or_default();

    let path_string = format!(".{}", uri);

    let path = Path::new(&path_string);

    let response = if path.is_dir() {
        format!("HTTP/1.1 200 OK\r\n\r\n{}", handle_dir(path))
    } else if path.is_file() {
        format!("HTTP/1.1 200 OK\r\n\r\n{}", handle_file(path))
    } else {
        String::from("HTTP/1.1 404 NOT FOUND\r\n\r\n")
    };

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_dir(path: &Path) -> String {
    let mut result = String::new();
    let dir_name = path
        .to_string_lossy()
        .to_string()
        .trim_start_matches(".")
        .to_string();

    result.push_str(&format!(
        "\
        <head>\
            <title>Index of {}</title>\
        </head>\
    ",
        &dir_name
    ));

    result.push_str(&format!(
        "<body><h1>Index of {}</h1><table><tbody>",
        dir_name
    ));

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        let metadata = entry.metadata().unwrap();
        let file_size = metadata.len();

        result.push_str("<tr>");

        if metadata.is_dir() {
            result.push_str("<td></td>");
            result.push_str(&format!(
                "<td><a href=\"{}{}/\">{}/</a></td>",
                &dir_name, &name, &name
            ));
        } else if metadata.is_file() {
            result.push_str(&format!("<td>{} Bytes</td>", file_size));
            result.push_str(&format!(
                "<td><a href=\"{}{}\">{}</a></td>",
                &dir_name, &name, &name
            ));
        }

        result.push_str("</tr>");
    }

    result.push_str("</tbody></table></body>");

    result
}

fn handle_file(path: &Path) -> String {
    String::from_utf8(fs::read(path).unwrap()).unwrap()
}
