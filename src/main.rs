use std::collections::HashMap;
#[allow(unused_imports)]
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Read};
use std::thread;
use std::time::Duration;

use codecrafters_http_server::ThreadPool;
use std::env;
use std::fs;
use std::fs::File;

extern crate libflate;

use std::io;
use libflate::gzip::Decoder;

use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;

use base64;


fn main() {    
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(move || {
                    handle_request(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_request(mut stream: TcpStream) {
    let request =  get_request_lines(&mut stream);

    let method = request.0.get("method").unwrap();
    let path = request.0.get("path").unwrap();

    if method.eq("GET") {
        handle_get_request(path.to_string(), stream, request.0, request.1);
    }
    else if method.eq("POST") {
        handle_post_request(path.to_string(), stream , request.0, request.1);
    }
}

fn handle_post_request(path: String, mut stream: TcpStream, request: HashMap<String, String>, body: Vec<u8>) {
    if path.starts_with("/files"){
        handle_save_files(path.to_string(), &mut stream, body);

    } else if path == "/" {
        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
    } else {
        stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
    }
}

fn handle_save_files(path: String, stream: &mut TcpStream,  body: Vec<u8>) {
    let parts = path.split("/").collect::<Vec<&str>>();

    if parts.len() < 3 {
        let response = format!("HTTP/1.1 404 Not Found\r\n\r\n");        
        stream.write(response.as_bytes()).unwrap();    
        return;
    }
    let file_path = format!("/tmp/data/codecrafters.io/http-server-tester/{}", parts[2]);

    let mut file = fs::File::create(file_path).unwrap();
    file.write_all(&body).unwrap();
    stream.write_all(b"HTTP/1.1 201 Created\r\n\r\n").unwrap();
}

fn handle_get_request(path: String, mut stream: TcpStream , request: HashMap<String, String>, body: Vec<u8> ) {
    if path.starts_with("/user-agent") {
        handle_user_agent_request(request, &mut stream);
    } else if path.starts_with("/echo/") {
        handle_echo_request(path.to_string(), &mut stream, request, body);
    } else if path.starts_with("/sleep") {
        handle_sleep_request(&mut stream);
    }
    else if path.starts_with("/files"){
        handle_read_file(path.to_string(), &mut stream);

    } else if path == "/" {
        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
    } else {
        stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
    }
}

fn handle_read_file(path: String,stream: &mut TcpStream) {
    let parts = path.split("/").collect::<Vec<&str>>();

    if parts.len() < 3 {
        let response = format!("HTTP/1.1 404 Not Found\r\n\r\n");        
        stream.write(response.as_bytes()).unwrap();    
        return;
    }

    let file_path = format!("/tmp/data/codecrafters.io/http-server-tester/{}", parts[2]);
    let file_open_result = File::open(file_path);
    let mut contents = String::new();

    match file_open_result {
        Ok(mut file) => {
            file.read_to_string(&mut contents);
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", contents.chars().count(), contents);        
            stream.write(response.as_bytes()).unwrap();
        },
    
        Err(error) => {
            let response = format!("HTTP/1.1 404 Not Found\r\n\r\n");        
            stream.write(response.as_bytes()).unwrap();    
        },
    }   
}

fn handle_sleep_request(stream: &mut TcpStream) {
    thread::sleep(Duration::from_secs(5));
    stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
}

fn handle_user_agent_request(request: HashMap<String, String>, stream: &mut TcpStream) {
    let agent: String = request
    .get("User-Agent")
    .map(|value| value.to_string()) // Convert the value if it exists
    .unwrap_or_else(|| String::from(""));

    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", agent.len(), agent);        
    stream.write(response.as_bytes()).unwrap();
}

fn handle_echo_request(path: String, stream: &mut TcpStream, request: HashMap<String, String>, body: Vec<u8>) {
    let temp = path.strip_prefix("/echo/");

    if request.contains_key("Accept-Encoding") {
        let encoding = request.get("Accept-Encoding").unwrap();
        let encodings = encoding.split(", ").collect::<Vec<&str>>();

        if encodings.contains(&"gzip") {
            println!("Gzip encoding is present!");
            let temp_byte = temp.unwrap().as_bytes();

            // let mut input = io::stdin();
            // let mut decoder = Decoder::new(&mut input).unwrap();
            // io::copy(&mut decoder, &mut io::stdout()).unwrap();
 
            let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
            e.write_all(temp_byte).expect("Failed to write to encoder");
            let compressed_bytes = e.finish().expect("Failed to finish compression");

            // Convert the compressed bytes to a Base64 string
            let as_string = base64::encode(compressed_bytes);

            let response = 
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n{}", 
            as_string.len(), 
            as_string);   
            stream.write(response.as_bytes()).unwrap();
     
        }
        else {
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n");   
            stream.write(response.as_bytes()).unwrap();
        }
    }
    else {
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", temp.unwrap().len(), temp.unwrap());        
        stream.write(response.as_bytes()).unwrap();
    }
}

fn get_request_lines(stream: &mut TcpStream) -> (HashMap<String, String>, Vec<u8>) {
    let mut buf_reader = BufReader::new( stream);
    let request_lines: Vec<_> = buf_reader
        .by_ref()
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let mut request_parts: HashMap<String, String> = HashMap::new();

    for (pos, line) in request_lines.iter().enumerate(){
        if pos == 0 {

            let parts = line.trim().split_whitespace().collect::<Vec<&str>>();
            request_parts.insert(String::from("method"), String::from(parts[0]));
            request_parts.insert(String::from("path"), String::from(parts[1]));
        }
        else {
            let parts = line.trim().split(": ").collect::<Vec<&str>>();

            if parts.len() == 2 {
                let key = String::from(parts[0]);
                let value = String::from(parts[1]);
                request_parts.insert(key, value);
            }
        }
    }

    let mut body = Vec::new();

    if request_parts.contains_key("Content-Length") {
        let content_length = request_parts.get("Content-Length").unwrap().to_string();
        let length = content_length.parse::<usize>().unwrap_or(0);
        if length > 0 {
            let mut buffer = vec![0; length];
            buf_reader.read_exact(&mut buffer).unwrap();
            body = buffer;
        }
    }

    return (request_parts, body);
}
