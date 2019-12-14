use serde::{Serialize, Deserialize};
use blake2::{Blake2b, Digest};
use crate::config::Port;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub r#type: MessageType,
    pub payload: String,
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
    Prepare,
    HandlerPrePrepare(PrePrepare),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrePrepare {
    // view indicates the view in which the message is being sent
    view: u64,
    // sequence number for pre-prepare messages
    n: u64,
    // client message's digest
    digest: String,
    // client message
    message: String,
}

impl PrePrepare {
    pub fn view(&self) -> u64 {
        self.view
    }

    pub fn n(&self) -> u64 {
        self.n
    }

    pub fn digest(&self) -> &String {
        &self.digest
    }

    pub fn from(view: u64, n: u64, message: String) -> Self {
        let digest = digest(message.as_bytes());
        Self { view, n, digest, message }
    }

    pub fn from_payload(payload: &String) -> Self {
        serde_json::from_str(payload).unwrap()
    }

    pub fn validate_digest(&self) -> Result<(), String> {
        if self.digest == digest(&self.message.as_bytes()) {
            Ok(())
        } else {
            Err(format!("The digest is not matched with message. digest: {}, message: {}", self.digest, self.message))
        }
    }
}

impl std::fmt::Display for PrePrepare {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl From<Message> for PrePrepare {
    fn from(m: Message) -> Self {
        serde_json::from_str(&m.payload).unwrap()
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
        println!("[PrePrepareSequence::increment] value has been incremented from {} to {}", from, self.value);
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Prepare {
    view: u64,
    n: u64,
    digest: String,
    i: Port,
}

impl Prepare {
    pub fn new(pre_prepare: &PrePrepare, i: &Port) -> Self {
        Self {
            view: pre_prepare.view,
            n: pre_prepare.n,
            digest: pre_prepare.digest.clone(),
            i: i.clone(),
        }
    }

    pub fn view(&self) -> u64 {
        self.view
    }

    pub fn n(&self) -> u64 {
        self.n
    }

    pub fn i(&self) -> Port {
        self.i.clone()
    }
}

impl std::fmt::Display for Prepare {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

fn digest(message: &[u8]) -> String {
    let hash = Blake2b::digest(message);
    format!("{:x}", hash)
}