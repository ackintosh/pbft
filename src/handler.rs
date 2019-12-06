use libp2p::swarm::protocols_handler::{KeepAlive, ProtocolsHandlerUpgrErr, ProtocolsHandlerEvent, SubstreamProtocol};
use libp2p::swarm::{PollParameters, ProtocolsHandler, NetworkBehaviour, NetworkBehaviourAction};
use crate::message::ClientRequest;
use tokio::prelude::{AsyncRead, AsyncWrite, Async};
use crate::behavior::{PbftEvent, PbftFailure, Pbft};
use futures::Poll;

#[derive(Debug)]
pub enum PbftHandlerIn {
    ClientRequest(ClientRequest),
}

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
    type InEvent = PbftHandlerIn;
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

    fn inject_event(&mut self, handler_in: PbftHandlerIn) {
        println!("PbftHandler::inject_event() : {:?}", handler_in);
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

