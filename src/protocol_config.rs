use bytes::BytesMut;
use libp2p::core::ProtocolName;
use libp2p::core::{Negotiated, UpgradeInfo};
use tokio::prelude::{AsyncRead, AsyncWrite};
use libp2p::{InboundUpgrade, OutboundUpgrade};
use futures::future::FutureResult;
use tokio::codec::Framed;
use unsigned_varint::codec::UviBytes;
use crate::message::MessageType;
use futures::{Stream, Sink};

#[derive(Clone)]
pub struct Name;

impl ProtocolName for Name {
    fn protocol_name(&self) -> &[u8] {
        b"/ackintosh/pbft/1.0.0"
    }
}

#[derive(Clone, Debug)]
pub struct PbftProtocolConfig;

impl UpgradeInfo for PbftProtocolConfig {
    type Info = Name;
    type InfoIter = std::iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        println!("Pbft::protocol_info()");
        std::iter::once(Name{})
    }
}

impl<TSubstream> InboundUpgrade<TSubstream> for PbftProtocolConfig
where
    TSubstream: AsyncRead + AsyncWrite
{
    type Output = PbftInStreamSink<Negotiated<TSubstream>>;
    type Error = std::io::Error;
    type Future = FutureResult<Self::Output, std::io::Error>;

    fn upgrade_inbound(
        self,
        socket: Negotiated<TSubstream>,
        _info: Self::Info,
    ) -> Self::Future {
        println!("PbftProtocolConfig::upgrade_inbound");
        let codec = UviBytes::default();

        // TODO: Protocol Buffers
        futures::future::ok(
            Framed::new(socket, codec)
                .from_err()
                .with::<_, fn(_) -> _, _>(|response| {
                    println!("[PbftProtocolConfig::upgrade_inbound] [with] response: {:?}", response);
                    Ok(response)
                })
                .and_then::<fn(_) -> _, _>(|bytes| {
                    println!("[PbftProtocolConfig::upgrade_inbound] [and_then]");
                    Ok(bytes_to_message(&bytes))
                })
        )
    }
}

impl<TSubstream> OutboundUpgrade<TSubstream> for PbftProtocolConfig
where
    TSubstream: AsyncRead + AsyncWrite
{
    type Output = PbftOutStreamSink<Negotiated<TSubstream>>;
    type Error = <Self as InboundUpgrade<TSubstream>>::Error;
    type Future = FutureResult<Self::Output, std::io::Error>;

    fn upgrade_outbound(
        self,
        socket: Negotiated<TSubstream>,
        _info: Self::Info,
    ) -> Self::Future {
        println!("[PbftProtocolConfig::upgrade_outbound]");
        let codec = UviBytes::default();

        // TODO: Protocol Buffers
        futures::future::ok(
            Framed::new(socket, codec)
                .from_err()
                .with::<_, fn(_) -> _, _>(|outbound_message| {
                    println!("[PbftProtocolConfig::upgrade_outbound] [with] outbound_message : {:?}", outbound_message);
                    Ok(message_to_json(&outbound_message).into_bytes())
                })
                .and_then::<fn(_) -> _, _>(|bytes| {
                    println!("[PbftProtocolConfig::upgrade_outbound] [and_then]");
                    Ok(bytes.to_vec())
                })
        )
    }
}

pub type PbftInStreamSink<S> = PbftStreamSink<S, Vec<u8>, MessageType>;

pub type PbftOutStreamSink<S> = PbftStreamSink<S, MessageType, Vec<u8>>;

pub type PbftStreamSink<S, A, B> = futures::stream::AndThen<
    futures::sink::With<
        futures::stream::FromErr<Framed<S, UviBytes<Vec<u8>>>, std::io::Error>,
        A,
        fn(A) -> Result<Vec<u8>, std::io::Error>,
        Result<Vec<u8>, std::io::Error>
    >,
    fn(BytesMut) -> Result<B, std::io::Error>,
    Result<B, std::io::Error>,
>;

fn message_to_json(message: &MessageType) -> String {
    let json = match message {
        MessageType::PrePrepare(_) | MessageType::Prepare(_) => {
            message.to_string()
        }
        MessageType::ClientRequest(_) => unreachable!()
    };
    println!("[protocol_config::message_to_json] json: {:?}", json);
    return json;
}

fn bytes_to_message(bytes: &BytesMut) -> MessageType {
    let message = bytes.to_vec().into();
    println!("[protocol_config::bytes_to_message] message: {:?}", message);
    return message;
}