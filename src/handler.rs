use libp2p::core::Negotiated;
use libp2p::swarm::protocols_handler::{KeepAlive, ProtocolsHandlerUpgrErr, ProtocolsHandlerEvent, SubstreamProtocol};
use libp2p::swarm::ProtocolsHandler;
use crate::message::{Message, PrePrepare, Prepare, Commit};
use tokio::prelude::{AsyncRead, AsyncWrite, Async, AsyncSink};
use crate::behavior::PbftFailure;
use futures::Poll;
use futures::sink::Sink;
use futures::stream::Stream;
use crate::protocol_config::{PbftProtocolConfig, PbftOutStreamSink, PbftInStreamSink};
use libp2p::{OutboundUpgrade, InboundUpgrade};
use std::collections::VecDeque;

/// Event to send to the handler.
#[derive(Debug)]
pub enum PbftHandlerIn {
    PrePrepareRequest(PrePrepare),
    PrePrepareResponse(Vec<u8>, ConnectionId),
    PrepareRequest(Prepare),
    PrepareResponse(Vec<u8>, ConnectionId),
    CommitRequest(Commit),
    CommitResponse(Vec<u8>, ConnectionId),
}

pub struct PbftHandler<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    config: PbftProtocolConfig,
    substreams: VecDeque<SubstreamState<Negotiated<TSubstream>>>,
    next_connection_id: ConnectionId,
    _marker: std::marker::PhantomData<TSubstream>,
}

/// Unique identifier for a connection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionId(u64);

impl ConnectionId {
    fn new() -> Self {
        Self(0)
    }

    fn next_id(&mut self) -> Self {
        let next = self.0;
        self.0 += 1;
        println!("[ConnectionId::next_id] issued the connection_id: {:?}", next);
        Self(next)
    }
}

enum SubstreamState<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    /// We haven't started opening the outgoing substream yet.
    /// Contains the request we want to send, and the user data if we expect an answer.
    OutPendingOpen(Message),
    /// Waiting to send a message to the remote.
    OutPendingSend(PbftOutStreamSink<TSubstream>, Message),
    /// Waiting to flush the substream so that the data arrives to the remote.
    OutPendingFlush(PbftOutStreamSink<TSubstream>),
    // TODO: add timeout
    OutWaitingAnswer(PbftOutStreamSink<TSubstream>),
    /// The substream is being closed.
    OutClosing(PbftOutStreamSink<TSubstream>),
    /// Waiting for a request from the remote.
    InWaitingMessage(ConnectionId, PbftInStreamSink<TSubstream>),
    /// Waiting to send a `PbftHandlerIn` event containing the response.
    InWaitingToProcessMessage(ConnectionId, PbftInStreamSink<TSubstream>),
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
        connection_id: ConnectionId,
    },
    Response {
        response: Vec<u8>,
    },
    ProcessPrepareRequest {
        request: Prepare,
        connection_id: ConnectionId,
    },
    ProcessCommitRequest {
        request: Commit,
        connection_id: ConnectionId,
    },
}

impl<TSubstream> PbftHandler<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite
{
    pub fn new() -> Self {
        Self {
            config: PbftProtocolConfig {},
            substreams: VecDeque::with_capacity(100), // FIXME
            next_connection_id: ConnectionId::new(),
            _marker: std::marker::PhantomData,
        }
    }

