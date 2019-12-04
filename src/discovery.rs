use libp2p::mdns::{Mdns, MdnsEvent};
use libp2p::{NetworkBehaviour, PeerId};
use tokio::prelude::{AsyncRead, AsyncWrite};
use std::collections::HashSet;
use std::sync::{RwLock, Arc};
use crate::protocol::{Pbft, PbftEvent};

#[derive(NetworkBehaviour)]
pub struct Discovery<TSubstream: AsyncRead + AsyncWrite> {
    mdns: Mdns<TSubstream>,
    pub pbft: Pbft<TSubstream>,
    #[behaviour(ignore)]
    nodes: Arc<RwLock<HashSet<PeerId>>>,
}

impl<TSubstream: AsyncRead + AsyncWrite> Discovery<TSubstream> {
    pub fn new(mdns: Mdns<TSubstream>, pbft: Pbft<TSubstream>, nodes: Arc<RwLock<HashSet<PeerId>>>) -> Self {
        Self {
            mdns,
            pbft,
            nodes,
        }
    }
}

impl<TSubstream: AsyncRead + AsyncWrite> libp2p::swarm::NetworkBehaviourEventProcess<MdnsEvent>
    for Discovery<TSubstream>
{
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer_id, addr) in list {
                    if self.nodes.write().unwrap().insert(peer_id.clone()) {
                        println!("The node has been discovered: {:?}", addr);
                        self.pbft.add_peer(&peer_id);
                    }
                }
            },
            MdnsEvent::Expired(list) => {
                for (peer_id, addr) in list {
                    if self.nodes.write().unwrap().remove(&peer_id) {
                        println!("The node has been expired: {:?}", addr);
                    }
                }
            }
        }
    }
}

impl<TSubstream: AsyncRead + AsyncWrite> libp2p::swarm::NetworkBehaviourEventProcess<PbftEvent>
    for Discovery<TSubstream>
{
    fn inject_event(&mut self, event: PbftEvent) {
        println!("inject_event : PbftEvent: {:?}", event);
    }
}
