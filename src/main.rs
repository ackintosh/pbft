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
use std::collections::HashSet;
use crate::protocol::Pbft;

mod config;
mod discovery;
mod message_handler;
mod protocol;
mod state;
mod node_type;
mod message;
mod view;

fn main() {
    println!("Hello, PBFT!");

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
    let nodes = Arc::new(RwLock::new(HashSet::new()));

//    MessageHandler::new(
//        config,
//        port,
//        current_type,
//        state
//    ).listen();

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
        local_peer_id);

    Swarm::listen_on(&mut swarm, "/ip4/127.0.0.1/tcp/0".parse().unwrap()).unwrap();

    let mut listening = false;
    tokio::run(futures::future::poll_fn(move || {
        loop {
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
}
