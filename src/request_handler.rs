use crate::config::{Config, Port};
use std::net::{TcpListener, TcpStream};

pub struct RequestHandler {
    config: Config,
    port: Port,
}

impl RequestHandler {
    pub fn new(config: Config, port: Port) -> Self {
        Self { config, port }
    }

    pub fn listen(&self) {
        let address = format!("127.0.0.1:{}", self.port.value());
        println!("RequestHandler is listening on {}", address);
        let listener = TcpListener::bind(address).unwrap();

        for stream in listener.incoming() {
            self.handle(&stream.unwrap());
        }
    }

    fn handle(&self, stream: &TcpStream) {
        // TODO
        println!("{:?}", stream);
    }
}