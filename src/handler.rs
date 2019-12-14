use libp2p::core::{Negotiated, UpgradeInfo};
use libp2p::swarm::protocols_handler::{KeepAlive, ProtocolsHandlerUpgrErr, ProtocolsHandlerEvent, SubstreamProtocol};
use libp2p::swarm::ProtocolsHandler;
use crate::message::{ClientRequest, MessageType, PrePrepare, Message};
use tokio::prelude::{AsyncRead, AsyncWrite, Async, AsyncSink};
use crate::behavior::PbftFailure;
use futures::Poll;
use futures::sink::Sink;
use futures::stream::Stream;
use crate::protocol_config::{PbftProtocolConfig, PbftOutStreamSink, PbftInStreamSink};
use libp2p::{OutboundUpgrade, InboundUpgrade};
use std::error::Error;

/// Event to send to the handler.
#[derive(Debug)]
pub enum PbftHandlerIn {
    PrePrepareRequest(PrePrepare),
    PrePrepareResponse(Vec<u8>),
}

pub struct PbftHandler<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    config: PbftProtocolConfig,
    substreams: Vec<SubstreamState<Negotiated<TSubstream>>>,
    _marker: std::marker::PhantomData<TSubstream>,
}

enum SubstreamState<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    /// We haven't started opening the outgoing substream yet.
    /// Contains the request we want to send, and the user data if we expect an answer.
    OutPendingOpen(MessageType),
    /// Waiting to send a message to the remote.
    OutPendingSend(PbftOutStreamSink<TSubstream>, MessageType),
    /// Waiting to flush the substream so that the data arrives to the remote.
    OutPendingFlush(PbftOutStreamSink<TSubstream>),
    // TODO: add timeout
    OutWaitingAnswer(PbftOutStreamSink<TSubstream>),
    /// The substream is being closed.
    OutClosing(PbftOutStreamSink<TSubstream>),
    /// Waiting for a request from the remote.
    InWaitingMessage(PbftInStreamSink<TSubstream>),
    /// Waiting for the user to send a `PbftHandlerIn` event containing the response.
    InWaitingUser(PbftInStreamSink<TSubstream>),
    /// Waiting to send an answer back to the remote.
    InPendingSend(PbftInStreamSink<TSubstream>, Vec<u8>),
    /// Waiting to flush an answer back to the remote.
    InPendingFlush(PbftInStreamSink<TSubstream>),
    /// The substream is being closed.
    InClosing(PbftInStreamSink<TSubstream>),
}

#[derive(Debug)]
pub enum PbftHandlerEvent {
    ProcessPrePrepareRequest {
        request: PrePrepare,
    },
    PrePrepareResponse {
        response: Vec<u8>,
    },
}

