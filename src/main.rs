use tokio::io::{ AsyncReadExt, AsyncWriteExt, Error, Result };
use tokio::net::{ TcpListener, TcpStream };
use std::env::{ self, Args };
use std::io::ErrorKind;
use std::process;
use lazy_static::lazy_static;
use regex::bytes::Regex;

#[tokio::main]
async fn main() -> Result<()> {
    let port = parse_port(env::args());
    let listener = TcpListener::bind(format!("{}{}", "127.0.0.1:", port)).await?;
    loop {
        let (stream, addr) = listener.accept().await?;
        
        if let Err(e) = handle_connection(stream).await {
            eprintln!("Problem with connection to {addr}: {e}");
        }
    }
}

async fn handle_connection(mut client: TcpStream) -> Result<()> {
    const KILOBYTE: usize = 1024;
    let mut request = [0; KILOBYTE * 8];
    let mut response = [0; KILOBYTE * 16];

    let mut bytes = client.read(&mut request).await?;
    if bytes == 0 || !request.starts_with(b"GET") {
        return Err(Error::new(ErrorKind::Other, "Empty or non-get request"));
    }

    let str = std::str::from_utf8(&request).unwrap();
    let domain = parse_domain(str);
    let mut server = TcpStream::connect(format!("{}{}", domain, ":80")).await?;
    
    // is the request asking for a jpeg?
    let image: bool = str.contains(".jpg");
    
    if image {
        let clown_url = format!(
            "GET http://pages.cpsc.ucalgary.ca/~carey/CPSC441/ass1/clown{}.png HTTP/1.0\r\n\r\n",
            fastrand::bool() as u8 + 1
        );
        bytes = clown_url.len();
        (&mut request[..bytes]).copy_from_slice(clown_url.as_bytes());
    }

    server.try_write(&request[..bytes])?;
    while let Ok(bytes) = server.read(&mut response).await {
        let response = &mut response[..bytes];
        if !image { happy_silly_sub(response); }
        if client.write(response).await? == 0 {
            return Err(Error::new(ErrorKind::Other, "Client closed connection early"));
        }
    }
    
    Ok(())
}

fn parse_domain(request: &str) -> &str {
    // extract the path from the first line
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
        eprintln!("Usage: {name} <port>, where 1023 < <port> < 65536");
        process::exit(1);
    }
    if args.len() != 1 {
        usage(name);
        unreachable!();
    }
    
    match args.next().unwrap().parse::<u16>() {
        Ok(port) if port < 1024 => {
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

fn happy_silly_sub(s: &mut [u8]) {
    lazy_static! {
        static ref RE: Regex = Regex::new("(?i)happy").unwrap();
    }
    if !RE.is_match(s) { return; }
    let mut replaces = Vec::with_capacity(s.len() / 5);
    for range in RE.find_iter(s).map(|word| word.range()) {
        let mut arr = b"silly".to_owned();
        for (i, ix) in range.clone().enumerate() {
            if s[ix].is_ascii_uppercase() {
                arr[i].make_ascii_uppercase();
            }
        }
        replaces.push((range, arr));
    }
    for (range, slice) in replaces {
        let s = &mut s[range];
        s.copy_from_slice(&slice);
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
    
    #[test]
    fn lower_sub() {
        let mut target = b"happy".to_owned();
        happy_silly_sub(&mut target);
        assert_eq!(b"silly", &target, "target = {}", String::from_utf8_lossy(&target));
    }

    #[test]
    fn upper_sub() {
        let mut target = b"HAPPY".to_owned();
        happy_silly_sub(&mut target);
        assert_eq!(b"SILLY", &target, "target = {}", String::from_utf8_lossy(&target));
    }

    #[test]
    fn mixed_sub() {
        let mut target = b"hAppY".to_owned();
        happy_silly_sub(&mut target);
        assert_eq!(b"sIllY".to_owned(), target);
    }
}
