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

use std::{borrow::Cow, fmt, str::FromStr};

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::PeerId;

/// URL path prefix for proxy signaling.
///
/// This newtype wraps a String path prefix (e.g., "/clusters/123") and provides
/// BinProt serialization by encoding as a length-prefixed byte array.
///
/// Used by `Proxied` variant for flexible path-based proxy configurations.
/// The legacy `HttpsProxy(u16, HttpSignalingInfo)` is preserved for BinProt
/// backward compatibility.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, derive_more::Display)]
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
/// Signaling methods can be parsed from string representations or constructed
/// programmatically. They support serialization for storage and network transmission.
///
/// # Example
///
/// ```
/// // Direct HTTPS signaling
/// let method = "/https/signal.example.com/443".parse::<SignalingMethod>()?;
///
/// // P2P relay through an existing peer
/// let method = SignalingMethod::P2p { relay_peer_id: peer_id };
/// ```
#[derive(BinProtWrite, BinProtRead, Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
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
    /// representation following the URL format patterns. The formatted
    /// string can be parsed back using [`FromStr`].
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

/// Errors that can occur when parsing signaling method strings.
///
/// `SignalingMethodParseError` provides detailed error information for
/// parsing failures when converting string representations to [`SignalingMethod`]
/// instances. This helps with debugging configuration and user input validation.
///
/// # Error Types
///
/// The parser can fail for various reasons including missing components,
/// invalid formats, or unsupported method types. Each error variant provides
/// specific context about what went wrong during parsing.
#[derive(Error, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SignalingMethodParseError {
    /// Insufficient arguments provided for the signaling method.
    ///
    /// This occurs when the input string doesn't contain enough components
    /// to construct a valid signaling method. For example, missing host
    /// or port information for HTTP methods.
    #[error("not enough args for the signaling method")]
    NotEnoughArgs,

    /// Unknown or unsupported signaling method type.
    ///
    /// This occurs when the method type (first component) is not recognized.
    /// Supported methods are: `http`, `https`, `https_proxy`, `p2p`.
    #[error("unknown signaling method: `{0}`")]
    UnknownSignalingMethod(String),

    /// Invalid cluster ID for HTTPS proxy methods.
    ///
    /// This occurs when the cluster ID component cannot be parsed as a
    /// valid 16-bit unsigned integer for HTTPS proxy configurations.
    #[error("invalid cluster id")]
    InvalidClusterId,

    /// Failed to parse the host component.
    ///
    /// This occurs when the host string cannot be parsed as a valid
    /// hostname, IP address, or multiaddr format by the Host parser.
    #[error("host parse error: {0}")]
    HostParseError(String),

    /// Failed to parse the port component.
    ///
    /// This occurs when the port string cannot be parsed as a valid
    /// 16-bit unsigned integer port number.
    #[error("port parse error: {0}")]
    PortParseError(String),
}

impl FromStr for SignalingMethod {
    type Err = SignalingMethodParseError;

