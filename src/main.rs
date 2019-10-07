mod config;

fn main() {
    println!("Hello, PBFT!");

    let args: Vec<String> = std::env::args().collect();
    println!("Command line args: {:?}", args);

    if args.len() != 2 {
        println!("Usage: $ pbft {{port}}");
        std::process::exit(1);
    }

    let port: u64 = args
        .get(1).expect("Failed to get port number via CLI arguments")
        .parse().expect("Failed to parse the argument as port number");
    println!("port: {}", port);

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
}
