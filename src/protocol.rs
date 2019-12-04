use libp2p::core::ConnectedPoint;
use libp2p::core::ProtocolName;
use libp2p::core::{Negotiated, UpgradeInfo};
use libp2p::swarm::protocols_handler::{KeepAlive, ProtocolsHandlerUpgrErr, ProtocolsHandlerEvent, SubstreamProtocol};
use libp2p::swarm::{PollParameters, ProtocolsHandler, NetworkBehaviour, NetworkBehaviourAction};
use libp2p::multiaddr::Multiaddr;
use void::Void;
use std::error::Error;
use tokio::prelude::{AsyncRead, AsyncWrite, Async};
use futures::Poll;
use libp2p::{PeerId, InboundUpgrade, OutboundUpgrade};
use futures::future::FutureResult;
use std::collections::{VecDeque, HashSet};
use crate::message::ClientRequest;

#[derive(Clone)]
pub struct Name;

impl ProtocolName for Name {
    fn protocol_name(&self) -> &[u8] {
        b"/ackintosh/pbft/1.0.0"
    }
}

pub struct Pbft<TSubstream> {
    connected_peers: HashSet<PeerId>,
    client_requests: VecDeque<ClientRequest>,
    queued_events: VecDeque<NetworkBehaviourAction<Void, PbftEvent>>,
    _marker: std::marker::PhantomData<TSubstream>,
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

    pub fn add_peer(&mut self, peer_id: &PeerId) {
        println!("Pbft::add_address(): {:?}", peer_id);
        self.queued_events.push_back(NetworkBehaviourAction::DialPeer {
            peer_id: peer_id.clone(),
        });
    }

    pub fn add_client_request(&mut self, client_request: ClientRequest) {
        println!("Pbft::add_client_request(): {:?}", client_request);
        self.client_requests.push_back(client_request);
    }
}

impl<TSubstream> UpgradeInfo for Pbft<TSubstream> {
    type Info = Name;
    type InfoIter = std::iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        println!("Pbft::protocol_info()");
        std::iter::once(Name{})
    }
}

impl<TSubstream> InboundUpgrade<TSubstream> for Pbft<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    type Output = ();
    type Error = std::io::Error;
    type Future = FutureResult<Self::Output, std::io::Error>;

    fn upgrade_inbound(
        self,
        socket: Negotiated<TSubstream>,
        info: Self::Info,
    ) -> Self::Future {
        println!("upgrade_inbound");
        futures::future::ok(())
    }
}

impl<TSubstream> OutboundUpgrade<TSubstream> for Pbft<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    type Output = <Self as InboundUpgrade<TSubstream>>::Output;
    type Error = <Self as InboundUpgrade<TSubstream>>::Error;
    type Future = <Self as InboundUpgrade<TSubstream>>::Future;

    fn upgrade_outbound(
        self,
        socket: Negotiated<TSubstream>,
        info: Self::Info,
    ) -> Self::Future {
        println!("upgrade_outbound");
        futures::future::ok(())
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

pub struct PbftHandler<TSubstream> {
    _marker: std::marker::PhantomData<TSubstream>,
}

impl<TSubstream> PbftHandler<TSubstream> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<TSubstream> ProtocolsHandler for PbftHandler<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    type InEvent = Void;
    type OutEvent = PbftEvent;
    type Error = PbftFailure;
    type Substream = TSubstream;
    type InboundProtocol = Pbft<TSubstream>;
    type OutboundProtocol = Pbft<TSubstream>;
    type OutboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Pbft<TSubstream>> {
        println!("PbftHandler::listen_protocol()");
        SubstreamProtocol::new(Pbft::new())
    }

    fn inject_fully_negotiated_inbound(&mut self, _: ()) {
        println!("PbftHandler::inject_fully_negotiated_inbound()");
    }

    fn inject_fully_negotiated_outbound(&mut self, _out: (), _info: ()) {
        println!("PbftHandler::inject_fully_negotiated_outbound()");
    }

    fn inject_event(&mut self, _: Void) {
        println!("PbftHandler::inject_event()");
    }

    fn inject_dial_upgrade_error(&mut self, _info: (), _error: ProtocolsHandlerUpgrErr<std::io::Error>) {
        println!("PbftHandler::inject_dial_upgrade_error()");
    }

    fn connection_keep_alive(&self) -> KeepAlive {
//        println!("PbftHandler::connection_keep_alive()");
        KeepAlive::Yes
    }

    fn poll(&mut self) -> Poll<ProtocolsHandlerEvent<Pbft<TSubstream>, (), PbftEvent>, Self::Error> {
        println!("PbftHandler::poll()");
        Ok(Async::NotReady)
//        Ok(Async::Ready(
//            ProtocolsHandlerEvent::OutboundSubstreamRequest {
//                protocol: SubstreamProtocol::new(Pbft::new()),
//                info: (),
//            }
//        ))
    }
}

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

    fn addresses_of_peer(&mut self, _peer_id: &PeerId) -> Vec<Multiaddr> {
        println!("addresses_of_peer");
        Vec::new()
    }

    fn inject_connected(&mut self, peer_id: PeerId, _: ConnectedPoint) {
        println!("Pbft::inject_connected()");
        println!("Connected to the node: {:?}", peer_id);
        self.connected_peers.insert(peer_id);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, _: ConnectedPoint) {
        println!("Pbft::inject_disconnected()");
        println!("Disconnected to the node: {:?}", peer_id);
        self.connected_peers.remove(peer_id);
    }

    fn inject_node_event(&mut self, _peer: PeerId, _event: PbftEvent) {
        println!("inject_node_event");
    }

    fn poll(&mut self, _: &mut impl PollParameters) -> Async<NetworkBehaviourAction<Void, PbftEvent>>
    {
        println!("Pbft::poll()");
        if let Some(event) = self.queued_events.pop_front() {
            println!("Pbft.queued_evnets.pop_front: {:?}", event);
            return Async::Ready(event);
        }
        Async::NotReady
    }
}