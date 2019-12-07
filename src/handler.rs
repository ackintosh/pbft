use libp2p::swarm::protocols_handler::{KeepAlive, ProtocolsHandlerUpgrErr, ProtocolsHandlerEvent, SubstreamProtocol};
use libp2p::swarm::ProtocolsHandler;
use crate::message::{ClientRequest, MessageType, PrePrepare};
use tokio::prelude::{AsyncRead, AsyncWrite, Async};
use crate::behavior::{PbftEvent, PbftFailure};
use futures::Poll;
use crate::protocol_config::PbftProtocolConfig;
use libp2p::{OutboundUpgrade, InboundUpgrade};
use std::error::Error;

#[derive(Debug)]
pub enum PbftHandlerIn {
    ClientRequest(ClientRequest),
}

pub struct PbftHandler<TSubstream> {
    config: PbftProtocolConfig,
    substreams: Vec<SubstreamState>,
    _marker: std::marker::PhantomData<TSubstream>,
}

enum SubstreamState {
    OutPendingOpen(MessageType),
}

impl<TSubstream> PbftHandler<TSubstream> {
    pub fn new() -> Self {
        Self {
            config: PbftProtocolConfig {},
            substreams: Vec::new(),
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
    type InboundProtocol = PbftProtocolConfig;
    type OutboundProtocol = PbftProtocolConfig;
    type OutboundOpenInfo = MessageType;

    fn listen_protocol(&self) -> SubstreamProtocol<PbftProtocolConfig> {
        println!("PbftHandler::listen_protocol()");
        SubstreamProtocol::new(self.config.clone())
    }

    fn inject_fully_negotiated_inbound(
        &mut self,
        protocol: <Self::InboundProtocol as InboundUpgrade<TSubstream>>::Output,
    ) {
        println!("PbftHandler::inject_fully_negotiated_inbound()");
    }

    fn inject_fully_negotiated_outbound(
        &mut self,
        protocol: <Self::OutboundProtocol as OutboundUpgrade<TSubstream>>::Output,
        _info: MessageType
    ) {
        println!("PbftHandler::inject_fully_negotiated_outbound()");
    }

    fn inject_event(&mut self, handler_in: PbftHandlerIn) {
        println!("PbftHandler::inject_event() : {:?}", handler_in);
        match handler_in {
            PbftHandlerIn::ClientRequest(request) => {
                let message = PrePrepare::from(
                    1, // TODO
                    1, // TODO
                    request.operation()
                );
                self.substreams.push(
                    SubstreamState::OutPendingOpen(MessageType::HandlerPrePrepare(message))
                );
            }
        }
    }

    fn inject_dial_upgrade_error(&mut self, info: MessageType, error: ProtocolsHandlerUpgrErr<std::io::Error>) {
        println!("PbftHandler::inject_dial_upgrade_error(), info: {:?}, error: {:?}", info, error);
    }

    fn connection_keep_alive(&self) -> KeepAlive {
//        println!("PbftHandler::connection_keep_alive()");
        KeepAlive::Yes
    }

    fn poll(&mut self) -> Poll<ProtocolsHandlerEvent<PbftProtocolConfig, MessageType, PbftEvent>, Self::Error> {
        println!("PbftHandler::poll()");
        if let Some(substream) = self.substreams.pop() {
            match substream {
                SubstreamState::OutPendingOpen(message) => {
                    println!("PbftHandler::poll() -> SubstreamState::OutPendingOpen : {:?}", message);
                    let event = ProtocolsHandlerEvent::OutboundSubstreamRequest {
                        protocol: SubstreamProtocol::new(self.config.clone()),
                        info: message,
                    };
                    return Ok(Async::Ready(event));
                }
            }
        }

        Ok(Async::NotReady)
//        Ok(Async::Ready(
//            ProtocolsHandlerEvent::OutboundSubstreamRequest {
//                protocol: SubstreamProtocol::new(Pbft::new()),
//                info: (),
//            }
//        ))
    }
}

