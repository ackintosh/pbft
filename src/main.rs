use crate::config::Port;
use crate::message_handler::MessageHandler;
use std::sync::{Arc, RwLock};
use crate::node_type::CurrentType;
use crate::state::State;
use libp2p::{PeerId, build_development_transport, Swarm};
use libp2p::identity::Keypair;
use crate::discovery::Discovery;
use futures::Async;
use futures::stream::Stream;
use std::collections::{HashSet, VecDeque};
use crate::behavior::Pbft;
use crate::message::{MessageType, ClientRequest};
use std::thread::JoinHandle;
use tokio::prelude::{AsyncRead, AsyncWrite};

mod config;
mod discovery;
mod handler;
mod message_handler;
mod behavior;
mod protocol_config;
mod state;
mod node_type;
mod message;
mod view;

fn main() {
    println!("Hello, PBFT!");
    let nodes = Arc::new(RwLock::new(HashSet::new()));

    let local_key = Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    let transport = build_development_transport(local_key);
    let mut swarm = Swarm::new(
        transport,
        Discovery::new(
            libp2p::mdns::Mdns::new().expect("Failed to create mDNS service"),
            Pbft::new(),
            nodes.clone()
        ),
        local_peer_id
    );

    Swarm::listen_on(&mut swarm, "/ip4/127.0.0.1/tcp/0".parse().unwrap()).unwrap();

    let client_requests = Arc::new(RwLock::new(VecDeque::new()));
    let _ = run_client_request_handler(
        client_requests.clone(),
    );

    let mut listening = false;
    tokio::run(futures::future::poll_fn(move || {
        loop {
            if let Some(client_request) = client_requests.write().unwrap().pop_front() {
                swarm.pbft.add_client_request(client_request);
            }

            match swarm.poll().expect("Error while polling swarm") {
                Async::Ready(Some(_)) => {}
                Async::Ready(None) | Async::NotReady => {
                    if !listening {
                        if let Some(a) = Swarm::listeners(&swarm).next() {
                            println!("Listening on {:?}", a);
                            listening = true;
                        }
                    }
//                    break;
                    return Ok(Async::NotReady);
                }
            }
        }
    }));

    fn run_client_request_handler(
        client_requests: Arc<RwLock<VecDeque<ClientRequest>>>,
    ) -> JoinHandle<()> {
        let args: Vec<String> = std::env::args().collect();
        println!("Command line args: {:?}", args);

        if args.len() != 2 {
            println!("Usage: $ pbft {{port}}");
            std::process::exit(1);
        }

        let port: Port = args
            .get(1).expect("Failed to get port number via CLI arguments")
            .into();
        println!("{:?}", port);

        let config = match config::read_config() {
            Ok(c) => {
                println!("{:?}", c);
                c
            },
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        };

        let current_type = match CurrentType::from(&config, &port) {
            Ok(c) => Arc::new(RwLock::new(c)),
            Err(_) => {
                println!("The port number does not exist in the p2p network configuration: {:?}", port);
                std::process::exit(1);
            }
        };

        let state = Arc::new(RwLock::new(State::new()));

        std::thread::spawn(move || {
            MessageHandler::new(
                config,
                port,
                current_type,
                state,
                client_requests,
            ).listen();
        })
    }
}
