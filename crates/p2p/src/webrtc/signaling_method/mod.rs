//! WebRTC Signaling Transport Methods
//!
//! This module defines the different transport methods available for WebRTC signaling
//! in Mina Rust's peer-to-peer network. WebRTC requires an external signaling mechanism
//! to exchange connection metadata before establishing direct peer-to-peer connections.
//!
//! ## Signaling Transport Methods
//!
//! The Mina Rust node supports multiple signaling transport methods to accommodate different
//! network environments and security requirements:
//!
//! ### HTTP/HTTPS Direct Connections
//!
//! - **HTTP**: Direct HTTP connections to signaling servers (typically for local/testing)
//! - **HTTPS**: Secure HTTPS connections to signaling servers (recommended for production)
//!
//! These methods allow peers to directly contact signaling servers to exchange offers
//! and answers for WebRTC connection establishment.
//!
//! ### HTTPS Proxy
//!
//! - **HTTPS Proxy**: Uses an SSL gateway/proxy server to reach the actual signaling server
//!
//! ### P2P Relay Signaling
//!
//! - **P2P Relay**: Uses existing peer connections to relay signaling messages
//! - Enables signaling through already-established peer connections
//! - Provides redundancy when direct signaling server access is unavailable
//! - Supports bootstrapping new connections through existing network peers
//!
//! ## URL Format
//!
//! Signaling methods use a structured URL format:
//!
//! - HTTP: `/http/{host}/{port}`
//! - HTTPS: `/https/{host}/{port}`
//! - HTTPS Proxy (legacy): `/https_proxy/{cluster_id}/{host}/{port}`
//! - Proxied: `/proxied/{http|https}/{encoded_prefix}/{host}/{port}`
//! - P2P Relay: `/p2p/{peer_id}`
//!
//! ## Connection Strategy
//!
//! The signaling method determines how peers discover and connect to each other:
//!
//! 1. **Direct Methods** (HTTP/HTTPS) - Can connect immediately to signaling servers
//! 2. **Proxy Methods** - Route through intermediate proxy infrastructure
//! 3. **Relay Methods** - Require existing peer connections for message routing

mod http;
pub use http::HttpSignalingInfo;

use std::{borrow::Cow, fmt};

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};

use crate::PeerId;

/// URL path prefix for proxy signaling.
///
/// This newtype wraps a String path prefix (e.g., "/clusters/123") and provides
/// BinProt serialization by encoding as a length-prefixed byte array.
///
/// Used by `Proxied` variant for flexible path-based proxy configurations.
/// The legacy `HttpsProxy(u16, HttpSignalingInfo)` is preserved for BinProt
/// backward compatibility.
#[derive(
    Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize, derive_more::Display,
)]
pub struct PathPrefix(String);

impl PathPrefix {
    /// Consumes the PathPrefix and returns the inner String.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for PathPrefix {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for PathPrefix {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<Cow<'_, str>> for PathPrefix {
    fn from(s: Cow<'_, str>) -> Self {
        Self(s.into_owned())
    }
}

impl<'a> From<&'a PathPrefix> for Cow<'a, str> {
    fn from(p: &'a PathPrefix) -> Self {
        Cow::Borrowed(p.as_ref())
    }
}

impl AsRef<str> for PathPrefix {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Proxy connection scheme (HTTP or HTTPS).
///
/// Determines whether the proxy connection uses plain HTTP or secure HTTPS.
/// HTTPS is recommended for production environments.
#[derive(
    BinProtWrite,
    BinProtRead,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    derive_more::Display,
)]
pub enum ProxyScheme {
    /// Plain HTTP proxy connection.
    #[display(fmt = "http")]
    Http,
    /// Secure HTTPS proxy connection.
    #[display(fmt = "https")]
    Https,
}

impl BinProtRead for PathPrefix {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let bytes: Vec<u8> = BinProtRead::binprot_read(r)?;
        let s = String::from_utf8(bytes).map_err(|e| binprot::Error::from(e.utf8_error()))?;
        Ok(s.into())
    }
}

impl BinProtWrite for PathPrefix {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.as_ref().as_bytes().to_vec().binprot_write(w)
    }
}

/// WebRTC signaling transport method configuration.
///
/// `SignalingMethod` defines how WebRTC signaling messages (offers and answers)
/// are transported between peers. Different methods provide flexibility for
/// various network environments and infrastructure requirements.
///
/// # Method Types
///
/// - **HTTP/HTTPS**: Direct connections to signaling servers
/// - **HTTPS Proxy**: Connections through SSL gateway/proxy servers
/// - **P2P Relay**: Signaling through existing peer connections
///
/// Each method encapsulates the necessary connection information to establish
/// the signaling channel, which is used before the actual WebRTC peer-to-peer
/// connection is established.
///
/// # Usage
///
/// Signaling methods are constructed programmatically (typically from multiaddr
/// parsing) and support serialization for network transmission.
#[derive(
    BinProtWrite, BinProtRead, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize,
)]
pub enum SignalingMethod {
    /// HTTP signaling server connection.
    ///
    /// Uses plain HTTP for signaling message exchange. Typically used for
    /// local development or testing environments where encryption is not required.
    Http(HttpSignalingInfo),

