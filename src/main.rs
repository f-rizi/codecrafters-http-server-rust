#[allow(unused_imports)]
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader};

fn main() {    
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();

    if let Err(e) = reader.read_line(&mut request_line) {
        eprintln!("Failed to read from stream: {}", e);
        return;
    }

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 3 {
        eprintln!("Invalid HTTP request line: {}", request_line);
        return;
    }

    let method = parts[0]; 
    let path = parts[1]; 

    println!("Method: {}, Path: {}", method, path);

    if path.starts_with("/echo/") {
        let temp = path.strip_prefix("/echo/");
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", temp.unwrap().len(), temp.unwrap());        
        stream.write(response.as_bytes()).unwrap();
    }
    else if path.starts_with("/") {
        stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();

    }
    else {
        
        stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
    }
}