    /// Parses a string representation into a [`SignalingMethod`].
    ///
    /// This method parses URL-like strings that represent different signaling
    /// transport methods. The parser supports the following formats:
    ///
    /// # Supported Formats
    ///
    /// - **HTTP**: `/http/{host}/{port}`
    /// - **HTTPS**: `/https/{host}/{port}`
    /// - **HTTPS Proxy**: `/https_proxy/{cluster_id}/{host}/{port}`
    /// - **P2P Relay**: `/p2p/{peer_id}`
    ///
    /// # Examples
    ///
    /// ```
    /// use mina::signaling_method::SignalingMethod;
    ///
    /// // HTTP signaling
    /// let method: SignalingMethod = "/http/localhost/8080".parse()?;
    ///
    /// // HTTPS signaling
    /// let method: SignalingMethod = "/https/signal.example.com/443".parse()?;
    ///
    /// // HTTPS proxy with cluster ID
    /// let method: SignalingMethod = "/https_proxy/123/proxy.example.com/443".parse()?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`SignalingMethodParseError`] for various parsing failures:
    /// - Missing components (host, port, etc.)
    /// - Unknown method types
    /// - Invalid numeric values (ports, cluster IDs)
    /// - Invalid host formats
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(SignalingMethodParseError::NotEnoughArgs);
        }

        let method_end_index = s[1..]
            .find('/')
            .map(|i| i + 1)
            .filter(|i| s.len() > *i)
            .ok_or(SignalingMethodParseError::NotEnoughArgs)?;

        let rest = &s[method_end_index..];
        match &s[1..method_end_index] {
            "http" => Ok(Self::Http(rest.parse()?)),
            "https" => Ok(Self::Https(rest.parse()?)),
            "https_proxy" => {
                let mut iter = rest.splitn(3, '/').filter(|v| !v.trim().is_empty());
                let (cluster_id, rest) = (
                    iter.next()
                        .ok_or(SignalingMethodParseError::NotEnoughArgs)?,
                    iter.next()
                        .ok_or(SignalingMethodParseError::NotEnoughArgs)?,
                );
                let cluster_id: u16 = cluster_id
                    .parse()
                    .or(Err(SignalingMethodParseError::InvalidClusterId))?;
                Ok(Self::HttpsProxy(cluster_id, rest.parse()?))
            }
            "proxied" => {
                // Format: /proxied/{scheme}/{encoded_prefix}/{host}/{port}
                let mut iter = rest.splitn(4, '/').filter(|v| !v.trim().is_empty());
                let scheme_str = iter
                    .next()
                    .ok_or(SignalingMethodParseError::NotEnoughArgs)?;
                let scheme = match scheme_str {
                    "http" => ProxyScheme::Http,
                    "https" => ProxyScheme::Https,
                    _ => {
                        return Err(SignalingMethodParseError::UnknownSignalingMethod(format!(
                            "proxied/{}",
                            scheme_str
                        )))
                    }
                };
                let encoded_prefix = iter
                    .next()
                    .ok_or(SignalingMethodParseError::NotEnoughArgs)?;
                let rest = iter
                    .next()
                    .ok_or(SignalingMethodParseError::NotEnoughArgs)?;
                let path_prefix = percent_decode_str(encoded_prefix)
                    .decode_utf8()
                    .map_err(|e| SignalingMethodParseError::HostParseError(e.to_string()))?
                    .into_owned();
                Ok(Self::Proxied(scheme, path_prefix.into(), rest.parse()?))
            }
            method => Err(SignalingMethodParseError::UnknownSignalingMethod(
                method.to_owned(),
            )),
        }
    }
}

impl Serialize for SignalingMethod {
    /// Serializes the signaling method as a string.
    ///
    /// This uses the `Display` implementation to convert the signaling
    /// method to its string representation for serialization.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for SignalingMethod {
    /// Deserializes a signaling method from a string.
    ///
    /// This uses the [`FromStr`] implementation to parse the string
    /// representation back into a [`SignalingMethod`] instance.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error if the string cannot be parsed
    /// as a valid signaling method.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for SignalingMethod parsing
    //!
    //! Run these tests with:
    //! ```bash
    //! cargo test -p p2p signaling_method::tests
    //! ```

    use super::*;
    use crate::webrtc::Host;
    use std::net::Ipv4Addr;

    #[test]
    fn test_from_str_valid_http() {
        let method: SignalingMethod = "/http/example.com/8080".parse().unwrap();
        match method {
            SignalingMethod::Http(info) => {
                assert_eq!(info.host, Host::Domain("example.com".to_string()));
                assert_eq!(info.port, 8080);
            }
            x => panic!("Expected Http variant, got {x:?}"),
        }
    }

    #[test]
    fn test_from_str_valid_https() {
        let method: SignalingMethod = "/https/signal.example.com/443".parse().unwrap();
        match method {
            SignalingMethod::Https(info) => {
                assert_eq!(info.host, Host::Domain("signal.example.com".to_string()));
                assert_eq!(info.port, 443);
            }
            x => panic!("Expected Https variant, got {x:?}"),
        }
    }

