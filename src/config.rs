use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct Port {
    #[serde(alias = "port")]
    value: u64,
}

impl Port {
    pub fn value(&self) -> u64 {
        self.value
    }
}

impl From<&str> for Port {
    fn from(p: &str) -> Self {
        let port: u64 = p.parse().expect("Failed to port as u64");
        Port { value: port }
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
