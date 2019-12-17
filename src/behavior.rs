use libp2p::core::ConnectedPoint;
use libp2p::swarm::{PollParameters, NetworkBehaviour, NetworkBehaviourAction};
use libp2p::multiaddr::Multiaddr;
use std::error::Error;
use tokio::prelude::{AsyncRead, AsyncWrite, Async};
use futures::Poll;
use libp2p::PeerId;
use futures::future::FutureResult;
use std::collections::{VecDeque, HashSet, HashMap};
use crate::message::{ClientRequest, PrePrepareSequence, PrePrepare, Prepare};
use crate::handler::{PbftHandlerIn, PbftHandler, PbftHandlerEvent};
use crate::state::State;

pub struct Pbft<TSubstream> {
    addresses: HashMap<PeerId, HashSet<Multiaddr>>,
    connected_peers: HashSet<PeerId>,
    client_requests: VecDeque<ClientRequest>,
    queued_events: VecDeque<NetworkBehaviourAction<PbftHandlerIn, PbftEvent>>,
    state: State,
    pre_prepare_sequence: PrePrepareSequence,
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
            addresses: HashMap::new(),
            connected_peers: HashSet::new(),
            client_requests: VecDeque::with_capacity(100), // FIXME
            queued_events: VecDeque::with_capacity(100), // FIXME
            state: State::new(),
            pre_prepare_sequence: PrePrepareSequence::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn has_peer(&self, peer_id: &PeerId) -> bool {
        self.connected_peers.iter().any(|connected_peer_id| {
            connected_peer_id.clone() == peer_id.clone()
        })
    }

    pub fn add_peer(&mut self, peer_id: &PeerId, address: &Multiaddr) {
        println!("[Pbft::add_peer] {:?}, {:?}", peer_id, address);
        {
            let mut addresses = match self.addresses.get(peer_id) {
                Some(addresses) => addresses.clone(),
                None => HashSet::new(),
            };
            addresses.insert(address.clone());

            self.addresses.insert(peer_id.clone(), addresses.clone());
        }

        self.queued_events.push_back(NetworkBehaviourAction::DialPeer {
            peer_id: peer_id.clone(),
        });
    }

    pub fn add_client_request(&mut self, client_request: ClientRequest) {
        println!("[Pbft::add_client_request] client_request: {:?}", client_request);

        // In the pre-prepare phase, the primary assigns a sequence number, n, to the request
        self.pre_prepare_sequence.increment();
        let pre_prepare = PrePrepare::from(
            self.state.current_view(),
            self.pre_prepare_sequence.value(),
            client_request.operation(),
        );

        println!("[Pbft::add_client_request] [broadcasting the pre_prepare message] pre_prepare: {:?}", pre_prepare);
        println!("[Pbft::add_client_request] [broadcasting to the peers] connected_peers: {:?}", self.connected_peers);
        if self.connected_peers.is_empty() {
            panic!("[Pbft::add_client_request] !!! connected_peers is empty !!!");
        }

        for peer_id in self.connected_peers.iter() {
            self.queued_events.push_back(NetworkBehaviourAction::SendEvent {
                peer_id: peer_id.clone(),
                event: PbftHandlerIn::PrePrepareRequest(pre_prepare.clone())
            });
        }

        self.process_pre_prepare(pre_prepare).unwrap(); // TODO: error handling
    }

    fn process_pre_prepare(&mut self, pre_prepare: PrePrepare) -> Result<(), String> {
        self.validate_pre_prepare(&pre_prepare)?;
        self.state.insert_pre_prepare(pre_prepare.clone());

        // If backup replica accepts the message, it enters the prepare phase by multicasting a PREPARE message to
        // all other replicas and adds both messages to its log.
        let prepare = Prepare::from(&pre_prepare);
        self.state.insert_prepare(prepare.clone());

        if self.connected_peers.is_empty() {
            panic!("[Pbft::inject_node_event] [PbftHandlerEvent::PrePrepareRequest] !!! Peers not found !!!");
        }

        for peer_id in self.connected_peers.iter() {
            println!("[Pbft::inject_node_event] [PbftHandlerEvent::PrePrepareRequest] [add queued_events] peer_id: {:?}", peer_id);
            self.queued_events.push_back(NetworkBehaviourAction::SendEvent {
                peer_id: peer_id.clone(),
                event: PbftHandlerIn::PrepareRequest(prepare.clone())
            })
        }
        Ok(())
    }

