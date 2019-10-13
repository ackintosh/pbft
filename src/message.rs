use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    operation: String,
    timestamp: u64,
    client: Option<String>,
}

impl Request {
    pub fn from(s: &String) -> Self {
        serde_json::from_str(s).unwrap()
    }
}
