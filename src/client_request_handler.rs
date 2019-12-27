use crate::config::Port;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::io::Read;
use crate::message::{ClientRequest, Message};
use std::collections::VecDeque;
use std::time::Duration;

pub struct ClientRequestHandler {
    port: Port,
    listener: TcpListener,
    client_requests: Arc<RwLock<VecDeque<ClientRequest>>>,
    stream_states: VecDeque<ClientStreamState>,
}

impl ClientRequestHandler {
    pub fn new(
        port: Port,
        client_requests: Arc<RwLock<VecDeque<ClientRequest>>>,
    ) -> Self {
        let address = format!("127.0.0.1:{}", port.value());
        println!("MessageHandler is listening on {}", address);
        let listener = TcpListener::bind(address).unwrap();
        listener.set_nonblocking(true).expect("Cannot set non-blocking");

        Self {
            port,
            listener,
            client_requests,
            stream_states: VecDeque::new(),
        }
    }

    pub fn tick(&mut self) {
        println!("[ClientRequestHandler::tick]");

        if let Some(tcp_stream) = self.incoming().unwrap() {
            self.stream_states.push_back(ClientStreamState::WaitingForIncomingStream(tcp_stream));
        }

        // TODO: consume a job to reply to client

        if let Some(state) = self.stream_states.pop_front() {
            match state {
                ClientStreamState::WaitingForIncomingStream(tcp_stream) => {
                    if let Some(message) = self.read_client_stream(tcp_stream).unwrap() {
                        self.stream_states.push_back(ClientStreamState::ReceivedClientMessage(message));
                    }
                }
                ClientStreamState::ReceivedClientMessage(message) => {
                    match message {
                        Message::ClientRequest(client_request) => {
                            // TODO: transfer the message to primary replica if this node is running as backup
                            self.client_requests.write().unwrap().push_back(client_request);
                        }
                        _ => unreachable!()
                    }
                }
            }
        }
    }

    fn incoming(&self) -> Result<Option<TcpStream>, std::io::Error> {
        for tcp_stream in self.listener.incoming() {
            match tcp_stream {
                Ok(s) => return Ok(Some(s)),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
                Err(e) => {
                    println!("encountered IO error: {}", e);
                    return Err(e)
                },
            }
            unreachable!();
        }
        unreachable!();
    }

    fn read_client_stream(&self, mut tcp_stream: TcpStream) -> Result<Option<Message>, std::io::Error> {
        let mut buffer = [0u8; 512];
        match tcp_stream.read(&mut buffer) {
            Ok(size) => {
                let message = String::from_utf8_lossy(&buffer[..size]).to_string().into();
                return Ok(Some(message));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => {
                println!("encountered IO error: {}", e);
                return Err(e)
            },
        }
    }
}

enum ClientStreamState {
    WaitingForIncomingStream(TcpStream),
    ReceivedClientMessage(Message),
}

