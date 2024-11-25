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
    let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);
    handle_Request(&mut stream);
}

fn handle_Request(stream: &mut TcpStream) {
    let request = get_request_lines(stream);
    let parts: Vec<&str> = request.get(0).unwrap().trim().split_whitespace().collect();

    let path = parts[1];

    if path.starts_with("/user-agent") {
        handle_user_agent_request(request, stream);
    }
    else if path.starts_with("/echo/") {
        handle_echo_request(path.to_string(), stream);
    }
    else if path.eq("/") {
        stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
    }
    else {
        stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
    }
}

fn handle_user_agent_request(request: Vec<String>, stream: &mut TcpStream) {
    let mut agent: String = String::from("");

    for item in request {
        if item.starts_with("User-Agent") {
            // Split into parts and collect into a vector
            let parts: Vec<&str> = item.splitn(2, ": ").collect();
            if parts.len() > 1 {
                agent = parts[1].to_string(); // Create an owned String
            }
        }
    }

    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", agent.len(), agent);        
    stream.write(response.as_bytes()).unwrap();
}

fn handle_echo_request(path: String, stream: &mut TcpStream) {
    let temp = path.strip_prefix("/echo/");
    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", temp.unwrap().len(), temp.unwrap());        
    stream.write(response.as_bytes()).unwrap();
}

fn get_request_lines(stream: &mut TcpStream) -> Vec<String> {
    let buf_reader = BufReader::new( stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    return http_request;
}
