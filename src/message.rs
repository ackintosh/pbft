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

pub struct PrePrepare {
    // view indicates the view in which the message is being sent
    view: u64,
    // sequence number for pre-prepare messages
    n: u64,
    // client message's digest
    digest: String,
}

pub struct PrePrepareSequence {
    value: u64,
}

impl PrePrepareSequence {
    pub fn new() -> Self {
        Self { value: 1 }
    }
}