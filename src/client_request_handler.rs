use crate::config::Port;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::io::Read;
use crate::message::{ClientRequest, Message};
use std::collections::VecDeque;

pub struct ClientRequestHandler {
    port: Port,
    client_requests: Arc<RwLock<VecDeque<ClientRequest>>>,
}

impl ClientRequestHandler {
    pub fn new(
        port: Port,
        client_requests: Arc<RwLock<VecDeque<ClientRequest>>>,
    ) -> Self {
        Self {
            port,
            client_requests,
        }
    }

    pub fn listen(&mut self) {
        let address = format!("127.0.0.1:{}", self.port.value());
        println!("MessageHandler is listening on {}", address);
        let listener = TcpListener::bind(address).unwrap();

        for stream in listener.incoming() {
            self.handle(&stream.unwrap());
        }
    }

    fn handle(&mut self, mut stream: &TcpStream) -> Result<(), String> {
        let mut buffer = [0u8; 512];
        let size = stream.read(&mut buffer).unwrap();
        let message: Message = String::from_utf8_lossy(&buffer[..size]).to_string().into();
        println!("[ClientRequestHandler::handle] message: {:?}", message);

        match message {
             Message::ClientRequest(client_request) => {
                 // TODO: transfer the messageto primary replica if this node is running as backup
                 self.client_requests.write().unwrap().push_back(client_request);
            },
            _ => unreachable!()
        }

        Ok(())
    }
}

