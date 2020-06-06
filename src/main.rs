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
    let mut buffer = [0; 1024];

    stream.read(&mut buffer)?;

    let req = String::from_utf8_lossy(&buffer[..]).to_string();

    println!("\n{}", req.lines().next().unwrap_or_default());
    let (request_uri, _) = parse_uri(req);

    let path_string = format!(".{}", request_uri);

    let path = Path::new(&path_string);

    let mut response: Vec<u8> = Vec::new();
    if path.is_dir() {
        response.extend_from_slice(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n",
        );
        response.extend(handle_dir(path)?);
        stream.write_all(response.as_slice())?;
    } else if path.is_file() {
        response.extend_from_slice(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/text; charset=UTF-8\r\n\r\n",
        );
        response.extend(handle_file(path)?);
        stream.write_all(response.as_slice())?;
    } else {
        response.extend_from_slice(b"HTTP/1.1 404 NOT FOUND\r\n\r\n");
        stream.write_all(response.as_slice())?;
    }

    println!(
        "{}",
        String::from_utf8_lossy(&response).lines().next().unwrap_or_default()
    );

    stream.flush()?;

    Ok(())
}

fn parse_uri(request: String) -> (String, String) {
    let request_uri: Vec<&str> = request
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .split('?')
        .collect();

    let uri = percent_decode(request_uri.get(0).unwrap());

    (
        uri,
        String::from(request_uri.get(1).unwrap_or(&"").to_owned()),
    )
}

fn percent_decode(input: &str) -> String {
    let mut chars = input.chars();

    let mut bytes: Vec<u8> = Vec::new();

    while let Some(char) = chars.next() {
        if char == '%' {
            let h = chars
                .next()
                .unwrap_or_default()
                .to_digit(16)
                .unwrap_or_default() as u8;
            let l = chars
                .next()
                .unwrap_or_default()
                .to_digit(16)
                .unwrap_or_default() as u8;
            bytes.push(h * 0x10 + l);
        } else {
            bytes.push(char as u8);
        }
    }

    String::from_utf8(bytes).unwrap_or_default()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_percent_decode_only_ascii() {
        let expect = String::from("abc");
        let actual = percent_decode("abc");

        assert_eq!(expect, actual);
    }

    #[test]
    fn test_percent_decode() {
        let expect = String::from(" !\"#$%");
        let actual = percent_decode("%20%21%22%23%24%25");

        assert_eq!(expect, actual);
    }

    #[test]
    fn test_percent_decode_mix_ascii() {
        let expect = String::from("!1#3");
        let actual = percent_decode("%211%233");

        assert_eq!(expect, actual);
    }
}
