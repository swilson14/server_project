use server_project::ThreadPool;
use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(socket) => socket,
        Err(error) => panic!("TCP listener failed to bind to 127.0.0.1:7878 {error}"),
    };

    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(3) {
        let stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("Failed to accept connection: {error}");
                continue;
            }
        };

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = match buf_reader.lines().next() {
        Some(Ok(request)) => request,
        Some(Err(error)) => {
            eprintln!("Failed to read request {error}");
            return;
        }
        None => return,
    };

    println!("{request_line}");

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let (status_line, contents) = match fs::read_to_string(filename) {
        Ok(contents) => (status_line, contents),
        Err(error) => {
            eprintln!("Failed to read {filename}: {error}");
            (
                "HTTP/1.1 500 INTERNAL SERVER ERROR",
                String::from("Internal server error"),
            )
        }
    };

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    if let Err(error) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to write response: {error}");
    };
}