    #[test]
    fn test_from_str_valid_https_proxy() {
        let method: SignalingMethod = "/https_proxy/123/proxy.example.com/443".parse().unwrap();
        match method {
            SignalingMethod::HttpsProxy(cluster_id, info) => {
                assert_eq!(cluster_id, 123);
                assert_eq!(info.host, Host::Domain("proxy.example.com".to_string()));
                assert_eq!(info.port, 443);
            }
            x => panic!("Expected HttpsProxy variant, got {x:?}"),
        }
    }

    #[test]
    fn test_from_str_valid_https_proxy_max_cluster_id() {
        let method: SignalingMethod = "/https_proxy/65535/proxy.example.com/443".parse().unwrap();
        match method {
            SignalingMethod::HttpsProxy(cluster_id, info) => {
                assert_eq!(cluster_id, 65535);
                assert_eq!(info.host, Host::Domain("proxy.example.com".to_string()));
                assert_eq!(info.port, 443);
            }
            x => panic!("Expected HttpsProxy variant, got {x:?}"),
        }
    }

    #[test]
    fn test_from_str_valid_http_ipv4() {
        let method: SignalingMethod = "/http/192.168.1.1/8080".parse().unwrap();
        match method {
            SignalingMethod::Http(info) => {
                assert_eq!(info.host, Host::Ipv4(Ipv4Addr::new(192, 168, 1, 1)));
                assert_eq!(info.port, 8080);
            }
            x => panic!("Expected Http variant, got {x:?}"),
        }
    }

    #[test]
    fn test_from_str_valid_https_ipv6() {
        let method: SignalingMethod = "/https/[::1]/443".parse().unwrap();
        match method {
            SignalingMethod::Https(info) => {
                assert!(matches!(info.host, Host::Ipv6(_)));
                assert_eq!(info.port, 443);
            }
            x => panic!("Expected Https variant, got {x:?}"),
        }
    }

    #[test]
    fn test_from_str_empty_string() {
        let result: Result<SignalingMethod, _> = "".parse();
        assert_eq!(result, Err(SignalingMethodParseError::NotEnoughArgs));
    }

    #[test]
    fn test_from_str_no_leading_slash() {
        let result: Result<SignalingMethod, _> = "http/example.com/8080".parse();
        // Without leading slash, it parses "ttp" as the method (s[1..] gives
        // "ttp/example.com/8080")
        assert_eq!(
            result,
            Err(SignalingMethodParseError::UnknownSignalingMethod(
                "ttp".to_string()
            ))
        );
    }

    #[test]
    fn test_from_str_only_slash() {
        let result: Result<SignalingMethod, _> = "/".parse();
        assert_eq!(result, Err(SignalingMethodParseError::NotEnoughArgs));
    }

    #[test]
    fn test_from_str_unknown_method() {
        let result: Result<SignalingMethod, _> = "/websocket/example.com/8080".parse();
        assert_eq!(
            result,
            Err(SignalingMethodParseError::UnknownSignalingMethod(
                "websocket".to_string()
            ))
        );
    }

    #[test]
    fn test_from_str_unknown_method_with_valid_format() {
        let result: Result<SignalingMethod, _> = "/ftp/example.com/21".parse();
        assert_eq!(
            result,
            Err(SignalingMethodParseError::UnknownSignalingMethod(
                "ftp".to_string()
            ))
        );
    }

    #[test]
    fn test_from_str_http_missing_host() {
        let result: Result<SignalingMethod, _> = "/http".parse();
        assert_eq!(result, Err(SignalingMethodParseError::NotEnoughArgs));
    }

    #[test]
    fn test_from_str_http_missing_port() {
        let result: Result<SignalingMethod, _> = "/http/example.com".parse();
        assert_eq!(result, Err(SignalingMethodParseError::NotEnoughArgs));
    }

    #[test]
    fn test_from_str_http_invalid_port() {
        let result: Result<SignalingMethod, _> = "/http/example.com/abc".parse();
        assert!(
            matches!(result, Err(SignalingMethodParseError::PortParseError(_))),
            "expected PortParseError, got {result:?}"
        );
    }