impl<TSubstream> PbftHandler<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
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
    type OutEvent = PbftHandlerEvent;
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
        self.substreams.push(SubstreamState::InWaitingMessage(protocol));
    }

    fn inject_fully_negotiated_outbound(
        &mut self,
        protocol: <Self::OutboundProtocol as OutboundUpgrade<TSubstream>>::Output,
        message: Self::OutboundOpenInfo,
    ) {
        println!("PbftHandler::inject_fully_negotiated_outbound()");
        self.substreams.push(SubstreamState::OutPendingSend(protocol, message));
    }

    fn inject_event(&mut self, handler_in: PbftHandlerIn) {
        println!("PbftHandler::inject_event() : {:?}", handler_in);
        match handler_in {
            PbftHandlerIn::PrePrepareRequest(request) => {
                self.substreams.push(
                    SubstreamState::OutPendingOpen(MessageType::HandlerPrePrepare(request))
                );
            }
            PbftHandlerIn::PrePrepareResponse(response) => {
                println!("[PbftHandler::inject_event] [PbftHandlerIn::PrePrepareResponse] response: {:?}", response);
                let pos = self.substreams.iter().position(|state| {
                    match state {
                        SubstreamState::InWaitingUser(_) => true, // TODO: select appropriate connection
                        _ => false,
                    }
                });

                if let Some(pos) = pos {
                    let substream = match self.substreams.remove(pos) {
                        SubstreamState::InWaitingUser(substream) => substream,
                        _ => unreachable!(),
                    };
                    self.substreams.push(SubstreamState::InPendingSend(substream, response));
                }
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

    fn poll(&mut self) -> Poll<ProtocolsHandlerEvent<PbftProtocolConfig, MessageType, Self::OutEvent>, Self::Error> {
        println!("[PbftHandler::poll]");

        if let Some(mut substream_state) = self.substreams.pop() {
            println!("[PbftHandler::poll] [substream_state]");

            loop {
                match handle_substream(substream_state, self.config.clone()) {
                    (Some(new_substream_state), None, true) => {
                        println!("[PbftHandler::poll] (Some, None false)");
                        substream_state = new_substream_state;
                        continue;
                    },
                    (Some(new_substream_state), None, false) => {
                        println!("[PbftHandler::poll] (Some, None, false)");
                        self.substreams.push(new_substream_state);
                        break;
                    },
                    (None, Some(protocol_handler_event), _)  => {
                        println!("[PbftHandler::poll] (None, Some, _) protocol_handler_event : {:?}", protocol_handler_event);
                        return Ok(Async::Ready(protocol_handler_event));
                    },
                    (Some(new_substream_state), Some(protocol_handler_event), _) => {
                        println!("[PbftHandler::poll] (Some, Some, _)");
                        self.substreams.push(new_substream_state);
                        return Ok(Async::Ready(protocol_handler_event));
                    }
                    (None, None, _) => {
                        // TODO
                        println!("[PbftHandler::poll] (None, None, _)");
                        break;
                    }
                }
            }
        }

        Ok(Async::NotReady)
    }

}

fn handle_substream<TSubstream>(
    substream_state: SubstreamState<TSubstream>,
    config: PbftProtocolConfig,
) -> (
    Option<SubstreamState<TSubstream>>,
    Option<
        ProtocolsHandlerEvent<
            PbftProtocolConfig,
            MessageType,
            PbftHandlerEvent,
        >,
    >,
    bool, // whether the substream should be polled again
)
where
    TSubstream: AsyncRead + AsyncWrite
{
    match substream_state {
        SubstreamState::OutPendingOpen(message) => {
            println!("[PbftHandler::handle_substream()] [SubstreamState::OutPendingOpen] message: {:?}", message);
            let event = ProtocolsHandlerEvent::OutboundSubstreamRequest {
                protocol: SubstreamProtocol::new(config),
                info: message,
            };
            return (None, Some(event), false);
        }
        SubstreamState::OutPendingSend(mut substream, message) => {
            println!("[PbftHandler::handle_substream()] [SubstreamState::OutPendingSend] message: {:?}", message);
            match substream.start_send(message) {
                Ok(AsyncSink::Ready) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutPendingSend] [start_send::Ready]");
                    (
                        Some(SubstreamState::OutPendingFlush(substream)),
                        None,
                        true,
                    )
                },
                Ok(AsyncSink::NotReady(msg)) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutPendingSend] [start_send::NotReady] msg: {:?}", msg);
                    (
                        Some(SubstreamState::OutPendingSend(substream, msg)),
                        None,
                        false,
                    )
                },
                Err(e) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutPendingSend] [start_send::Err] Err: {:?}", e);
                    (None, None, false) // TODO
                }
            }
        }
        SubstreamState::OutPendingFlush(mut substream) => {
            match substream.poll_complete() {
                Ok(Async::Ready(())) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutPendingFlush] [Ready]");
                    (
                        Some(SubstreamState::OutWaitingAnswer(substream)),
                        None,
                        true,
                    )
                }
                Ok(Async::NotReady) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutPendingFlush] [NotReady]");
                    (
                        Some(SubstreamState::OutPendingFlush(substream)),
                        None,
                        false,
                    )
                }
                Err(e) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutPendingFlush] [Err] Err: {:?}", e);
                    (None, None, false) // TODO
                }
            }
        }
        SubstreamState::OutWaitingAnswer(mut substream) => {
            println!("[PbftHandler::handle_substream()] [SubstreamState::OutWaitingAnswer]");
            match substream.poll() {
                Ok(Async::Ready(Some(response))) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutWaitingAnswer] [Ready::Some] response: {:?}", response);
                    (
                        Some(SubstreamState::OutClosing(substream)),
                        Some(ProtocolsHandlerEvent::Custom(PbftHandlerEvent::PrePrepareResponse { response })), // TODO
                        true,
                    )
                }
                Ok(Async::NotReady) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutWaitingAnswer] [NotReady]");
                    (
                        Some(SubstreamState::OutWaitingAnswer(substream)),
                        None,
                        false,
                    )
                }
                Err(e) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutWaitingAnswer] [Err] Err: {:?}", e);
                    (None, None, false) // TODO
                }
                Ok(Async::Ready(None)) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutWaitingAnswer] [Ready::None]");
                    (None, None, false) // TODO
                }
            }
        }
        SubstreamState::OutClosing(mut substream) => {
            match substream.close() {
                Ok(Async::Ready(())) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutClosing] [Ready]");
                    (None, None, false)
                }
                Ok(Async::NotReady) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutClosing] [NotReady]");
                    (
                        Some(SubstreamState::OutClosing(substream)),
                        None,
                        false,
                    )
                }
                Err(e) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::OutClosing] [Err] Err: {:?}", e);
                    (None, None, false) // TODO
                }
            }
        }
        SubstreamState::InWaitingMessage(mut substream) => {
            match substream.poll() {
                Ok(Async::Ready(Some(msg))) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InWaitingMessage] [Ready(Some)] msg: {:?}", msg);
                    (
                        Some(SubstreamState::InWaitingUser(substream)),
                        Some(ProtocolsHandlerEvent::Custom(message_to_handler_event(msg))),
                        false,
                    )
                },
                Ok(Async::NotReady) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InWaitingMessage] [NotReady]");
                    (
                        Some(SubstreamState::InWaitingMessage(substream)),
                        None,
                        true,
                    )
                },
                Ok(Async::Ready(None)) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InWaitingMessage] [Ready(None)] Inbound substream EOF");
                    (None, None, false)
                },
                Err(e) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InWaitingMessage] [Err] Err: {:?}", e);
                    (None, None, false) // TODO
                }
            }
        }
        SubstreamState::InWaitingUser(mut substream) => {
            println!("[PbftHandler::handle_substream()] [SubstreamState::InWaitingUser]");
            (
                Some(SubstreamState::InWaitingUser(substream)),
                None,
                false,
            )
        }
        SubstreamState::InPendingSend(mut substream, response) => {
            match substream.start_send(response) {
                Ok(AsyncSink::Ready) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InPendingSend] [AsyncSink::Ready]");
                    (
                        Some(SubstreamState::InPendingFlush(substream)),
                        None,
                        true,
                    )
                },
                Ok(AsyncSink::NotReady(response)) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InPendingSend] [AsyncSink::NotReady]");
                    (
                        Some(SubstreamState::InPendingSend(substream, response)),
                        None,
                        false,
                    )
                },
                Err(e) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InPendingSend] [Err]");
                    (None, None, false) // TODO
                }
            }
        }
        SubstreamState::InPendingFlush(mut substream) => {
            match substream.poll_complete() {
                Ok(Async::Ready(())) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InPendingFlush] [Async::Ready]");
                    (
                        Some(SubstreamState::InClosing(substream)),
                        None,
                        true,
                    )
                },
                Ok(Async::NotReady) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InPendingFlush] [Async::NotReady]");
                    (
                        Some(SubstreamState::InPendingFlush(substream)),
                        None,
                        false,
                    )
                },
                Err(e) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InPendingFlush] [Err]");
                    (None, None, false)
                }
            }
        }
        SubstreamState::InClosing(mut substream) => {
            match substream.close() {
                Ok(Async::Ready(())) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InClosing] [Async::Ready]");
                    (None, None, false)
                },
                Ok(Async::NotReady) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InClosing] [Async::NotReady]");
                    (Some(SubstreamState::InClosing(substream)), None, false)
                },
                Err(e) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InClosing] [Err]");
                    (None, None, false) // TODO
                }
            }
        }
    }
}

fn message_to_handler_event(message: MessageType) -> PbftHandlerEvent {
    // TODO
    match message {
        MessageType::HandlerPrePrepare(pre_prepare) => {
            PbftHandlerEvent::ProcessPrePrepareRequest { request: pre_prepare }
        }
        _ => unreachable!()
    }
}