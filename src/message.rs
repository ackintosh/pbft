use serde::{Serialize, Deserialize};
use blake2::{Blake2b, Digest};
use libp2p::PeerId;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    ClientRequest(ClientRequest),
    PrePrepare(PrePrepare),
    Prepare(Prepare),
    Commit(Commit),
}

impl From<Vec<u8>> for Message {
    fn from(item: Vec<u8>) -> Self {
        serde_json::from_str(&String::from_utf8(item).unwrap()).unwrap()
    }
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        serde_json::from_str(&s).unwrap()
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientRequest {
    operation: String,
    timestamp: u64,
    client: Option<String>, // TODO: client address
}

impl ClientRequest {
    pub fn operation(&self) -> String {
        self.operation.clone()
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

#[derive(Debug)]
pub struct ClientReply {
    view: u64,
    timestamp: u64,
    // c?
    peer_id: PeerId,
    result: String,
}

impl ClientReply {
    pub fn new(peer_id: PeerId, pre_prepare: &PrePrepare, commit: &Commit) -> Self {
        Self {
            view: commit.view(),
            timestamp: pre_prepare.client_reqeust().timestamp(),
            peer_id,
            result: "awesome!".to_owned(), // TODO
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrePrepare {
    // view indicates the view in which the message is being sent
    view: u64,
    // sequence number for pre-prepare messages
    sequence_number: u64,
    // client message's digest
    digest: String,
    // client message
    message: ClientRequest,
}

impl PrePrepare {
    pub fn view(&self) -> u64 {
        self.view
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn digest(&self) -> &String {
        &self.digest
    }

    pub fn client_reqeust(&self) -> &ClientRequest {
        &self.message
    }

    pub fn from(view: u64, n: u64, client_request: ClientRequest) -> Self {
        let digest = digest(client_request.operation.as_bytes());
        Self { view, sequence_number: n, digest, message: client_request }
    }

    pub fn validate_digest(&self) -> Result<(), String> {
        if self.digest == digest(&self.message.operation.as_bytes()) {
            Ok(())
        } else {
            Err(format!("The digest is not matched with message. digest: {}, message.operation: {}", self.digest, self.message.operation))
        }
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
        println!("[PrePrepareSequence::increment] value has been incremented from {} to {}", from, self.value);
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Prepare {
    view: u64,
    sequence_number: u64,
    digest: String,
}

impl Prepare {
    pub fn from(pre_prepare: &PrePrepare) -> Self {
        Self {
            view: pre_prepare.view,
            sequence_number: pre_prepare.sequence_number,
            digest: pre_prepare.digest.clone(),
        }
    }

    pub fn view(&self) -> u64 {
        self.view
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn digest(&self) -> &String {
        &self.digest
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commit {
    view: u64,
    sequence_number: u64,
    digest: String,
}

impl Commit {
    pub fn view(&self) -> u64 {
        self.view
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }
}

impl From<Prepare> for Commit {
    fn from(prepare: Prepare) -> Self {
        Self {
            view: prepare.view(),
            sequence_number: prepare.sequence_number(),
            digest: prepare.digest().clone(),
        }
    }
}

impl std::fmt::Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
