use crate::config::{Config, Port};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use crate::node_type::CurrentType;
use std::io::{Read, Write};
use crate::message::{ClientRequest, PrePrepareSequence, PrePrepare, Message, MessageType, Prepare};
use crate::state::State;

pub struct MessageHandler {
    config: Config,
    port: Port,
    current_type: Arc<RwLock<CurrentType>>,
    state: Arc<RwLock<State>>,
    pre_prepare_sequence: PrePrepareSequence,
}

impl MessageHandler {
    pub fn new(
        config: Config,
        port: Port,
        current_type: Arc<RwLock<CurrentType>>,
        state: Arc<RwLock<State>>
    ) -> Self {
        {
            let ct = current_type.read().unwrap();
            println!("MessageHandler has been initialized as {} replica", ct);
        }
        Self {
            config,
            port,
            current_type,
            state,
            pre_prepare_sequence: PrePrepareSequence::new(),
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
        let body = String::from_utf8_lossy(&buffer[..size]).to_string();

        let message = Message::from(&body);
        println!("{:?}", message);

        match message.r#type {
             MessageType::ClientRequest => {
                 if self.current_type.read().unwrap().is_backup() {
                     println!("Now the replica is running as BACKUP.");
                     // TODO: transfer the message to primary replica
                 } else {
                     self.handle_client_request(message.into());
                 }
            },
            MessageType::PrePrepare => {
                if self.current_type.read().unwrap().is_backup() {
                    self.handle_pre_prepare(message.into())?;
                } else {
                    println!("Something wrong! : PRIMARY replica received a PrePrepare message.")
                }
            },
            MessageType::Prepare => {
                // TODO
            }
        }

        Ok(())
    }

    fn handle_client_request(&mut self, request: ClientRequest) {
        println!("{:?}", request);

        // TODO: multicast the request to backup replicas
        // In the pre-prepare phase, the primary assigns a sequence number, n, to the request
        self.pre_prepare_sequence.increment();

        let pre_prepare_message = PrePrepare::from(
            self.state.read().unwrap().current_view(),
            self.pre_prepare_sequence.value(),
            request.operation()
        );
        println!("PrePrepare: {:?}", pre_prepare_message);
        self.send_pre_prepare(&pre_prepare_message);

    }

    fn send_pre_prepare(&self, pre_prepare: &PrePrepare) {
        for node in self.config.backup_nodes() {
            match TcpStream::connect(format!("127.0.0.1:{}", node)) {
                Ok(mut stream) => {
                    println!("Successfully connected to the node: {:?}", node);

                    stream.write(
                        Message::new(MessageType::PrePrepare, pre_prepare.to_string())
                            .to_string()
                            .as_bytes()
                    );
                    // TODO receive a reply from the backup replica
                }
                // TODO retry the message transmission
                // TODO error handling
                Err(e) => println!("Failed to connect to the node: {:?}, error: {:?}", node, e)
            }
        }
    }

    fn handle_pre_prepare(&self, pre_prepare: PrePrepare) -> Result<(), String> {
        println!("PrePrepare: {:?}", pre_prepare);

        self.validate_pre_prepare(&pre_prepare)?;

        // If backup replica accepts the message, it enters the prepare phase by multicasting a PREPARE message to
        // all other replicas and adds both messages to its log.
        let prepare = Prepare::new(&pre_prepare, &self.port);
        self.send_prepare(&prepare);
        {
            let mut state = self.state.write().unwrap();
            state.insert_pre_prepare(pre_prepare);
            state.insert_prepare(prepare);
        }
        Ok(())
    }

    fn validate_pre_prepare(&self, pre_prepare: &PrePrepare) -> Result<(), String> {
        // TODO: the signatures in the request and the pre-prepare message are correct

        // _d_ is the digest for _m_
        pre_prepare.validate_digest()?;

        // it is in view _v_
        {
            let current_view = self.state.read().unwrap().current_view();
            if pre_prepare.view() != current_view {
                return Err(format!("view number isn't matched. message: {}, state: {}", pre_prepare.view(), current_view));
            }
        }

        // TODO: it has not accepted a pre-prepare message for view _v_ and sequence number _n_ containing a different digest

        // TODO: the sequence number in the pre-prepare message is between a low water mark, _h_, and a high water mark, _H_

        Ok(())
    }

    fn send_prepare(&self, prepare: &Prepare) {
        let message = Message::new(
            MessageType::Prepare,
            prepare.to_string()
        ).to_string();

        for node in self.config.all_nodes_without_me(&self.port) {
            match TcpStream::connect(format!("127.0.0.1:{}", node)) {
                Ok(mut stream) => {
                    println!("Successfully connected to the node: {:?}", node);

                    stream.write(message.as_bytes());
                    // TODO receive a reply from the backup replica
                }
                // TODO retry the message transmission
                // TODO error handling
                Err(e) => println!("Failed to connect to the node: {:?}, error: {:?}", node, e)
            }
        }
    }
}

