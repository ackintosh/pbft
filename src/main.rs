use crate::config::Port;
use crate::message_handler::MessageHandler;
use std::sync::{Arc, RwLock};
use crate::node_type::NodeType;
use libp2p::{PeerId, build_development_transport, Swarm};
use libp2p::identity::Keypair;
use crate::network_behaviour_composer::NetworkBehaviourComposer;
use futures::Async;
use futures::stream::Stream;
use std::collections::VecDeque;
use crate::behavior::Pbft;
use crate::message::ClientRequest;
use std::thread::JoinHandle;

mod config;
mod network_behaviour_composer;
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
    let cli_args: Vec<String> = std::env::args().collect();
    println!("[main] cli_args: {:?}", cli_args);
    let node_type = determine_node_type(&cli_args).expect("Usage: $ pbft [primary]");
    println!("[main] node_type: {:?}", node_type);

    let client_requests = Arc::new(RwLock::new(VecDeque::new()));

    if node_type == NodeType::Primary {
        let _ = run_client_request_handler(
            client_requests.clone(),
        );
    }

    let local_key = Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    let transport = build_development_transport(local_key);
    let mut swarm = Swarm::new(
        transport,
        NetworkBehaviourComposer::new(
            libp2p::mdns::Mdns::new().expect("Failed to create mDNS service"),
            Pbft::new(),
        ),
        local_peer_id
    );

    Swarm::listen_on(&mut swarm, "/ip4/127.0.0.1/tcp/0".parse().unwrap()).unwrap();

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
                    return Ok(Async::NotReady);
                }
            }
        }
    }));
}

fn determine_node_type(args: &Vec<String>) -> Result<NodeType, ()> {
    match args.len() {
        1 => Ok(NodeType::Backup),
        2 => {
            if let Some(node_type) = args.get(1) {
                if node_type == "primary" {
                    return Ok(NodeType::Primary)
                } else {
                    panic!(format!("[main::determine_node_type] Invalid node_type: {:?}", node_type));
                }
            } {
                unreachable!();
            }
        },
        _ => Err(()),
    }
}

fn run_client_request_handler(
    client_requests: Arc<RwLock<VecDeque<ClientRequest>>>,
) -> JoinHandle<()> {
    let port: Port = "8000".into();
    println!("{:?}", port);

    std::thread::spawn(move || {
        MessageHandler::new(
            port,
            client_requests,
        ).listen();
    })
}
