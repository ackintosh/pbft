use crate::config::{Config, Port};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use crate::node_type::CurrentType;
use std::io::Read;
use crate::message::Request;

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
            if self.current_type.read().unwrap().is_backup() {
                println!("Now the replica is running as BACKUP.");
                // TODO: transfer the message to primary replica
                continue;
            }
            self.handle(&stream.unwrap());
        }
    }

    fn handle(&self, mut stream: &TcpStream) {
        let mut buffer = [0u8; 512];
        let size = stream.read(&mut buffer).unwrap();
        let body = String::from_utf8_lossy(&buffer[..size]).to_string();

        let request= Request::from(&body);
        println!("{:?}", request);

        // TODO: multicast the request to backup replicas
    }

}
