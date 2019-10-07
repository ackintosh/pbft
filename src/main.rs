mod config;

fn main() {
    println!("Hello, PBFT!");
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