    #[test]
    fn test_from_str_http_port_too_large() {
        let result: Result<SignalingMethod, _> = "/http/example.com/99999".parse();
        assert!(
            matches!(result, Err(SignalingMethodParseError::PortParseError(_))),
            "expected PortParseError, got {result:?}"
        );
    }

    #[test]
    fn test_from_str_https_proxy_missing_cluster_id() {
        let result: Result<SignalingMethod, _> = "/https_proxy".parse();
        assert_eq!(result, Err(SignalingMethodParseError::NotEnoughArgs));
    }

    #[test]
    fn test_from_str_https_proxy_missing_host() {
        let result: Result<SignalingMethod, _> = "/https_proxy/123".parse();
        assert_eq!(result, Err(SignalingMethodParseError::NotEnoughArgs));
    }

    #[test]
    fn test_from_str_https_proxy_invalid_cluster_id() {
        let result: Result<SignalingMethod, _> = "/https_proxy/abc/proxy.example.com/443".parse();
        assert_eq!(result, Err(SignalingMethodParseError::InvalidClusterId));
    }

    #[test]
    fn test_from_str_https_proxy_cluster_id_too_large() {
        let result: Result<SignalingMethod, _> = "/https_proxy/99999/proxy.example.com/443".parse();
        assert_eq!(result, Err(SignalingMethodParseError::InvalidClusterId));
    }

    #[test]
    fn test_from_str_https_proxy_negative_cluster_id() {
        let result: Result<SignalingMethod, _> = "/https_proxy/-1/proxy.example.com/443".parse();
        assert_eq!(result, Err(SignalingMethodParseError::InvalidClusterId));
    }

    #[test]
    fn test_from_str_invalid_host() {
        // This will depend on Host's parsing behavior - assuming it rejects
        // certain formats
        let result: Result<SignalingMethod, _> = "/http//8080".parse();
        // Should be either NotEnoughArgs or HostParseError depending on
        // implementation
        assert!(
            matches!(
                result,
                Err(SignalingMethodParseError::NotEnoughArgs)
                    | Err(SignalingMethodParseError::HostParseError(_))
            ),
            "expected NotEnoughArgs or HostParseError, got {result:?}"
        );
    }

    #[test]
    fn test_from_str_extra_slashes() {
        let result: Result<SignalingMethod, _> = "//http//example.com//8080//".parse();
        // The double leading slashes mean s[1..] gives "/http//...", split
        // produces empty first component
        assert_eq!(
            result,
            Err(SignalingMethodParseError::UnknownSignalingMethod(
                "".to_string()
            ))
        );
    }

