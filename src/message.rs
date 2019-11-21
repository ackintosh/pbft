use serde::{Serialize, Deserialize};
use blake2::{Blake2b, Digest};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub r#type: MessageType,
    payload: String,
}

impl Message {
    pub fn new(r#type: MessageType, payload: String) -> Self {
        Self {r#type, payload}
    }

    pub fn from(s: &String) -> Self {
        serde_json::from_str(s).unwrap()
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    ClientRequest,
    PrePrepare,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientRequest {
    operation: String,
    timestamp: u64,
    client: Option<String>,
}

impl ClientRequest {
    pub fn from(s: &String) -> Self {
        serde_json::from_str(s).unwrap()
    }

    pub fn operation(&self) -> String {
        self.operation.clone()
    }
}

impl From<Message> for ClientRequest {
    fn from(m: Message) -> Self {
        serde_json::from_str(&m.payload).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrePrepare {
    // view indicates the view in which the message is being sent
    view: u64,
    // sequence number for pre-prepare messages
    n: u64,
    // client message's digest
    digest: String,
}

impl PrePrepare {
    pub fn from(view: u64, n: u64, message: String) -> Self {
        let hash = Blake2b::digest(message.as_bytes());
        let digest = format!("{:x}", hash);
        Self { view, n, digest }
    }
}

impl std::fmt::Display for PrePrepare {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

pub struct PrePrepareSequence {
    value: u64,
}

impl PrePrepareSequence {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn increment(&mut self) {
        let from = self.value.clone();
        self.value += 1;
        println!("PrePrepareSequence has been incremented from {} to {}", from, self.value);
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}