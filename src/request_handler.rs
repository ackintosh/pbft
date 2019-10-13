use crate::config::{Config, Port};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use crate::node_type::CurrentType;

pub struct RequestHandler {
    config: Config,
    port: Port,
    current_type: Arc<RwLock<CurrentType>>
}

impl RequestHandler {
    pub fn new(config: Config, port: Port, current_type: Arc<RwLock<CurrentType>>) -> Self {
        {
            let ct = current_type.read().unwrap();
            println!("RequestHandler has been initialized as {} replica", ct);
        }
        Self { config, port, current_type }
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