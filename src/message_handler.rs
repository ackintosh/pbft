use crate::config::{Config, Port};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use crate::node_type::CurrentType;
use std::io::Read;
use crate::message::{Request, PrePrepareSequence, PrePrepare};
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
            if self.current_type.read().unwrap().is_backup() {
                println!("Now the replica is running as BACKUP.");
                // TODO: transfer the message to primary replica
                continue;
            }
            self.handle(&stream.unwrap());
        }
    }

    fn handle(&mut self, mut stream: &TcpStream) {
        let mut buffer = [0u8; 512];
        let size = stream.read(&mut buffer).unwrap();
        let body = String::from_utf8_lossy(&buffer[..size]).to_string();

        let request= Request::from(&body);
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
    }

}
