use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};

fn main() {
    match TcpListener::bind("127.0.0.1:7878") {
        Ok(listener) => listener.incoming().for_each(|c| match c {
            Ok(mut stream) => handle_tcp_conn(&mut stream),
            Err(e) => println!("Connection established error! {:?}", e),
        }),
        Err(e) => println!(r#"error: TcpListener::bind "127.0.0.0.1:7878" {:?}"#, e),
    }
}

fn handle_tcp_conn(stream: &mut TcpStream) {
    let buf_reader = BufReader::new(stream as &TcpStream);
    match buf_reader.lines().next() {
        Some(res) => match res {
            Ok(first_line) => response_tcp_conn(&first_line, stream),
            Err(_) => println!("Error http request get first_line"),
        },
        None => println!("Error http request"),
    }
}

fn response_tcp_conn(first_line: &String, stream: &mut TcpStream) {
    // println!("first_line:{}", first_line);
    // first_line:POST /api/v1/expense/findByIds HTTP/1.1
    // first_line:GET /api/v1/expense/findByIds?ids=10078,29987,30014 HTTP/1.1
    let http_first_line = HttpFirstLine::from(first_line);
    println!("httpFirstLine:{:?}", http_first_line);

    let response_content = get_response_content(&http_first_line);
    let response = format!(
        "{}\r\nContent-Length:{}\r\n\r\n{}",
        response_content.status_line,
        response_content.contents.len(),
        response_content.contents
    );

    match stream.write_all(response.as_bytes()) {
        Ok(_) => println!("response_tcp_conn success!"),
        Err(e) => println!("response_tcp_conn error: {:?}", e),
    };
}

fn get_response_content(http_first_line: &Option<HttpFirstLine>) -> ResponseContent {
    match http_first_line {
        Some(hp) => ResponseContent {
            status_line: String::from("HTTP/1.1 200 OK"),
            contents: match hp.method {
                HttpMethod::GET => {
                    let file_path_str = format!("resource/{}", hp.path);
                    let file_path = Path::new(&file_path_str);
                    if !file_path.exists() || !file_path.is_file() {
                        String::from("")
                    } else {
                        fs::read_to_string(file_path).expect("msg")
                    }
                }
                HttpMethod::POST => todo!(),
                HttpMethod::DELETE => todo!(),
                HttpMethod::PUT => todo!(),
            },
        },
        None => ResponseContent {
            status_line: String::from("HTTP/1.1 404 NOT FOUND"),
            contents: String::from(""),
        },
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct HttpFirstLine {
    method: HttpMethod,
    path: String,
    http_protocol: HttpProtocol,
}

#[derive(Debug)]
struct ResponseContent {
    status_line: String,
    contents: String,
}

#[derive(Debug)]
enum HttpMethod {
    GET,
    POST,
    DELETE,
    PUT,
}

#[derive(Debug)]
enum HttpProtocol {
    HTTP1_1,
    HTTP2_0,
}

impl HttpMethod {
    fn from(str: &str) -> Option<Self> {
        match str.to_uppercase().as_str() {
            "GET" => Some(Self::GET),
            "POST" => Some(Self::POST),
            "DELETE" => Some(Self::DELETE),
            "PUT" => Some(Self::PUT),
            err => {
                println!("HttpMethod::from error, cant match param:{}", err);
                None
            }
        }
    }
}

impl HttpProtocol {
    fn from(str: &str) -> Option<Self> {
        match str.to_uppercase().as_str() {
            "HTTP/1.1" => Some(Self::HTTP1_1),
            "HTTP/2" => Some(Self::HTTP2_0),
            err => {
                println!("HttpProtocol::from error, cant match param:{}", err);
                None
            }
        }
    }
}

impl HttpFirstLine {
    fn from(str: &str) -> Option<Self> {
        // param 'str' demo:
        // POST /api/v1/expense/findByIds HTTP/1.1
        // GET /api/v1/expense/findByIds?ids=10078,29987,30014 HTTP/1.1
        let ele_str: Vec<&str> = str.split(char::is_whitespace).collect();
        if ele_str.len() != 3 {
            println!("HttpFirstLine::from error, cant match param:{:?}", ele_str);
            return None;
        }

        Some(Self {
            method: if let Some(hm) = HttpMethod::from(ele_str[0]) {
                hm
            } else {
                return None;
            },
            path: ele_str[1].to_string(),
            http_protocol: if let Some(hp) = HttpProtocol::from(ele_str[2]) {
                hp
            } else {
                return None;
            },
        })
    }
}
