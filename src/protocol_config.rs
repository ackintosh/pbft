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
        info: Self::Info,
    ) -> Self::Future {
        println!("PbftProtocolConfig::upgrade_inbound");
        let mut codec = UviBytes::default();

        futures::future::ok(
            Framed::new(socket, codec)
                .from_err()
                .with::<_, fn(_) -> _, _>(|response| {
                    println!("PbftProtocolConfig::upgrade_inbound -> with, response : {:?}", response);
                    Ok(Vec::new()) // TODO
                })
                .and_then::<fn(_) -> _, _>(|bytes| {
                    println!("PbftProtocolConfig::upgrade_inbound -> and_then");
                    Ok(MessageType::Prepare) // FIXME
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
        info: Self::Info,
    ) -> Self::Future {
        println!("PbftProtocolConfig::upgrade_outbound");
        let mut codec = UviBytes::default();

        futures::future::ok(
            Framed::new(socket, codec)
                .from_err()
                .with::<_, fn(_) -> _, _>(|request| {
                    println!("PbftProtocolConfig::upgrade_outbound -> with, request : {:?}", request);
                    Ok(Vec::new()) // TODO
                })
                .and_then::<fn(_) -> _, _>(|bytes| {
                    println!("PbftProtocolConfig::upgrade_outbound -> and_then");
                    Ok(()) // TODO
                })
        )
    }
}

type PbftInStreamSink<S> = PbftStreamSink<S, (), MessageType>;

type PbftOutStreamSink<S> = PbftStreamSink<S, MessageType, ()>;

type PbftStreamSink<S, A, B> = futures::stream::AndThen<
    futures::sink::With<
        futures::stream::FromErr<Framed<S, UviBytes<Vec<u8>>>, std::io::Error>,
        A,
        fn(A) -> Result<Vec<u8>, std::io::Error>,
        Result<Vec<u8>, std::io::Error>
    >,
    fn(BytesMut) -> Result<B, std::io::Error>,
    Result<B, std::io::Error>,
>;
