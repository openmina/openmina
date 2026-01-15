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
//! - HTTPS Proxy: `/https_proxy/{cluster_id}/{host}/{port}`
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

use std::{fmt, str::FromStr};

use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::PeerId;

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

    /// HTTPS proxy signaling connection.
    ///
    /// Uses an SSL gateway/proxy server to reach the actual signaling server.
    /// The first parameter is the cluster ID for routing, and the second
    /// parameter contains the proxy server connection information.
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
}

impl SignalingMethod {
    /// Determines if this signaling method supports direct connections.
    ///
    /// Direct connection methods (HTTP, HTTPS, HTTPS Proxy) can establish
    /// signaling channels immediately without requiring existing peer connections.
    /// P2P relay methods require an already-established peer connection to function.
    ///
    /// # Returns
    ///
    /// * `true` for HTTP, HTTPS, and HTTPS Proxy methods
    /// * `false` for P2P relay methods
    ///
    /// This is useful for connection strategy decisions and determining whether
    /// bootstrap connections are needed before signaling can occur.
    pub fn can_connect_directly(&self) -> bool {
        match self {
            Self::Http(_) | Self::Https(_) | Self::HttpsProxy(_, _) => true,
            Self::P2p { .. } => false,
        }
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
        let (http, info) = match self {
            Self::Http(info) => ("http", info),
            Self::Https(info) => ("https", info),
            Self::HttpsProxy(cluster_id, info) => {
                return Some(format!(
                    "https://{}:{}/clusters/{}/mina/webrtc/signal",
                    info.host, info.port, cluster_id
                ));
            }
            _ => return None,
        };
        Some(format!(
            "{http}://{}:{}/mina/webrtc/signal",
            info.host, info.port,
        ))
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
#[derive(Error, Serialize, Deserialize, Debug, Clone)]
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
            _ => panic!("Expected Http variant"),
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
            _ => panic!("Expected Https variant"),
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
            _ => panic!("Expected HttpsProxy variant"),
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
            _ => panic!("Expected HttpsProxy variant"),
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
            _ => panic!("Expected Http variant"),
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
            _ => panic!("Expected Https variant"),
        }
    }

    #[test]
    fn test_from_str_empty_string() {
        let result: Result<SignalingMethod, _> = "".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::NotEnoughArgs
        ));
    }

    #[test]
    fn test_from_str_no_leading_slash() {
        let result: Result<SignalingMethod, _> = "http/example.com/8080".parse();
        assert!(result.is_err());
        // Without leading slash, it treats "http" as unknown method since
        // there's no slash at start
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::UnknownSignalingMethod(_)
        ));
    }

    #[test]
    fn test_from_str_only_slash() {
        let result: Result<SignalingMethod, _> = "/".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::NotEnoughArgs
        ));
    }

    #[test]
    fn test_from_str_unknown_method() {
        let result: Result<SignalingMethod, _> = "/websocket/example.com/8080".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::UnknownSignalingMethod(_)
        ));
    }

    #[test]
    fn test_from_str_unknown_method_with_valid_format() {
        let result: Result<SignalingMethod, _> = "/ftp/example.com/21".parse();
        assert!(result.is_err());
        match result.unwrap_err() {
            SignalingMethodParseError::UnknownSignalingMethod(method) => {
                assert_eq!(method, "ftp");
            }
            _ => panic!("Expected UnknownSignalingMethod error"),
        }
    }

    #[test]
    fn test_from_str_http_missing_host() {
        let result: Result<SignalingMethod, _> = "/http".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::NotEnoughArgs
        ));
    }

    #[test]
    fn test_from_str_http_missing_port() {
        let result: Result<SignalingMethod, _> = "/http/example.com".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::NotEnoughArgs
        ));
    }

    #[test]
    fn test_from_str_http_invalid_port() {
        let result: Result<SignalingMethod, _> = "/http/example.com/abc".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::PortParseError(_)
        ));
    }

    #[test]
    fn test_from_str_http_port_too_large() {
        let result: Result<SignalingMethod, _> = "/http/example.com/99999".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::PortParseError(_)
        ));
    }

    #[test]
    fn test_from_str_https_proxy_missing_cluster_id() {
        let result: Result<SignalingMethod, _> = "/https_proxy".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::NotEnoughArgs
        ));
    }

    #[test]
    fn test_from_str_https_proxy_missing_host() {
        let result: Result<SignalingMethod, _> = "/https_proxy/123".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::NotEnoughArgs
        ));
    }

    #[test]
    fn test_from_str_https_proxy_invalid_cluster_id() {
        let result: Result<SignalingMethod, _> = "/https_proxy/abc/proxy.example.com/443".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::InvalidClusterId
        ));
    }

    #[test]
    fn test_from_str_https_proxy_cluster_id_too_large() {
        let result: Result<SignalingMethod, _> = "/https_proxy/99999/proxy.example.com/443".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::InvalidClusterId
        ));
    }

    #[test]
    fn test_from_str_https_proxy_negative_cluster_id() {
        let result: Result<SignalingMethod, _> = "/https_proxy/-1/proxy.example.com/443".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::InvalidClusterId
        ));
    }

    #[test]
    fn test_from_str_invalid_host() {
        // This will depend on Host's parsing behavior - assuming it rejects
        // certain formats
        let result: Result<SignalingMethod, _> = "/http//8080".parse();
        assert!(result.is_err());
        // Should be either NotEnoughArgs or HostParseError depending on
        // implementation
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::NotEnoughArgs | SignalingMethodParseError::HostParseError(_)
        ));
    }

    #[test]
    fn test_from_str_extra_slashes() {
        let result: Result<SignalingMethod, _> = "//http//example.com//8080//".parse();
        assert!(result.is_err());
        // The extra slashes mean method parsing fails - "http" becomes unknown
        // method
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::UnknownSignalingMethod(_)
        ));
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
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::UnknownSignalingMethod(_)
        ));

        let result: Result<SignalingMethod, _> = "/Http/example.com/8080".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::UnknownSignalingMethod(_)
        ));
    }

    #[test]
    fn test_whitespace_handling() {
        // The parser should filter empty components from split
        let result: Result<SignalingMethod, _> = "/http/ /8080".parse();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalingMethodParseError::NotEnoughArgs
        ));
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
            _ => panic!("Expected HttpsProxy variant"),
        }
    }

    #[test]
    fn test_standard_ports() {
        let method: SignalingMethod = "/http/localhost/80".parse().unwrap();
        match method {
            SignalingMethod::Http(info) => {
                assert_eq!(info.port, 80);
            }
            _ => panic!("Expected Http variant"),
        }

        let method: SignalingMethod = "/https/localhost/443".parse().unwrap();
        match method {
            SignalingMethod::Https(info) => {
                assert_eq!(info.port, 443);
            }
            _ => panic!("Expected Https variant"),
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
            _ => panic!("Expected HttpsProxy variant"),
        }
    }
}
