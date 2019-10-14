use crate::config::Port;
use crate::message_handler::MessageHandler;
use std::sync::{Arc, RwLock};
use crate::node_type::CurrentType;
use crate::state::State;

mod config;
mod message_handler;
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

    MessageHandler::new(
        config,
        port,
        current_type,
        state
    ).listen();
}