    #[test]
    fn test_roundtrip_http() {
        let original = SignalingMethod::Http(HttpSignalingInfo {
            host: Host::Domain("example.com".to_string()),
            port: 8080,
        });

        let serialized = original.to_string();
        let deserialized: SignalingMethod = serialized.parse().unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_roundtrip_https() {
        let original = SignalingMethod::Https(HttpSignalingInfo {
            host: Host::Ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 443,
        });

        let serialized = original.to_string();
        let deserialized: SignalingMethod = serialized.parse().unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_roundtrip_https_proxy() {
        let original = SignalingMethod::HttpsProxy(
            123,
            HttpSignalingInfo {
                host: Host::Domain("proxy.example.com".to_string()),
                port: 443,
            },
        );

        let serialized = original.to_string();
        let deserialized: SignalingMethod = serialized.parse().unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_case_sensitivity() {
        let result: Result<SignalingMethod, _> = "/HTTP/example.com/8080".parse();
        assert_eq!(
            result,
            Err(SignalingMethodParseError::UnknownSignalingMethod(
                "HTTP".to_string()
            ))
        );

        let result: Result<SignalingMethod, _> = "/Http/example.com/8080".parse();
        assert_eq!(
            result,
            Err(SignalingMethodParseError::UnknownSignalingMethod(
                "Http".to_string()
            ))
        );
    }

    #[test]
    fn test_whitespace_handling() {
        // The parser should filter empty components from split
        let result: Result<SignalingMethod, _> = "/http/ /8080".parse();
        assert_eq!(result, Err(SignalingMethodParseError::NotEnoughArgs));
    }

    #[test]
    fn test_https_proxy_zero_cluster_id() {
        let method: SignalingMethod = "/https_proxy/0/proxy.example.com/443".parse().unwrap();
        match method {
            SignalingMethod::HttpsProxy(cluster_id, info) => {
                assert_eq!(cluster_id, 0);
                assert_eq!(info.host, Host::Domain("proxy.example.com".to_string()));
                assert_eq!(info.port, 443);
            }
            x => panic!("Expected HttpsProxy variant, got {x:?}"),
        }
    }

    #[test]
    fn test_standard_ports() {
        let method: SignalingMethod = "/http/localhost/80".parse().unwrap();
        match method {
            SignalingMethod::Http(info) => {
                assert_eq!(info.port, 80);
            }
            x => panic!("Expected Http variant, got {x:?}"),
        }

        let method: SignalingMethod = "/https/localhost/443".parse().unwrap();
        match method {
            SignalingMethod::Https(info) => {
                assert_eq!(info.port, 443);
            }
            x => panic!("Expected Https variant, got {x:?}"),
        }
    }

    #[test]
    fn test_https_proxy_with_ipv4() {
        let method: SignalingMethod = "/https_proxy/456/192.168.1.1/8443".parse().unwrap();
        match method {
            SignalingMethod::HttpsProxy(cluster_id, info) => {
                assert_eq!(cluster_id, 456);
                assert_eq!(info.host, Host::Ipv4(Ipv4Addr::new(192, 168, 1, 1)));
                assert_eq!(info.port, 8443);
            }
            x => panic!("Expected HttpsProxy variant, got {x:?}"),
        }
    }

    // Proxied tests

    #[test]
    fn test_from_str_valid_proxied() {
        // URL-encoded path prefix: /clusters/123 -> %2Fclusters%2F123
        let method: SignalingMethod = "/proxied/https/%2Fclusters%2F123/proxy.example.com/443"
            .parse()
            .unwrap();
        match method {
            SignalingMethod::Proxied(ProxyScheme::Https, prefix, info) => {
                assert_eq!(prefix.as_ref(), "/clusters/123");
                assert_eq!(info.host, Host::Domain("proxy.example.com".to_string()));
                assert_eq!(info.port, 443);
            }
            x => panic!("Expected Proxied variant, got {x:?}"),
        }
    }

    #[test]
    fn test_from_str_proxied_complex_path() {
        // URL-encoded path: /api/v2/webrtc -> %2Fapi%2Fv2%2Fwebrtc
        let method: SignalingMethod =
            "/proxied/https/%2Fapi%2Fv2%2Fwebrtc/gateway.example.com/8443"
                .parse()
                .unwrap();
        match method {
            SignalingMethod::Proxied(ProxyScheme::Https, prefix, info) => {
                assert_eq!(prefix.as_ref(), "/api/v2/webrtc");
                assert_eq!(info.host, Host::Domain("gateway.example.com".to_string()));
                assert_eq!(info.port, 8443);
            }
            x => panic!("Expected Proxied variant, got {x:?}"),
        }
    }

    #[test]
    fn test_roundtrip_proxied() {
        let original = SignalingMethod::Proxied(
            ProxyScheme::Https,
            "/clusters/789".into(),
            HttpSignalingInfo {
                host: Host::Domain("proxy.example.com".to_string()),
                port: 443,
            },
        );

        let serialized = original.to_string();
        assert_eq!(
            serialized,
            "/proxied/https/%2Fclusters%2F789/proxy.example.com/443"
        );

        let deserialized: SignalingMethod = serialized.parse().unwrap();
        assert_eq!(original, deserialized);
    }

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

        let url = method.http_url().unwrap();
        assert_eq!(
            url,
            "https://gateway.example.com:443/custom/path/mina/webrtc/signal"
        );
    }

    #[test]
    fn test_proxied_https_url_no_leading_slash() {
        let method = SignalingMethod::Proxied(
            ProxyScheme::Https,
            "custom/path".into(),
            HttpSignalingInfo {
                host: Host::Domain("gateway.example.com".to_string()),
                port: 443,
            },
        );

        let url = method.http_url().unwrap();
        // Should add leading slash
        assert_eq!(
            url,
            "https://gateway.example.com:443/custom/path/mina/webrtc/signal"
        );
    }

    #[test]
    fn test_proxied_https_url_trailing_slash() {
        let method = SignalingMethod::Proxied(
            ProxyScheme::Https,
            "/custom/path/".into(),
            HttpSignalingInfo {
                host: Host::Domain("gateway.example.com".to_string()),
                port: 443,
            },
        );

        let url = method.http_url().unwrap();
        // Should not double the trailing slash
        assert_eq!(
            url,
            "https://gateway.example.com:443/custom/path/mina/webrtc/signal"
        );
    }

    #[test]
    fn test_proxied_with_ipv4() {
        let method: SignalingMethod = "/proxied/https/%2Ftest/192.168.1.1/8443".parse().unwrap();
        match method {
            SignalingMethod::Proxied(ProxyScheme::Https, prefix, info) => {
                assert_eq!(prefix.as_ref(), "/test");
                assert_eq!(info.host, Host::Ipv4(Ipv4Addr::new(192, 168, 1, 1)));
                assert_eq!(info.port, 8443);
            }
            _ => panic!("Expected Proxied variant"),
        }
    }

    #[test]
    fn test_proxied_missing_prefix() {
        let result: Result<SignalingMethod, _> = "/proxied".parse();
        assert_eq!(result, Err(SignalingMethodParseError::NotEnoughArgs));
    }

    #[test]
    fn test_proxied_missing_host() {
        let result: Result<SignalingMethod, _> = "/proxied/https/%2Fprefix".parse();
        assert_eq!(result, Err(SignalingMethodParseError::NotEnoughArgs));
    }

    // HttpsProxy vs Proxied equivalency tests

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

        let url = method.http_url().unwrap();
        // Empty prefix should still result in valid URL with single slash
        assert_eq!(url, "https://gateway.example.com:443/mina/webrtc/signal");
    }

    #[test]
    fn test_proxied_just_slash() {
        let method = SignalingMethod::Proxied(
            ProxyScheme::Https,
            "/".into(),
            HttpSignalingInfo {
                host: Host::Domain("gateway.example.com".to_string()),
                port: 443,
            },
        );

        let url = method.http_url().unwrap();
        // Just "/" should work correctly
        assert_eq!(url, "https://gateway.example.com:443/mina/webrtc/signal");
    }

    #[test]
    fn test_proxied_slash_variations() {
        let info = HttpSignalingInfo {
            host: Host::Domain("example.com".to_string()),
            port: 443,
        };

        // No slashes
        let m1 = SignalingMethod::Proxied(ProxyScheme::Https, "path".into(), info.clone());
        assert_eq!(
            m1.http_url().unwrap(),
            "https://example.com:443/path/mina/webrtc/signal"
        );

        // Leading slash only
        let m2 = SignalingMethod::Proxied(ProxyScheme::Https, "/path".into(), info.clone());
        assert_eq!(
            m2.http_url().unwrap(),
            "https://example.com:443/path/mina/webrtc/signal"
        );

        // Trailing slash only
        let m3 = SignalingMethod::Proxied(ProxyScheme::Https, "path/".into(), info.clone());
        assert_eq!(
            m3.http_url().unwrap(),
            "https://example.com:443/path/mina/webrtc/signal"
        );

        // Both slashes
        let m4 = SignalingMethod::Proxied(ProxyScheme::Https, "/path/".into(), info.clone());
        assert_eq!(
            m4.http_url().unwrap(),
            "https://example.com:443/path/mina/webrtc/signal"
        );
    }

    #[test]
    fn test_proxied_multi_segment_path_slash_variations() {
        let info = HttpSignalingInfo {
            host: Host::Domain("example.com".to_string()),
            port: 443,
        };

        // No outer slashes
        let m1 = SignalingMethod::Proxied(
            ProxyScheme::Https,
            "api/v2/clusters/123".into(),
            info.clone(),
        );
        assert_eq!(
            m1.http_url().unwrap(),
            "https://example.com:443/api/v2/clusters/123/mina/webrtc/signal"
        );

        // Both outer slashes
        let m2 = SignalingMethod::Proxied(
            ProxyScheme::Https,
            "/api/v2/clusters/123/".into(),
            info.clone(),
        );
        assert_eq!(
            m2.http_url().unwrap(),
            "https://example.com:443/api/v2/clusters/123/mina/webrtc/signal"
        );
    }

    #[test]
    fn test_proxied_roundtrip_just_slash() {
        // %2F is URL-encoded "/"
        let method: SignalingMethod = "/proxied/https/%2F/example.com/443".parse().unwrap();
        match &method {
            SignalingMethod::Proxied(ProxyScheme::Https, prefix, info) => {
                assert_eq!(prefix.as_ref(), "/");
                assert_eq!(info.host, Host::Domain("example.com".to_string()));
                assert_eq!(info.port, 443);
            }
            x => panic!("Expected Proxied variant, got {x:?}"),
        }

        // Roundtrip
        let serialized = method.to_string();
        let deserialized: SignalingMethod = serialized.parse().unwrap();
        assert_eq!(method, deserialized);
    }

    #[test]
    fn test_proxied_roundtrip_empty_prefix() {
        // Empty string prefix can't roundtrip because the parser filters empty components.
        // This is acceptable - empty prefix is treated as "no prefix" and produces
        // the same URL as Https variant. Test verifies the expected parse error.
        let original = SignalingMethod::Proxied(
            ProxyScheme::Https,
            "".into(),
            HttpSignalingInfo {
                host: Host::Domain("example.com".to_string()),
                port: 443,
            },
        );

        let serialized = original.to_string();
        // Format is /proxied//example.com/443 - empty prefix component
        // Parser sees: ["proxied", "example.com", "443"] after filtering empties
        // This means it tries to parse "example.com" as the prefix, "443" as host
        let result: Result<SignalingMethod, _> = serialized.parse();
        assert!(
            result.is_err(),
            "Empty prefix can't roundtrip - use just '/' prefix instead"
        );
    }

    // HTTP proxy tests (new functionality)

    #[test]
    fn test_proxied_http_scheme() {
        let method = SignalingMethod::Proxied(
            ProxyScheme::Http,
            "/api/proxy".into(),
            HttpSignalingInfo {
                host: Host::Domain("gateway.example.com".to_string()),
                port: 8080,
            },
        );

        let url = method.http_url().unwrap();
        assert_eq!(
            url,
            "http://gateway.example.com:8080/api/proxy/mina/webrtc/signal"
        );
    }

    #[test]
    fn test_proxied_http_scheme_roundtrip() {
        let original = SignalingMethod::Proxied(
            ProxyScheme::Http,
            "/dev/proxy".into(),
            HttpSignalingInfo {
                host: Host::Domain("localhost".to_string()),
                port: 3000,
            },
        );

        let serialized = original.to_string();
        assert!(serialized.contains("/proxied/http/"));

        let deserialized: SignalingMethod = serialized.parse().unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_from_str_proxied_http() {
        let method: SignalingMethod = "/proxied/http/%2Fdev%2Fproxy/localhost/3000"
            .parse()
            .unwrap();
        match method {
            SignalingMethod::Proxied(ProxyScheme::Http, prefix, info) => {
                assert_eq!(prefix.as_ref(), "/dev/proxy");
                assert_eq!(info.host, Host::Domain("localhost".to_string()));
                assert_eq!(info.port, 3000);
            }
            x => panic!("Expected Proxied variant with Http scheme, got {x:?}"),
        }
    }

    #[test]
    fn test_proxied_invalid_scheme() {
        let result: Result<SignalingMethod, _> = "/proxied/ftp/%2Fpath/example.com/21".parse();
        assert_eq!(
            result,
            Err(SignalingMethodParseError::UnknownSignalingMethod(
                "proxied/ftp".to_string()
            ))
        );
    }
}
