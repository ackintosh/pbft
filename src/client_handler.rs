use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::io::{Read, Write};
use crate::message::{ClientRequest, Message, ClientReply};
use std::collections::VecDeque;
use crate::node_type::NodeType;

pub struct ClientHandler {
    node_type: NodeType,
    listener: TcpListener,
    client_requests: Arc<RwLock<VecDeque<ClientRequest>>>,
    client_replies: Arc<RwLock<VecDeque<ClientReply>>>,
    stream_states: VecDeque<ClientStreamState>,
}

impl ClientHandler {
    pub fn new(
        node_type: NodeType,
        client_requests: Arc<RwLock<VecDeque<ClientRequest>>>,
        client_replies: Arc<RwLock<VecDeque<ClientReply>>>,
    ) -> Self {
        let address = if node_type == NodeType::Primary {
            // Make the port number of primary node fixed in order to easily debug
            "127.0.0.1:8000"
        } else {
            "127.0.0.1:0"
        };
        let listener = TcpListener::bind(address).unwrap();
        listener.set_nonblocking(true).expect("Cannot set non-blocking");
        println!("[ClientHandler::new] Listening on {:?}", listener.local_addr().unwrap());

        Self {
            node_type,
            listener,
            client_requests,
            client_replies,
            stream_states: VecDeque::new(),
        }
    }

    pub fn tick(&mut self) {
        println!("[ClientHandler::tick]");

        // Accept an incoming stream
        if let Some(tcp_stream) = self.incoming().unwrap() {
            self.stream_states.push_back(ClientStreamState::WaitingForIncomingStream(tcp_stream));
        }

        // Consume a job to reply to client
        if let Some(reply) = self.client_replies.write().unwrap().pop_front() {
            self.stream_states.push_back(ClientStreamState::PrepareToSendReply(reply));
        }

        if let Some(state) = self.stream_states.pop_front() {
            match state {
                ClientStreamState::WaitingForIncomingStream(tcp_stream) => {
                    println!("[ClientHandler::tick] [ClientStreamState::WaitingForIncomingStream]");
                    let new_state = self.read_client_stream(tcp_stream).unwrap();
                    self.stream_states.push_back(new_state);
                }
                ClientStreamState::ReceivedClientMessage(message) => {
                    println!("[ClientHandler::tick] [ClientStreamState::ReceivedClientMessage] message: {:?}", message);
                    match message {
                        Message::ClientRequest(client_request) => {
                            if self.node_type == NodeType::Backup {
                                // TODO: transfer the message to primary replica if this node is running as backup
                                eprintln!("Can't process the client request as running as backup. client_request: {:?}", client_request);
                                return;
                            }
                            self.client_requests.write().unwrap().push_back(client_request);
                        }
                        _ => unreachable!()
                    }
                }
                ClientStreamState::PrepareToSendReply(reply) => {
                    println!("[ClientHandler::tick] [ClientStreamState::PrepareToSendReply] reply: {:?}", reply);
                    let mut stream = TcpStream::connect("127.0.0.1:9000").unwrap(); // TODO
                    stream.set_nonblocking(true).expect("Cannot set non-blocking");

                    match stream.write(reply.to_string().as_bytes()) {
                        Ok(_size) => println!("[ClientHandler::tick] [ClientStreamState::PrepareToSendReply] Sent the reply to the client. reply: {:?}", reply),
                        Err(e) => eprintln!("[ClientHandler::tick] [ClientStreamState::PrepareToSendReply] Failed to send the reply to the client. error: {:?}, reply: {:?}", e, reply)
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
        }
        unreachable!();
    }

    fn read_client_stream(&self, mut tcp_stream: TcpStream) -> Result<ClientStreamState, std::io::Error> {
        let mut buffer = [0u8; 512];
        match tcp_stream.read(&mut buffer) {
            Ok(size) => {
                let message = String::from_utf8_lossy(&buffer[..size]).to_string().into();
                println!("[ClientHandler::read_client_stream] message: {:?}", message);
                return Ok(ClientStreamState::ReceivedClientMessage(message));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                println!("[ClientHandler::read_client_stream] [ErrorKind::WouldBlock] e: {:?}", e);
                return Ok(ClientStreamState::WaitingForIncomingStream(tcp_stream));
            },
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
    PrepareToSendReply(ClientReply),
}

