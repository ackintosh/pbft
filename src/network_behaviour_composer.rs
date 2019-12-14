use libp2p::mdns::{Mdns, MdnsEvent};
use libp2p::{NetworkBehaviour, PeerId};
use tokio::prelude::{AsyncRead, AsyncWrite};
use std::collections::HashSet;
use std::sync::{RwLock, Arc};
use crate::behavior::{Pbft, PbftEvent};

#[derive(NetworkBehaviour)]
pub struct NetworkBehaviourComposer<TSubstream: AsyncRead + AsyncWrite> {
    mdns: Mdns<TSubstream>,
    pub pbft: Pbft<TSubstream>,
}

impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourComposer<TSubstream> {
    pub fn new(mdns: Mdns<TSubstream>, pbft: Pbft<TSubstream>) -> Self {
        Self {
            mdns,
            pbft,
        }
    }
}

impl<TSubstream: AsyncRead + AsyncWrite> libp2p::swarm::NetworkBehaviourEventProcess<MdnsEvent>
    for NetworkBehaviourComposer<TSubstream>
{
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer_id, address) in list {
                    if !self.pbft.has_peer(&peer_id) {
                        println!("The node has been discovered: {:?}", address);
                        self.pbft.add_peer(&peer_id, &address);
                    }
                }
            },
            MdnsEvent::Expired(list) => {
                for (peer_id, addr) in list {
                    if self.pbft.has_peer(&peer_id) {
                        println!("The node has been expired: {:?}", addr);
                        // TODO
                    }
                }
            }
        }
    }
}

impl<TSubstream: AsyncRead + AsyncWrite> libp2p::swarm::NetworkBehaviourEventProcess<PbftEvent>
    for NetworkBehaviourComposer<TSubstream>
{
    fn inject_event(&mut self, event: PbftEvent) {
        println!("inject_event : PbftEvent: {:?}", event);
    }
}
