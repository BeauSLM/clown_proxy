use std::io::prelude::*;
use std::net::{ TcpListener, TcpStream };
use std::env::{ self, Args };
use std::process;

fn main() {
    let port = rust_clown_proxy::parse_port(env::args());
    let listener = TcpListener::bind(format!("{}{}", "127.0.0.1:", port)).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        handle_connection(stream);
    }
}

fn handle_connection(mut client: TcpStream) {
    const KILOBYTE: usize = 1024;
    let mut request = [0; KILOBYTE * 8];
    let mut response = [0; KILOBYTE * 8096];

    let bytes = client.read(&mut request).unwrap();
    if bytes == 0 || !request.starts_with("GET".as_bytes()) { return; }

    let domain = parse_domain(std::str::from_utf8(&request).unwrap());
    let mut server = TcpStream::connect(format!("{}{}", domain, ":80")).unwrap();

    server.write(&request[..bytes]).unwrap();
    while let Ok(bytes) = server.read(&mut response) {
        client.write(&response[..bytes]).unwrap();
    }
}

fn parse_domain(request: &str) -> &str {
    let mut domain = request
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<_>>()[1];
    const PAT: &str = "://";
    if let Some(ix) = domain.find(PAT) {
        domain = &domain[ix+PAT.len()..];
    }
    let ix = domain.find('/').unwrap();
    &domain[..ix]
}

pub fn parse_port(mut args: Args) -> u16 {
    let name = args.next().unwrap(); // name of the executable
    fn usage(name: String) {
        eprintln!("Usage: {name} <port>, where 23 < <port> < 65536");
        process::exit(1);
    }
    if args.len() != 1 {
        usage(name);
        unreachable!();
    }
    
    match args.next().unwrap().parse::<u16>() {
        Ok(port) if port < 24 => {
            usage(name);
            unreachable!();
        },
        Ok(port) => {
            port
        },
        _ => {
            usage(name);
            unreachable!();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn parse_normal() {
        let request = "GET pages.cpsc.ucalgary.ca/ HTTP/1.0\r\n\r\n";
        assert_eq!("pages.cpsc.ucalgary.ca", parse_domain(request));
    }
    
    #[test]
    fn parse_fancy() {
        let request = "GET http://pages.cpsc.ucalgary.ca/ HTTP/1.0\r\n\r\n";
        assert_eq!("pages.cpsc.ucalgary.ca", parse_domain(request));
    }
    
    #[test]
    fn parse_long() {
        let request = "GET http://pages.cpsc.ucalgary.ca/~carey/index.html HTTP/1.0\r\n\r\nskldjflsd\r\nslkdjf\r\n\r\n";
        assert_eq!("pages.cpsc.ucalgary.ca", parse_domain(request));
    }
}
