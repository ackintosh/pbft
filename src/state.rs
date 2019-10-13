use std::sync::{RwLock, Arc};

pub struct State {
    logs: Vec<String>,
    current_view: Arc<RwLock<u64>>
}

impl State {
    pub fn new() -> Self {
        Self {
            logs: vec![],
            current_view: Arc::new(RwLock::new(1))
        }
    }
}