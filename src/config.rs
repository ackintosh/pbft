use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    nodes: Vec<Port>,
    primary: Port,
}

impl Config {
    pub fn is_primary(&self, port: &Port) -> bool {
        self.primary.value == port.value
    }

    pub fn is_backup(&self, port: &Port) -> bool {
        self.backup_nodes().contains(&port)
    }

    pub fn backup_nodes(&self) -> Vec<&Port> {
        self.nodes.iter().filter(|n| {
            !self.is_primary(n)
        }).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Port {
    #[serde(alias = "port")]
    value: u64,
}

impl Port {
    pub fn value(&self) -> u64 {
        self.value
    }
}

impl From<&String> for Port {
    fn from(p: &String) -> Self {
        let port: u64 = p.parse().expect("Failed to port as u64");
        Port { value: port }
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
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