    /// HTTPS signaling server connection.
    ///
    /// Uses secure HTTPS for signaling message exchange. Recommended for
    /// production environments to protect signaling data in transit.
    Https(HttpSignalingInfo),

    /// HTTPS proxy signaling connection (legacy format).
    ///
    /// Uses an SSL gateway/proxy server to reach the actual signaling server.
    /// The first parameter is the cluster ID for routing, and the second
    /// parameter contains the proxy server connection information.
    ///
    /// Kept for BinProt backward compatibility. Prefer `Proxied` for new code.
    HttpsProxy(u16, HttpSignalingInfo),

    /// P2P relay signaling through an existing peer connection.
    ///
    /// Uses an already-established peer connection to relay signaling messages
    /// to other peers. This enables signaling when direct access to signaling
    /// servers is unavailable and provides redundancy in the signaling process.
    P2p {
        /// The peer ID of the relay peer that will forward signaling messages.
        relay_peer_id: PeerId,
    },

    /// Proxy signaling connection (extended format).
    ///
    /// Uses a gateway/proxy server to reach the actual signaling server.
    /// Supports both HTTP and HTTPS proxy connections via the `ProxyScheme` field.
    ///
    /// Fields:
    /// - `ProxyScheme`: Whether to use HTTP or HTTPS for the proxy connection
    /// - `PathPrefix`: The URL path prefix (e.g., "/clusters/123")
    /// - `HttpSignalingInfo`: The proxy server connection information
    Proxied(ProxyScheme, PathPrefix, HttpSignalingInfo),
}

impl SignalingMethod {
    /// Determines if this signaling method supports direct connections.
    ///
    /// Direct connection methods (HTTP, HTTPS, HTTPS Proxy, Proxied) can establish
    /// signaling channels immediately without requiring existing peer connections.
    /// P2P relay methods require an already-established peer connection to function.
    ///
    /// # Returns
    ///
    /// * `true` for HTTP, HTTPS, HTTPS Proxy, and Proxied methods
    /// * `false` for P2P relay methods
    ///
    /// This is useful for connection strategy decisions and determining whether
    /// bootstrap connections are needed before signaling can occur.
    pub fn can_connect_directly(&self) -> bool {
        !matches!(self, Self::P2p { .. })
    }

    /// Constructs the HTTP(S) URL for sending WebRTC offers.
    ///
    /// This method generates the appropriate URL endpoint for sending WebRTC
    /// signaling messages based on the signaling method configuration.
    ///
    /// # URL Formats
    ///
    /// - **HTTP**: `http://{host}:{port}/mina/webrtc/signal`
    /// - **HTTPS**: `https://{host}:{port}/mina/webrtc/signal`
    /// - **HTTPS Proxy**: `https://{host}:{port}/clusters/{cluster_id}/mina/webrtc/signal`
    /// - **Proxied**: `{http|https}://{host}:{port}{prefix}/mina/webrtc/signal`
    ///
    /// # Returns
    ///
    /// * `Some(String)` containing the signaling URL for HTTP-based methods
    /// * `None` for P2P relay methods that don't use HTTP endpoints
    ///
    /// # Example
    ///
    /// ```
    /// let method = SignalingMethod::Https(info);
    /// let url = method.http_url(); // Some("https://signal.example.com:443/mina/webrtc/signal")
    /// ```
    pub fn http_url(&self) -> Option<String> {
        let slash = Cow::Borrowed("/");
        let (http, prefix, HttpSignalingInfo { host, port }) = match self {
            Self::Http(info) => ("http", slash, info),
            Self::Https(info) => ("https", slash, info),
            Self::HttpsProxy(cluster_id, info) => (
                "https",
                Cow::Owned(format!("/clusters/{cluster_id}/")),
                info,
            ),
            Self::Proxied(scheme, prefix, info) => {
                // Handle empty prefix or just "/" as equivalent to no prefix
                let prefix_str = prefix.as_ref();
                let prefix_cow = if prefix_str.is_empty() || prefix_str == "/" {
                    slash
                } else {
                    let needs_start_slash = !prefix_str.starts_with('/');
                    let needs_end_slash = !prefix_str.ends_with('/');
                    Cow::Owned(format!(
                        "{}{}{}",
                        if needs_start_slash { "/" } else { "" },
                        prefix_str,
                        if needs_end_slash { "/" } else { "" }
                    ))
                };
                return Some(format!(
                    "{scheme}://{host}:{port}{prefix_cow}mina/webrtc/signal",
                    host = info.host,
                    port = info.port
                ));
            }
            _ => return None,
        };
        Some(format!("{http}://{host}:{port}{prefix}mina/webrtc/signal",))
    }

