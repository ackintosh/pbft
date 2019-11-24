use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use crate::view::View;
use crate::message::{PrePrepare, Prepare};
use crate::config::Port;

pub struct State {
    logs: Vec<String>,
    current_view: Arc<RwLock<View>>,
    pre_prepares: HashMap<PrePrepareKey, PrePrepare>,
    prepares: HashMap<PrepareKey, HashMap<Port, Prepare>>,
}

#[derive(PartialEq, Eq, Hash)]
struct PrePrepareKey(u64, u64); // (view, n)

#[derive(PartialEq, Debug, Eq, Hash)]
struct PrepareKey(u64, u64);// (view, n)

impl State {
    pub fn new() -> Self {
        Self {
            logs: vec![],
            current_view: Arc::new(RwLock::new(View::new())),
            pre_prepares: HashMap::new(),
            prepares: HashMap::new(),
        }
    }

    pub fn current_view(&self) -> u64 {
        self.current_view.read().unwrap().value()
    }

    pub fn insert_pre_prepare(&mut self, pre_prepare: PrePrepare) {
        println!("The PrePrepare message has been stored into logs: {}", pre_prepare);

        self.pre_prepares.insert(
            PrePrepareKey(pre_prepare.view(), pre_prepare.n()),
            pre_prepare
        );
    }

    pub fn insert_prepare(&mut self, prepare: Prepare) {
        println!("The Prepare message has been stored into logs: {}", prepare);

        let key = PrepareKey(prepare.view(), prepare.n());
        let p = self.prepares
            .entry(key)
            .or_insert(HashMap::new());
        p.insert(prepare.i(), prepare);
    }
}