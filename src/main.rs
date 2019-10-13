use crate::config::Port;
use crate::request_handler::RequestHandler;

mod config;
mod request_handler;

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

    if config.is_primary(&port) {
        println!("Running as primary node");
        RequestHandler::new(config, port).listen();
    } else if config.is_backup(&port) {
        println!("Running as backup node");
    } else {
        println!("The port number does not exist in current p2p network: {:?}", port);
        std::process::exit(1);
    }
}