    /// Extracts the relay peer ID for P2P signaling methods.
    ///
    /// For P2P relay signaling methods, this returns the peer ID of the
    /// intermediate peer that will forward signaling messages. This is used
    /// to identify which existing peer connection should be used for relaying.
    ///
    /// # Returns
    ///
    /// * `Some(PeerId)` for P2P relay methods
    /// * `None` for direct connection methods (HTTP/HTTPS)
    ///
    /// # Usage
    ///
    /// This method is typically used when setting up message routing for
    /// P2P relay signaling to determine which peer connection should handle
    /// the signaling traffic.
    pub fn p2p_relay_peer_id(&self) -> Option<PeerId> {
        match self {
            Self::P2p { relay_peer_id } => Some(*relay_peer_id),
            _ => None,
        }
    }
}

impl fmt::Display for SignalingMethod {
    /// Formats the signaling method as a URL path string.
    ///
    /// This implementation converts the signaling method into its string
    /// representation following the URL format patterns.
    ///
    /// # Format Patterns
    ///
    /// - HTTP: `/http/{host}/{port}`
    /// - HTTPS: `/https/{host}/{port}`
    /// - HTTPS Proxy: `/https_proxy/{cluster_id}/{host}/{port}`
    /// - P2P Relay: `/p2p/{peer_id}`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(signaling) => {
                write!(f, "/http")?;
                signaling.fmt(f)
            }
            Self::Https(signaling) => {
                write!(f, "/https")?;
                signaling.fmt(f)
            }
            Self::HttpsProxy(cluster_id, signaling) => {
                write!(f, "/https_proxy/{cluster_id}")?;
                signaling.fmt(f)
            }
            Self::Proxied(scheme, path_prefix, signaling) => {
                let encoded = utf8_percent_encode(path_prefix.as_ref(), NON_ALPHANUMERIC);
                write!(f, "/proxied/{scheme}/{encoded}")?;
                signaling.fmt(f)
            }
            Self::P2p { relay_peer_id } => {
                write!(f, "/p2p/{relay_peer_id}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webrtc::Host;

    #[test]
    fn test_proxied_https_url() {
        let method = SignalingMethod::Proxied(
            ProxyScheme::Https,
            "/custom/path".into(),
            HttpSignalingInfo {
                host: Host::Domain("gateway.example.com".to_string()),
                port: 443,
            },
        );

        assert_eq!(
            method.http_url().unwrap(),
            "https://gateway.example.com:443/custom/path/mina/webrtc/signal"
        );
    }

    #[test]
    fn test_https_proxy_and_proxied_equivalent_url() {
        let info = HttpSignalingInfo {
            host: Host::Domain("gateway.example.com".to_string()),
            port: 443,
        };

        let legacy = SignalingMethod::HttpsProxy(123, info.clone());
        let proxied = SignalingMethod::Proxied(ProxyScheme::Https, "/clusters/123".into(), info);

        assert_eq!(legacy.http_url(), proxied.http_url());
        assert_eq!(
            legacy.http_url().unwrap(),
            "https://gateway.example.com:443/clusters/123/mina/webrtc/signal"
        );
    }

    #[test]
    fn test_proxied_empty_prefix() {
        let method = SignalingMethod::Proxied(
            ProxyScheme::Https,
            "".into(),
            HttpSignalingInfo {
                host: Host::Domain("gateway.example.com".to_string()),
                port: 443,
            },
        );

        assert_eq!(
            method.http_url().unwrap(),
            "https://gateway.example.com:443/mina/webrtc/signal"
        );
    }

    #[test]
    fn test_proxied_slash_variations() {
        let info = HttpSignalingInfo {
            host: Host::Domain("example.com".to_string()),
            port: 443,
        };

        let cases = vec![
            ("path", "https://example.com:443/path/mina/webrtc/signal"),
            ("/path", "https://example.com:443/path/mina/webrtc/signal"),
            ("path/", "https://example.com:443/path/mina/webrtc/signal"),
            ("/path/", "https://example.com:443/path/mina/webrtc/signal"),
        ];

        for (prefix, expected) in cases {
            let m = SignalingMethod::Proxied(ProxyScheme::Https, prefix.into(), info.clone());
            assert_eq!(m.http_url().unwrap(), expected, "prefix: {prefix:?}");
        }
    }

    #[test]
    fn test_proxied_http_scheme_url() {
        let method = SignalingMethod::Proxied(
            ProxyScheme::Http,
            "/api/proxy".into(),
            HttpSignalingInfo {
                host: Host::Domain("gateway.example.com".to_string()),
                port: 8080,
            },
        );

        assert_eq!(
            method.http_url().unwrap(),
            "http://gateway.example.com:8080/api/proxy/mina/webrtc/signal"
        );
    }

    #[test]
    fn test_p2p_has_no_http_url() {
        let method = SignalingMethod::P2p {
            relay_peer_id: crate::identity::SecretKey::rand().public_key().peer_id(),
        };
        assert!(method.http_url().is_none());
    }
}
