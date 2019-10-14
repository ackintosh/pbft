use std::sync::{RwLock, Arc};
use crate::view::View;

pub struct State {
    logs: Vec<String>,
    current_view: Arc<RwLock<View>>
}

impl State {
    pub fn new() -> Self {
        Self {
            logs: vec![],
            current_view: Arc::new(RwLock::new(View::new()))
        }
    }

    pub fn current_view(&self) -> u64 {
        self.current_view.read().unwrap().value()
    }
}