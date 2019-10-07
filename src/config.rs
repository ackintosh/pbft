use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    nodes: Vec<Port>,
    primary: Port,
}

#[derive(Debug, Serialize, Deserialize)]
struct Port {
    port: u64,
}

pub fn read_config() -> Result<Config, ConfigError> {
    std::fs::read_to_string("network.json")
        .and_then(|content| {
            serde_json::from_str(&content).map_err(|e| e.into())
        })
        .map_err(|e| e.into())
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    SerdeError(serde_json::error::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl From<serde_json::error::Error> for ConfigError {
    fn from(err: serde_json::error::Error) -> Self {
        ConfigError::SerdeError(err)
    }
}