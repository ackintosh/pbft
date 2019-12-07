use libp2p::core::ProtocolName;
use libp2p::core::{Negotiated, UpgradeInfo};
use tokio::prelude::{AsyncRead, AsyncWrite};
use libp2p::{InboundUpgrade, OutboundUpgrade};
use futures::future::FutureResult;

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

impl<TSubstream> OutboundUpgrade<TSubstream> for PbftProtocolConfig
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
