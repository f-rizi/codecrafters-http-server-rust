#[allow(unused_imports)]
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;

use codecrafters_http_server::ThreadPool;

fn main() {    
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(move || {
                    handle_Request(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    
}

fn handle_Request(mut stream: TcpStream) {
    let request = get_request_lines(&mut stream);

    // Ensure the first line is present
    let parts = match request.get(0) {
        Some(line) => line.trim().split_whitespace().collect::<Vec<&str>>(),
        None => {
            let response = "HTTP/1.1 400 Bad Request\r\n\r\nMalformed Request";
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
    };

    // Ensure the request line has enough parts
    if parts.len() < 2 {
        let response = "HTTP/1.1 400 Bad Request\r\n\r\nMalformed Request";
        stream.write_all(response.as_bytes()).unwrap();
        return;
    }

    let path = parts[1];

    if path.starts_with("/user-agent") {
        handle_user_agent_request(request, &mut stream);
    } else if path.starts_with("/echo/") {
        handle_echo_request(path.to_string(), &mut stream);
    } else if path.starts_with("/sleep") {
        handle_sleep_request(&mut stream);
    } else if path == "/" {
        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
    } else {
        stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
    }
}


fn handle_Request2(stream: &mut TcpStream) {
    let request = get_request_lines(stream);
    let parts: Vec<&str> = request.get(0).unwrap().trim().split_whitespace().collect();

    let path = parts[1];

    if path.starts_with("/user-agent") {
        handle_user_agent_request(request, stream);
    }
    else if path.starts_with("/echo/") {
        handle_echo_request(path.to_string(), stream);
    }
    else if path.starts_with("/sleep") {
        handle_sleep_request(stream);
    }
    else if path.eq("/") {
        stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
    }
    else {
        stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
    }
}

fn handle_sleep_request(stream: &mut TcpStream) {
    thread::sleep(Duration::from_secs(5));
    stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
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