    fn find_waiting_substream_state_pos(&self, connection_id: &ConnectionId) -> Option<usize> {
        self.substreams.iter().position(|state| {
            match state {
                SubstreamState::InWaitingToProcessMessage(substream_connection_id, _) => {
                    substream_connection_id == connection_id
                }
                _ => false
            }
        })
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
    type OutboundOpenInfo = Message;

    fn listen_protocol(&self) -> SubstreamProtocol<PbftProtocolConfig> {
        println!("PbftHandler::listen_protocol()");
        SubstreamProtocol::new(self.config.clone())
    }

    fn inject_fully_negotiated_inbound(
        &mut self,
        protocol: <Self::InboundProtocol as InboundUpgrade<TSubstream>>::Output,
    ) {
        println!("PbftHandler::inject_fully_negotiated_inbound()");
        self.substreams.push_back(SubstreamState::InWaitingMessage(self.next_connection_id.next_id(), protocol));
    }

    fn inject_fully_negotiated_outbound(
        &mut self,
        protocol: <Self::OutboundProtocol as OutboundUpgrade<TSubstream>>::Output,
        message: Self::OutboundOpenInfo,
    ) {
        println!("PbftHandler::inject_fully_negotiated_outbound()");
        self.substreams.push_back(SubstreamState::OutPendingSend(protocol, message));
    }

    fn inject_event(&mut self, handler_in: PbftHandlerIn) {
        println!("[PbftHandler::inject_event] handler_in: {:?}", handler_in);
        match handler_in {
            PbftHandlerIn::PrePrepareRequest(request) => {
                self.substreams.push_back(
                    SubstreamState::OutPendingOpen(Message::PrePrepare(request))
                );
            }
            PbftHandlerIn::PrePrepareResponse(response, connection_id) => {
                println!("[PbftHandler::inject_event] [PbftHandlerIn::PrePrepareResponse] response: {:?}, connection_id: {:?}", response, connection_id);
                let pos = self.substreams.iter().position(|state| {
                    match state {
                        SubstreamState::InWaitingToProcessMessage(substream_connection_id, _) => substream_connection_id.clone() == connection_id,
                        _ => false,
                    }
                });

                if let Some(pos) = pos {
                    let (_connection_id, substream) = match self.substreams.remove(pos) {
                        Some(SubstreamState::InWaitingToProcessMessage(connection_id, substream)) => (connection_id, substream),
                        _ => unreachable!(),
                    };
                    self.substreams.push_back(SubstreamState::InPendingSend(substream, response));
                } else {
                    panic!("[PbftHandler::inject_event] [PbftHandlerIn::PrePrepareResponse] substream state is not found, pos: {:?}, connection_id: {:?}", pos, connection_id);
                }
            }
            PbftHandlerIn::PrepareRequest(request) => {
                println!("[PbftHandler::inject_event] [PbftHandlerIn::PrepareRequest] request: {:?}", request);
                self.substreams.push_back(
                    SubstreamState::OutPendingOpen(Message::Prepare(request))
                );
            }
            PbftHandlerIn::PrepareResponse(response, connection_id) => {
                println!("[PbftHandler::inject_event] [PbftHandlerIn::PrepareResponse] response: {:?}, connection_id: {:?}", response, connection_id);

                if let Some(pos) = self.find_waiting_substream_state_pos(&connection_id) {
                    let (_connection_id, substream) = match self.substreams.remove(pos) {
                        Some(SubstreamState::InWaitingToProcessMessage(connection_id, substream)) => (connection_id, substream),
                        _ => unreachable!(),
                    };
                    self.substreams.push_back(SubstreamState::InPendingSend(substream, response));
                } else {
                    panic!("[PbftHandler::inject_event] [PbftHandlerIn::PrepareResponse] substream state is not found, connection_id: {:?}", connection_id);
                }
            }
            PbftHandlerIn::CommitRequest(request) => {
                println!("[PbftHandler::inject_event] [PbftHandlerIn::CommitRequest] request: {:?}", request);
                self.substreams.push_back(
                    SubstreamState::OutPendingOpen(Message::Commit(request))
                )
            }
            PbftHandlerIn::CommitResponse(response, connection_id) => {
                println!("[PbftHandler::inject_event] [PbftHandlerIn::CommitResponse] response: {:?}, connection_id: {:?}", response, connection_id);

                if let Some(pos) = self.find_waiting_substream_state_pos(&connection_id) {
                    let (_connection_id, substream) = match self.substreams.remove(pos) {
                        Some(SubstreamState::InWaitingToProcessMessage(connection_id, substream)) => (connection_id, substream),
                        _ => unreachable!(),
                    };
                    self.substreams.push_back(SubstreamState::InPendingSend(substream, response));
                } else {
                    panic!("[PbftHandler::inject_event] [PbftHandlerIn::CommitResponse] substream state is not found, connection_id: {:?}", connection_id);
                }
            }
        }
    }

    fn inject_dial_upgrade_error(&mut self, info: Message, error: ProtocolsHandlerUpgrErr<std::io::Error>) {
        println!("PbftHandler::inject_dial_upgrade_error(), info: {:?}, error: {:?}", info, error);
    }

    fn connection_keep_alive(&self) -> KeepAlive {
//        println!("PbftHandler::connection_keep_alive()");
        KeepAlive::Yes
    }

    fn poll(&mut self) -> Poll<ProtocolsHandlerEvent<PbftProtocolConfig, Message, Self::OutEvent>, Self::Error> {
        println!("[PbftHandler::poll]");

        for _ in 0..self.substreams.len() {
            if let Some(mut substream_state) = self.substreams.pop_front() {
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
                            self.substreams.push_back(new_substream_state);
                            break;
                        },
                        (None, Some(protocol_handler_event), _)  => {
                            println!("[PbftHandler::poll] (None, Some, _) protocol_handler_event : {:?}", protocol_handler_event);
                            return Ok(Async::Ready(protocol_handler_event));
                        },
                        (Some(new_substream_state), Some(protocol_handler_event), _) => {
                            println!("[PbftHandler::poll] (Some, Some, _)");
                            self.substreams.push_back(new_substream_state);
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
            Message,
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
                        Some(ProtocolsHandlerEvent::Custom(PbftHandlerEvent::Response { response })),
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
        SubstreamState::InWaitingMessage(connection_id, mut substream) => {
            match substream.poll() {
                Ok(Async::Ready(Some(msg))) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InWaitingMessage] [Ready(Some)] msg: {:?}", msg);
                    (
                        Some(SubstreamState::InWaitingToProcessMessage(connection_id.clone(), substream)),
                        Some(ProtocolsHandlerEvent::Custom(message_to_handler_event(msg, connection_id))),
                        false,
                    )
                },
                Ok(Async::NotReady) => {
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InWaitingMessage] [NotReady]");
                    (
                        Some(SubstreamState::InWaitingMessage(connection_id, substream)),
                        None,
                        false,
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
        SubstreamState::InWaitingToProcessMessage(connection_id, substream) => {
            println!("[PbftHandler::handle_substream()] [SubstreamState::InWaitingUser]");
            (
                Some(SubstreamState::InWaitingToProcessMessage(connection_id, substream)),
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
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InPendingSend] [Err]: {:?}", e);
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
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InPendingFlush] [Err]: {:?}", e);
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
                    println!("[PbftHandler::handle_substream()] [SubstreamState::InClosing] [Err]: {:?}", e);
                    (None, None, false) // TODO
                }
            }
        }
    }
}

fn message_to_handler_event(
    message: Message,
    connection_id: ConnectionId,
) -> PbftHandlerEvent {
    match message {
        Message::PrePrepare(pre_prepare) => {
            PbftHandlerEvent::ProcessPrePrepareRequest { request: pre_prepare, connection_id }
        }
        Message::Prepare(prepare) => {
            PbftHandlerEvent::ProcessPrepareRequest { request: prepare, connection_id }
        }
        Message::Commit(commit) => {
            PbftHandlerEvent::ProcessCommitRequest { request: commit, connection_id }
        }
        Message::ClientRequest(_) => unreachable!()
    }
}