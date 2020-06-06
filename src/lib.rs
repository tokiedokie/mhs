use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

pub fn parse_ages(mut args: env::Args) -> Option<i32> {
    let port: i32 = args.nth(1)?.parse().ok()?;
    Some(port)
}

pub fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer)?;

    let req = String::from_utf8_lossy(&buffer[..]).to_string();

    println!("\n{}", req.lines().next().unwrap_or_default());
    let (request_uri, _) = parse_request(req);

    let path_string = format!(".{}", request_uri);

    let path = Path::new(&path_string);

    let response: Vec<u8> = if path.is_dir() {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n");
        buf.extend(handle_dir(path)?);
        buf
    } else if path.is_file() {
        let mut buf: Vec<u8> = Vec::new();
        let header = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: {}; charset=UTF-8\r\n\r\n",
            determine_content_type(path)
        );
        buf.extend_from_slice(header.as_bytes());
        buf.extend(handle_file(path)?);
        buf
    } else {
        b"HTTP/1.1 404 NOT FOUND\r\n\r\n".to_vec()
    };

    stream.write_all(&response)?;

    println!(
        "{}",
        String::from_utf8_lossy(&response)
            .lines()
            .next()
            .unwrap_or_default()
    );

    stream.flush()?;

    Ok(())
}

/**
parse http request

returns (uri, query)
*/
fn parse_request(request: String) -> (String, String) {
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

fn determine_content_type<'a>(path: &'a Path) -> &'a str {
    match path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
    {
        // content-type is from [Common MIME types - HTTP | MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types)
        "aac" => "audio/aac",
        "abw" => "application/x-abiword",
        "arc" => "application/x-freearc",
        "avi" => "video/x-msvideo",
        "azw" => "application/vnd.amazon.ebook",
        "bin" => "application/octet-stream",
        "bmp" => "image/bmp",
        "bz" => "application/x-bzip",
        "bz2" => "application/x-bzip2",
        "csh" => "application/x-csh",
        "css" => "text/css",
        "csv" => "text/csv",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "eot" => "application/vnd.ms-fontobject",
        "epub" => "application/epub+zip",
        "gz" => "application/gzip",
        "gif" => "image/gif",
        "htm" => "text/html",
        "html" => "text/html",
        "ico" => "image/vnd.microsoft.icon",
        "ics" => "text/calender",
        "jar" => "application/java-archive",
        "jpeg" => "image/jpeg",
        "jpg" => "image/jpeg",
        "js" => "text/javascript",
        "json" => "application/json",
        "jsonld" => "application/ld+json",
        "mid" => "audio/midi",
        "midi" => "audio/midi",
        "mjs" => "text/javascript",
        "mp3" => "audio/mpeg",
        "mpeg" => "video/mpeg",
        "mpkg" => "application/vnd.apple.installer+xml",
        "odp" => "application/vnd.oasis.opendocument.presentation",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "odt" => "application/vnd.oasis.opendocument.text",
        "oga" => "audio/ogg",
        "ogv" => "video/ogg",
        "ogx" => "application/ogg",
        "opus" => "audio/opus",
        "otf" => "font/otf",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "php" => "application/x-httpd-php",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "rar" => "application/vnd.rar",
        "rtf" => "application/rtf",
        "sh" => "application/x-sh",
        "svg" => "image/svg+xml",
        "swf" => "application/x-shockwave-flash",
        "tar" => "application/x-tar",
        "tif" => "image/tiff",
        "tiff" => "image/tiff",
        "ts" => "video/mp2t",
        "ttf" => "font/ttf",
        "txt" => "text/plane",
        "vsd" => "application/vnd.visio",
        "wav" => "audio/wav",
        "weba" => "	audio/webm",
        "webm" => "video/webm",
        "webp" => "audio/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "xhtml" => "application/xhtml+xml",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xml" => "text/xml",
        "xul" => "application/vnd.mozilla.xul+xml",
        "zip" => "application/zip",
        "3gp" => "video/3gpp",
        "3g2" => "video/3gpp2",
        "7z" => "application/x-7z-compressed",
        
        //below is original
        "md" => "text/markdown",
        _ => "application/octet-stream",
    }
}

fn handle_file(path: &Path) -> Result<Vec<u8>, io::Error> {
    fs::read(path)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_percent_decode() {
        let expect = String::from(" !\"#$%");
        let actual = percent_decode("%20%21%22%23%24%25");

        assert_eq!(expect, actual);
    }

    #[test]
    fn test_percent_decode_only_ascii() {
        let expect = String::from("abc");
        let actual = percent_decode("abc");

        assert_eq!(expect, actual);
    }

    #[test]
    fn test_percent_decode_mix_ascii() {
        let expect = String::from("!1#3");
        let actual = percent_decode("%211%233");

        assert_eq!(expect, actual);
    }

    #[test]
    fn test_handle_file() {
        let path = Path::new(".gitignore");

        let expected = fs::read(path).unwrap_or_default();
        let actual = handle_file(path).unwrap_or_default();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_determine_content_type() {
        let path = Path::new("test.txt");

        let expect = "text/plane";
        let actual = determine_content_type(&path);

        assert_eq!(expect, actual);
    }
}
