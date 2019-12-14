use libp2p::core::ConnectedPoint;
use libp2p::swarm::{PollParameters, NetworkBehaviour, NetworkBehaviourAction};
use libp2p::multiaddr::Multiaddr;
use std::error::Error;
use tokio::prelude::{AsyncRead, AsyncWrite, Async};
use futures::Poll;
use libp2p::PeerId;
use futures::future::FutureResult;
use std::collections::{VecDeque, HashSet};
use crate::message::ClientRequest;
use crate::handler::{PbftHandlerIn, PbftHandler, PbftHandlerEvent};

pub struct Pbft<TSubstream> {
    connected_peers: HashSet<Peer>,
    client_requests: VecDeque<ClientRequest>,
    queued_events: VecDeque<NetworkBehaviourAction<PbftHandlerIn, PbftEvent>>,
    _marker: std::marker::PhantomData<TSubstream>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct Peer {
    peer_id: PeerId,
    address: Multiaddr,
}

impl Peer {
    fn new(peer_id: PeerId, address: Multiaddr) -> Self {
        Self {
            peer_id,
            address,
        }
    }
}

impl<TSubstream> Pbft<TSubstream> {
    pub fn new() -> Self {
        Self {
            connected_peers: HashSet::new(),
            client_requests: VecDeque::with_capacity(100), // FIXME
            queued_events: VecDeque::with_capacity(100), // FIXME
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_peer(&mut self, peer_id: &PeerId, address: &Multiaddr) {
        println!("Pbft::add_address(): {:?}, {:?}", peer_id, address);
        self.queued_events.push_back(NetworkBehaviourAction::DialPeer {
            peer_id: peer_id.clone(),
        });
    }

    pub fn add_client_request(&mut self, client_request: ClientRequest) {
        println!("Pbft::add_client_request(): {:?}", client_request);
        for peer in self.connected_peers.iter() {
            println!("Pbft::add_client_request(): queued PbftHandlerIn : peer -> {:?}", peer);
            self.queued_events.push_back(NetworkBehaviourAction::SendEvent {
                peer_id: peer.peer_id.clone(),
                event: PbftHandlerIn::ClientRequest(client_request.clone()),
            });
        }
    }
}

#[derive(Debug)]
pub struct PbftFailure;
impl Error for PbftFailure {
}

impl std::fmt::Display for PbftFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("pbft failure")
    }
}

#[derive(Debug)]
pub struct PbftEvent;

impl<TSubstream> NetworkBehaviour for Pbft<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    type ProtocolsHandler = PbftHandler<TSubstream>;
    type OutEvent = PbftEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        println!("Pbft::new_handler()");
        PbftHandler::new()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        Vec::new() // TODO?
//        let addresses = self.connected_peers.iter().filter(|peer| {
//            peer.peer_id != peer_id.clone()
//        }).map(|peer| {
//            peer.address.clone()
//        }).collect();
//
//        println!("Pbft::addresses_of_peer() : {:?}", addresses);
//        addresses
    }

    fn inject_connected(&mut self, peer_id: PeerId, connected_point: ConnectedPoint) {
        println!("Pbft::inject_connected()");
        let address = match connected_point {
            ConnectedPoint::Dialer { address } => address,
            ConnectedPoint::Listener { local_addr: _, send_back_addr } => send_back_addr
        };
        let peer = Peer::new(peer_id, address);
        println!("Connected to the peer: {:?}", peer);
        self.connected_peers.insert(peer);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, connected_point: ConnectedPoint) {
        println!("Pbft::inject_disconnected()");
        let address = match connected_point {
            ConnectedPoint::Dialer { address } => address,
            ConnectedPoint::Listener { local_addr: _, send_back_addr } => send_back_addr
        };
        let peer = Peer::new(peer_id.clone(), address);
        println!("Disconnected to the peer: {:?}", peer);
        self.connected_peers.remove(&peer);
    }

    fn inject_node_event(&mut self, peer_id: PeerId, handler_event: PbftHandlerEvent) {
        println!("[Pbft::inject_node_event] handler_event: {:?}", handler_event);
        match handler_event {
            PbftHandlerEvent::PrePrepareRequest { message } => {
                // TODO
                println!("[Pbft::inject_node_event] [PbftHandlerEvent::PrePrepareRequest] message: {:?}", message);
                self.queued_events.push_back(NetworkBehaviourAction::SendEvent {
                    peer_id,
                    event: PbftHandlerIn::PrePrepareResponse("OK".into()),
                });
            }
            PbftHandlerEvent::PrePrepareResponse { message } => {
                // TODO
                println!("[Pbft::inject_node_event] [PbftHandlerEvent::PrePrepareResponse] message: {:?}", message);
            }
        }
    }

    fn poll(&mut self, _: &mut impl PollParameters) -> Async<NetworkBehaviourAction<PbftHandlerIn, PbftEvent>> {
        println!("Pbft::poll()");
        if let Some(event) = self.queued_events.pop_front() {
            println!("Pbft.queued_evnets.pop_front: {:?}", event);
            return Async::Ready(event);
        }
        Async::NotReady
    }
}