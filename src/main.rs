#[macro_use]
extern crate serde_derive;

fn main() {
    println!("Hello, PBFT!");
    let c = read_config();
    println!("{:?}", c);
}


#[derive(Debug, Serialize, Deserialize)]
struct Config {
    nodes: Vec<Port>,
    primary: Port,
}

#[derive(Debug, Serialize, Deserialize)]
struct Port {
    port: u64,
}

fn read_config() -> Result<Config, String> {
    let content = match std::fs::read_to_string("network.json") {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to read network.json: {}", e))
    };

    println!("{}", content);
    match serde_json::from_str(&content) {
        Ok(c) => Ok(c),
        Err(e) => Err(format!("Failed to deserialize network.json: {}", e))
    }
}