use std::net::{TcpStream, TcpListener, Ipv4Addr};
use std::thread;
use std::io::Write;


use process;
use serde_json;

pub trait OpServer {
    fn ip(&self) -> Ipv4Addr;
    fn hostname(&self) -> String;
}

pub struct TcpServer<'a> {
    listener: &'a TcpListener,
}

impl<'a> OpServer for TcpServer<'a> {
    fn ip(&self) -> Ipv4Addr {
        Ipv4Addr::new(127, 0, 0, 1)
    }

    fn hostname(&self) -> String {
        "localhost".to_owned()
    }
}

fn handle_client(stream: &mut TcpStream) {
    stream.write(serde_json::to_string(&process::ps().unwrap()).unwrap().as_bytes());
    // stream.write(&[1]);
}

impl<'a> TcpServer<'a> {
    pub fn new(l: &'a TcpListener) -> TcpServer {
        TcpServer { listener: l }
    }

    pub fn serve(&self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut s) => {
                    // s.write(b"hello, world!");
                    thread::spawn(move || { handle_client(&mut s); });
                }
                Err(e) => println!("{:?}", e),
            }
        }
    }
}
