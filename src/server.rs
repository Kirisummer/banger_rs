use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream, SocketAddr};
use std::sync::Arc;
use std::thread;

use crate::bang_storage::BangStorage;
use crate::request::{QueryErr, parse_query};
use crate::response::{Response, StatusCode, encode};

fn process_query(
    storage: &BangStorage,
    query: Vec<String>,
    encoder: &dyn Fn(&str) -> String,
) -> String {
    let mut bang_opt = None;
    let mut query_parts = vec![];

    for token in query.iter().filter(|line| !line.is_empty()) {
        if token.len() == 0 {
            continue;
        }

        let mut token_it = token.chars();
        let (first, rest) = (token_it.nth(0).unwrap(), token_it.as_str());
        if bang_opt == None && first == '!' && storage.bangs.contains_key(rest) {
            bang_opt = Some(rest);
        } else {
            query_parts.push(token.to_string());
        }
    }

    let bang = match bang_opt {
        Some(value) => value,
        None => &storage.default,
    };

    let encoded: Vec<String> = query_parts.iter().map(|part| encoder(&part)).collect();
    storage.bangs[bang].replace("{}", &encoded.join("+"))
}

fn process_request(storage: &BangStorage, request: &str) -> String {
    const PROTO: &str = "HTTP/1.1";
    match parse_query(&request) {
        Ok(query) => {
            let response_url = process_query(&storage, query, &encode);
            let mut response = Response::new(PROTO, StatusCode::SeeOther);
            response.header("Location", &response_url);
            response
        }
        Err(err) => match err {
            QueryErr::BadRequest(err) => {
                let mut response = Response::new(PROTO, StatusCode::BadRequest);
                response.header("Content-Type", "text/plain");
                response.body(&err);
                response
            }
            QueryErr::MethodNotAllowed => {
                let mut response = Response::new(PROTO, StatusCode::MethodNotAllowed);
                response.header("Allow", "GET, HEAD");
                response
            }
        },
    }
    .make()
}

fn read_all(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    const BUFFER_SIZE: usize = 512;
    let mut buffer = [0; BUFFER_SIZE];
    let mut result = Vec::new();
    loop {
        match stream.read(&mut buffer) {
            Ok(read_count) => {
                result.extend_from_slice(&buffer[..read_count]);
                if read_count < BUFFER_SIZE {
                    break;
                }
            }
            Err(err) => {
                if err.kind() == ErrorKind::WouldBlock {
                    continue;
                } else {
                    return Err(err.to_string());
                }
            }
        }
    }
    Ok(result)
}

fn shutdown(stream: &mut TcpStream) -> () {
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Failed to shutdown stream: {err}")
        }
    }
}

fn serve_one(storage: Arc<BangStorage>, mut stream_arc: Arc<TcpStream>) -> () {
    let mut stream = Arc::<TcpStream>::get_mut(&mut stream_arc).unwrap();
    // Read
    let request_bytes = match read_all(&mut stream) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Failed to read from stream: {err}");
            shutdown(&mut stream);
            return;
        }
    };

    // Process
    let request = match String::from_utf8(request_bytes) {
        Ok(v) => v,
        Err(_) => "".to_string(),
    };
    eprintln!("Got request: {:?}", request);
    let response = process_request(&storage, &request);
    eprintln!("Made response: {:?}", response);
    let response_bytes = response.as_bytes();

    // Write
    match stream.write(response_bytes) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Failed to write to stream: {err}");
            shutdown(&mut stream);
            return;
        }
    }

    // Close
    shutdown(&mut stream);
}

pub fn serve(storage: BangStorage, address: SocketAddr) -> Result<(), String> {
    let storage_arc = Arc::new(storage);
    let listener = TcpListener::bind(address).map_err(|err| format!("{err}"))?;
    for stream_res in listener.incoming() {
        let stream_arc = match stream_res {
            Ok(stream) => Arc::new(stream),
            Err(err) => {
                eprintln!("Failed to accept connection: {err}");
                continue;
            }
        };
        match stream_arc.set_nonblocking(true) {
            Ok(_) => (),
            Err(err) => {
                return Err(format!("Failed to make socket non-blocking: {err}"));
            }
        }

        let storage_cl = storage_arc.clone();
        let stream_cl = stream_arc.clone();
        let _ = thread::spawn(|| serve_one(storage_cl, stream_cl));
    }

    Ok(())
}
