use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process;

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

fn parse_ages(mut args: env::Args) -> Option<i32> {
    let port: i32 = args.nth(1)?.parse().ok()?;
    Some(port)
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 512];

    stream.read_exact(&mut buffer)?;

    let req = String::from_utf8_lossy(&buffer[..]).to_string();

    let request_uri = req.split_whitespace().nth(1).unwrap_or("/");

    let path_string = format!(".{}", request_uri);

    let path = Path::new(&path_string);

    let mut response: Vec<u8> = Vec::new();
    if path.is_dir() {
        response.extend_from_slice(b"HTTP/1.1 200 OK\r\n\r\n");
        response.extend(handle_dir(path)?);
        stream.write_all(response.as_slice())?;
    } else if path.is_file() {
        response.extend_from_slice(b"HTTP/1.1 200 OK\r\n\r\n");
        response.extend(handle_file(path)?);
        stream.write_all(response.as_slice())?;
    } else {
        stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n")?;
    }

    stream.flush()?;

    Ok(())
}

fn handle_dir(path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut result = String::new();
    let dir_name = path
        .to_string_lossy()
        .to_string()
        .trim_start_matches('.')
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
        &dir_name
    ));

    if dir_name != "/" {
        result.push_str(&format!(
            "<tr><td><a href=\"{}../\">../</a></td><td></td></tr>",
            &dir_name
        ));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap();
        let metadata = entry.metadata()?;
        let file_size = metadata.len();

        result.push_str("<tr>");

        if metadata.is_dir() {
            result.push_str(&format!(
                "<td><a href=\"{}{}/\">{}/</a></td>",
                &dir_name, &name, &name
            ));
            result.push_str("<td></td>");
        } else if metadata.is_file() {
            result.push_str(&format!(
                "<td><a href=\"{}{}\">{}</a></td>",
                &dir_name, &name, &name
            ));
            result.push_str(&format!("<td>{} Bytes</td>", file_size));
        }

        result.push_str("</tr>");
    }

    result.push_str("</tbody></table></body>");

    Ok(result.into_bytes())
}

fn handle_file(path: &Path) -> Result<Vec<u8>, io::Error> {
    fs::read(path)
}