    fn validate_pre_prepare(&self, pre_prepare: &PrePrepare) -> Result<(), String> {
        // TODO: the signatures in the request and the pre-prepare message are correct

        // _d_ is the digest for _m_
        pre_prepare.validate_digest()?;

        {
            // it is in view _v_
            let current_view = self.state.current_view();
            if pre_prepare.view() != current_view {
                return Err(format!("view number isn't matched. message: {}, state: {}", pre_prepare.view(), current_view));
            }

            // it has not accepted a pre-prepare message for view _v_ and sequence number _n_ containing a different digest
            match self.state.get_pre_prepare(pre_prepare) {
                Some(stored_pre_prepare) => {
                    if pre_prepare.digest() != stored_pre_prepare.digest() {
                        return Err(format!("The pre-prepare key has already stored into logs and its digest dont match. message: {}, stored message: {}", pre_prepare, stored_pre_prepare));
                    }
                }
                None => {}
            }
        }

        // TODO: the sequence number in the pre-prepare message is between a low water mark, _h_, and a high water mark, _H_

        Ok(())
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
        println!("[Pbft::addresses_of_peer] peer_id: {:?}", peer_id);
        match self.addresses.get(peer_id) {
            Some(addresses) => {
                println!("[Pbft::addresses_of_peer] peer_id: {:?}, addresses: {:?}", peer_id, addresses);
                addresses.clone().into_iter().collect()
            },
            None => {
                println!("[Pbft::addresses_of_peer] addresses not found. peer_id: {:?}", peer_id);
                Vec::new()
            }
        }
    }

    fn inject_connected(&mut self, peer_id: PeerId, connected_point: ConnectedPoint) {
        println!("[Pbft::inject_connected] peer_id: {:?}, connected_point: {:?}", peer_id, connected_point);
//        match connected_point {
//            ConnectedPoint::Dialer { address } => {
//            },
//            ConnectedPoint::Listener { .. } => {}
//        };
        self.connected_peers.insert(peer_id);
        println!("[Pbft::inject_connected] connected_peers: {:?}, addresses: {:?}", self.connected_peers, self.addresses);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, connected_point: ConnectedPoint) {
        println!("[Pbft::inject_disconnected] {:?}, {:?}", peer_id, connected_point);
        let address = match connected_point {
            ConnectedPoint::Dialer { address } => address,
            ConnectedPoint::Listener { local_addr: _, send_back_addr } => send_back_addr
        };
        self.connected_peers.remove(peer_id);
        println!("[Pbft::inject_disconnected] connected_peers: {:?}, addresses: {:?}", self.connected_peers, self.addresses);
    }

    fn inject_node_event(&mut self, peer_id: PeerId, handler_event: PbftHandlerEvent) {
        println!("[Pbft::inject_node_event] handler_event: {:?}", handler_event);
        match handler_event {
            PbftHandlerEvent::ProcessPrePrepareRequest { request, connection_id } => {
                println!("[Pbft::inject_node_event] [PbftHandlerEvent::PrePrepareRequest] request: {:?}", request);
                self.process_pre_prepare(request.clone()).unwrap(); // TODO: error handling

                self.queued_events.push_back(NetworkBehaviourAction::SendEvent {
                    peer_id,
                    event: PbftHandlerIn::PrePrepareResponse("OK".into(), connection_id),
                });
            }
            PbftHandlerEvent::PrePrepareResponse { response } => {
                // TODO: handle the response
                let response_message = String::from_utf8(response).expect("Failed to parse response");
                println!("[Pbft::inject_node_event] [PbftHandlerEvent::PrePrepareResponse] response_message: {:?}", response_message);
                if response_message == "OK" {
                    println!("[Pbft::inject_node_event] [PbftHandlerEvent::PrePrepareResponse] the communications has done successfully")
                } else {
                    // TODO: retry?
                    println!("[Pbft::inject_node_event] [PbftHandlerEvent::PrePrepareResponse] response_message: {:?}", response_message);
                }
            }
            PbftHandlerEvent::ProcessPrepareRequest { request, connection_id } => {
                println!("[Pbft::inject_node_event] [PbftHandlerEvent::ProcessPrepareRequest] request: {:?}", request);
                // TODO: process the message

                self.queued_events.push_back(NetworkBehaviourAction::SendEvent {
                    peer_id,
                    event: PbftHandlerIn::PrepareResponse("OK".into(), connection_id)
                });
            }
        }
    }

    fn poll(&mut self, _: &mut impl PollParameters) -> Async<NetworkBehaviourAction<PbftHandlerIn, PbftEvent>> {
        println!("[Pbft::poll]");
        if let Some(event) = self.queued_events.pop_front() {
            println!("[Pbft::poll] event: {:?}", event);
            return Async::Ready(event);
        }
        Async::NotReady
    }
}