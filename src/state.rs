use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use crate::view::View;
use crate::message::{PrePrepare, Prepare, Commit};
use libp2p::PeerId;

pub struct State {
    logs: Vec<String>,
    current_view: Arc<RwLock<View>>,
    pre_prepares: HashMap<PrePrepareKey, PrePrepare>,
    prepares: HashMap<PrepareKey, HashMap<PeerId, Prepare>>,
    commits: HashMap<CommitKey, HashMap<PeerId, Commit>>,
}

#[derive(PartialEq, Eq, Hash)]
struct PrePrepareKey(u64, u64); // (view, sequence_number)

#[derive(PartialEq, Debug, Eq, Hash)]
struct PrepareKey(u64, u64);// (view, sequence_number)

#[derive(PartialEq, Eq, Hash)]
struct CommitKey(u64); // view

impl State {
    pub fn new() -> Self {
        Self {
            logs: vec![],
            current_view: Arc::new(RwLock::new(View::new())),
            pre_prepares: HashMap::new(),
            prepares: HashMap::new(),
            commits: HashMap::new(),
        }
    }

    pub fn current_view(&self) -> u64 {
        self.current_view.read().unwrap().value()
    }

    pub fn insert_pre_prepare(&mut self, pre_prepare: PrePrepare) {
        println!("[State::insert_pre_prepare] The PrePrepare message has been stored into logs: {}", pre_prepare);

        self.pre_prepares.insert(
            PrePrepareKey(pre_prepare.view(), pre_prepare.sequence_number()),
            pre_prepare
        );
    }

    pub fn insert_prepare(&mut self, peer_id: PeerId, prepare: Prepare) {
        println!("[State::insert_prepare] The Prepare message has been stored into logs: {}", prepare);

        let key = PrepareKey(prepare.view(), prepare.sequence_number());
        let p = self.prepares
            .entry(key)
            .or_insert(HashMap::new());
        p.insert(peer_id, prepare);
    }

    pub fn insert_commit(&mut self, peer_id: PeerId, commit: Commit) {
        println!("[State::insert_commit] The Commit message has been stored into logs: {}", commit);

        let key = CommitKey(commit.view());
        let c = self.commits
            .entry(key)
            .or_insert(HashMap::new());
        c.insert(peer_id, commit);
    }

    pub fn prepare_len(&self, view: u64, sequence_number: u64) -> usize {
        self.prepares.get(&PrepareKey(view, sequence_number)).unwrap().len()
    }

    pub fn commit_len(&self, view: u64) -> usize {
        self.commits.get(&CommitKey(view)).unwrap().len()
    }

    pub fn get_pre_prepare(&self, pre_prepare: &PrePrepare) -> Option<&PrePrepare> {
        self.pre_prepares.get(&PrePrepareKey(pre_prepare.view(), pre_prepare.sequence_number()))
    }

    pub fn get_pre_prepare_by_key(&self, view: u64, sequence_number: u64) -> Option<&PrePrepare> {
        self.pre_prepares.get(&PrePrepareKey(view, sequence_number))
    }
}