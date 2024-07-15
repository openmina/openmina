#![forbid(unsafe_code)]

pub use libp2p::identity::{ed25519, Keypair};
use libp2p::swarm::NetworkBehaviour;
use libp2p::Swarm;
use libp2p::{core::upgrade, noise, pnet, tcp, yamux, Transport};
use libp2p::{
    futures::{AsyncRead, AsyncWrite},
    identity, Multiaddr, SwarmBuilder,
};

pub use libp2p::futures;

/// Create a new random identity.
/// Use the same identity type as `Mina` uses.
pub fn generate_identity() -> Keypair {
    identity::Keypair::generate_ed25519()
}

/// Create and configure a libp2p swarm. This will be able to talk to the Mina node.
pub fn swarm<B, I, J>(
    local_key: Keypair,
    chain_id: &[u8],
    listen_on: J,
    peers: I,
    behaviour: B,
) -> Swarm<B>
where
    B: NetworkBehaviour,
    I: IntoIterator<Item = Multiaddr>,
    J: IntoIterator<Item = Multiaddr>,
{
    let pnet_key = {
        use blake2::{
            digest::{generic_array::GenericArray, Update, VariableOutput},
            Blake2bVar,
        };

        let mut key = GenericArray::default();
        Blake2bVar::new(32)
            .expect("valid constant")
            .chain(chain_id)
            .finalize_variable(&mut key)
            .expect("good buffer size");
        key.into()
    };
    let yamux = {
        use libp2p::core::{
            upgrade::{InboundConnectionUpgrade, OutboundConnectionUpgrade},
            UpgradeInfo,
        };
        use std::{
            io,
            pin::Pin,
            task::{self, Context, Poll},
        };

        #[derive(Clone)]
        struct CodaYamux(yamux::Config);

        pin_project_lite::pin_project! {
            struct SocketWrapper<C> {
                #[pin]
                inner: C,
            }
        }

        impl<C> AsyncWrite for SocketWrapper<C>
        where
            C: AsyncWrite,
        {
            fn poll_write(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &[u8],
            ) -> Poll<io::Result<usize>> {
                let this = self.project();
                let len = task::ready!(this.inner.poll_write(cx, buf))?;
                if len != 0 {
                    log::debug!("<- {}", hex::encode(&buf[..len]));
                }

                Poll::Ready(Ok(len))
            }

            fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
                let this = self.project();
                this.inner.poll_flush(cx)
            }

            fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
                let this = self.project();
                this.inner.poll_close(cx)
            }
        }

        impl<C> AsyncRead for SocketWrapper<C>
        where
            C: AsyncRead,
        {
            fn poll_read(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &mut [u8],
            ) -> Poll<io::Result<usize>> {
                let this = self.project();
                let len = task::ready!(this.inner.poll_read(cx, buf))?;
                if len != 0 {
                    log::debug!("-> {}", hex::encode(&buf[..len]));
                }

                Poll::Ready(Ok(len))
            }
        }

        impl UpgradeInfo for CodaYamux {
            type Info = &'static str;
            type InfoIter = std::iter::Once<Self::Info>;

            fn protocol_info(&self) -> Self::InfoIter {
                std::iter::once("/coda/yamux/1.0.0")
            }
        }

        impl<C> InboundConnectionUpgrade<C> for CodaYamux
        where
            C: AsyncRead + AsyncWrite + Send + Unpin + 'static,
        {
            type Output = <yamux::Config as InboundConnectionUpgrade<SocketWrapper<C>>>::Output;
            type Error = <yamux::Config as InboundConnectionUpgrade<C>>::Error;
            type Future = <yamux::Config as InboundConnectionUpgrade<SocketWrapper<C>>>::Future;

            fn upgrade_inbound(self, socket: C, info: Self::Info) -> Self::Future {
                self.0
                    .upgrade_inbound(SocketWrapper { inner: socket }, info)
            }
        }

        impl<C> OutboundConnectionUpgrade<C> for CodaYamux
        where
            C: AsyncRead + AsyncWrite + Send + Unpin + 'static,
        {
            type Output = <yamux::Config as OutboundConnectionUpgrade<SocketWrapper<C>>>::Output;
            type Error = <yamux::Config as OutboundConnectionUpgrade<C>>::Error;
            type Future = <yamux::Config as OutboundConnectionUpgrade<SocketWrapper<C>>>::Future;

            fn upgrade_outbound(self, socket: C, info: Self::Info) -> Self::Future {
                self.0
                    .upgrade_outbound(SocketWrapper { inner: socket }, info)
            }
        }

        CodaYamux(yamux::Config::default())
    };

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_other_transport(|local_key| {
            let pnet = pnet::PnetConfig::new(pnet::PreSharedKey::new(pnet_key));
            tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
                .and_then(move |socket, _| pnet.handshake(socket))
                .upgrade(upgrade::Version::V1)
                .authenticate(noise::Config::new(local_key).expect("libp2p-noise static keypair"))
                .multiplex(yamux)
                .timeout(std::time::Duration::from_secs(20))
                .boxed()
        })
        .unwrap()
        .with_dns()
        .unwrap()
        .with_behaviour(|_| behaviour)
        .unwrap()
        .build();

    for addr in listen_on {
        swarm.listen_on(addr).unwrap();
    }
    for peer in peers {
        swarm.dial(peer).unwrap();
    }

    swarm
}
