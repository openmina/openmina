//! Host address resolution for WebRTC connections.
//!
//! This module provides the [`Host`] enum for representing different types of
//! network addresses used in WebRTC signaling. It supports various address
//! formats including domain names, IPv4/IPv6 addresses, and multiaddr protocol
//! addresses.
//!
//! ## Supported Address Types
//!
//! - **Domain Names**: DNS resolvable hostnames (e.g., `signal.example.com`)
//! - **IPv4 Addresses**: Standard IPv4 addresses (e.g., `192.168.1.1`)
//! - **IPv6 Addresses**: Standard IPv6 addresses (e.g., `::1`)
//! - **Multiaddr**: Protocol-aware addressing format for P2P networks
//!
//! ## Usage
//!
//! The `Host` type is used throughout the WebRTC implementation to specify
//! signaling server addresses and peer endpoints. It provides automatic
//! parsing and resolution capabilities for different address formats.

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, derive_more::From,
)]
pub enum Host {
    /// A DNS domain name, as '.' dot-separated labels.
    /// Non-ASCII labels are encoded in punycode per IDNA if this is the host of
    /// a special URL, or percent encoded for non-special URLs. Hosts for
    /// non-special URLs are also called opaque hosts.
    Domain(String),

    /// An IPv4 address.
    Ipv4(Ipv4Addr),

    /// An IPv6 address.
    Ipv6(Ipv6Addr),
}

impl Host {
    pub fn resolve(self) -> Option<Self> {
        if let Self::Domain(domain) = self {
            let ip = format!("{domain}:0")
                .to_socket_addrs()
                .ok()
                .and_then(|mut it| it.next())
                .map(|a| a.ip())?;
            Some(ip.into())
        } else {
            Some(self)
        }
    }
}

impl<'a> From<&'a Host> for multiaddr::Protocol<'a> {
    fn from(value: &'a Host) -> Self {
        match value {
            Host::Domain(v) => multiaddr::Protocol::Dns4(v.into()),
            Host::Ipv4(v) => multiaddr::Protocol::Ip4(*v),
            Host::Ipv6(v) => multiaddr::Protocol::Ip6(*v),
        }
    }
}

mod binprot_impl {
    use super::*;
    use binprot::{BinProtRead, BinProtWrite};
    use binprot_derive::{BinProtRead, BinProtWrite};
    use mina_p2p_messages::string::CharString;

    #[derive(BinProtWrite, BinProtRead)]
    enum HostKind {
        Domain,
        Ipv4,
        Ipv6,
    }

    impl BinProtWrite for Host {
        fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
            match self {
                Self::Domain(v) => {
                    HostKind::Domain.binprot_write(w)?;
                    let v = CharString::from(v.as_bytes());
                    v.binprot_write(w)?
                }
                Self::Ipv4(v) => {
                    HostKind::Ipv4.binprot_write(w)?;
                    for b in v.octets() {
                        b.binprot_write(w)?;
                    }
                }
                Self::Ipv6(v) => {
                    HostKind::Ipv6.binprot_write(w)?;
                    for b in v.segments() {
                        b.binprot_write(w)?;
                    }
                }
            };
            Ok(())
        }
    }

    impl BinProtRead for Host {
        fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
        where
            Self: Sized,
        {
            let kind = HostKind::binprot_read(r)?;

            Ok(match kind {
                HostKind::Domain => {
                    // TODO(binier): maybe limit length?
                    let s = CharString::binprot_read(r)?;
                    Host::from_str(&s.to_string_lossy())
                        .map_err(|err| binprot::Error::CustomError(err.into()))?
                }
                HostKind::Ipv4 => {
                    let mut octets = [0; 4];
                    for octet in &mut octets {
                        *octet = u8::binprot_read(r)?;
                    }

                    Host::Ipv4(octets.into())
                }
                HostKind::Ipv6 => {
                    let mut segments = [0; 8];
                    for segment in &mut segments {
                        *segment = u16::binprot_read(r)?;
                    }

                    Host::Ipv6(segments.into())
                }
            })
        }
    }
}

impl FromStr for Host {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(url::Host::parse(s)?.into())
    }
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        url::Host::from(self).fmt(f)
    }
}

impl From<[u8; 4]> for Host {
    fn from(value: [u8; 4]) -> Self {
        Self::Ipv4(value.into())
    }
}

impl From<url::Host> for Host {
    fn from(value: url::Host) -> Self {
        match value {
            url::Host::Domain(v) => Host::Domain(v),
            url::Host::Ipv4(v) => Host::Ipv4(v),
            url::Host::Ipv6(v) => Host::Ipv6(v),
        }
    }
}

impl<'a> From<&'a Host> for url::Host<&'a str> {
    fn from(value: &'a Host) -> Self {
        match value {
            Host::Domain(v) => url::Host::Domain(v),
            Host::Ipv4(v) => url::Host::Ipv4(*v),
            Host::Ipv6(v) => url::Host::Ipv6(*v),
        }
    }
}

impl From<IpAddr> for Host {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(v4) => Host::Ipv4(v4),
            IpAddr::V6(v6) => Host::Ipv6(v6),
        }
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for Host address resolution and parsing
    //!
    //! Run these tests with:
    //! ```bash
    //! cargo test -p p2p webrtc::host::tests
    //! ```

    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_resolve_ipv4_unchanged() {
        let host = Host::Ipv4(Ipv4Addr::new(192, 168, 1, 1));
        let resolved = host.resolve().unwrap();

        match resolved {
            Host::Ipv4(addr) => assert_eq!(addr, Ipv4Addr::new(192, 168, 1, 1)),
            _ => panic!("Expected IPv4 variant unchanged"),
        }
    }

    #[test]
    fn test_resolve_ipv6_unchanged() {
        let host = Host::Ipv6(Ipv6Addr::LOCALHOST);
        let resolved = host.resolve().unwrap();

        match resolved {
            Host::Ipv6(addr) => assert_eq!(addr, Ipv6Addr::LOCALHOST),
            _ => panic!("Expected IPv6 variant unchanged"),
        }
    }

    #[test]
    fn test_resolve_localhost_domain() {
        let host = Host::Domain("localhost".to_string());
        let resolved = host.resolve();

        // localhost should resolve to either 127.0.0.1 or ::1
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();

        match resolved {
            Host::Ipv4(addr) => {
                // Should be 127.0.0.1 or similar loopback
                assert!(addr.is_loopback());
            }
            Host::Ipv6(addr) => {
                // Should be ::1 or similar loopback
                assert!(addr.is_loopback());
            }
            Host::Domain(_) => panic!("Expected domain to resolve to IP address"),
        }
    }

    #[test]
    fn test_resolve_invalid_domain() {
        let host = Host::Domain("invalid.domain.that.should.not.exist.xyz123".to_string());
        let resolved = host.resolve();

        // Invalid domain should return None
        assert!(resolved.is_none());
    }

    #[test]
    fn test_resolve_empty_domain() {
        let host = Host::Domain("".to_string());
        let resolved = host.resolve();

        // Empty domain should return None
        assert!(resolved.is_none());
    }

    #[test]
    fn test_from_str_ipv4() {
        let host: Host = "192.168.1.1".parse().unwrap();
        match host {
            Host::Ipv4(addr) => assert_eq!(addr, Ipv4Addr::new(192, 168, 1, 1)),
            _ => panic!("Expected IPv4 variant"),
        }
    }

    #[test]
    fn test_from_str_ipv6() {
        let host: Host = "[::1]".parse().unwrap();
        match host {
            Host::Ipv6(addr) => assert_eq!(addr, Ipv6Addr::LOCALHOST),
            _ => panic!("Expected IPv6 variant"),
        }
    }

    #[test]
    fn test_from_str_ipv6_brackets() {
        let host: Host = "[::1]".parse().unwrap();
        match host {
            Host::Ipv6(addr) => assert_eq!(addr, Ipv6Addr::LOCALHOST),
            _ => panic!("Expected IPv6 variant"),
        }
    }

    #[test]
    fn test_from_str_domain() {
        let host: Host = "example.com".parse().unwrap();
        match host {
            Host::Domain(domain) => assert_eq!(domain, "example.com"),
            _ => panic!("Expected Domain variant"),
        }
    }

    #[test]
    fn test_from_str_invalid() {
        let result: Result<Host, _> = "not a valid host".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_display_ipv4() {
        let host = Host::Ipv4(Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(host.to_string(), "10.0.0.1");
    }

    #[test]
    fn test_display_ipv6() {
        let host = Host::Ipv6(Ipv6Addr::LOCALHOST);
        assert_eq!(host.to_string(), "[::1]");
    }

    #[test]
    fn test_display_domain() {
        let host = Host::Domain("test.example.org".to_string());
        assert_eq!(host.to_string(), "test.example.org");
    }

    #[test]
    fn test_roundtrip_ipv4() {
        let original = Host::Ipv4(Ipv4Addr::new(203, 0, 113, 42));
        let serialized = original.to_string();
        let deserialized: Host = serialized.parse().unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_roundtrip_ipv6() {
        let original = Host::Ipv6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
        let serialized = original.to_string();
        let deserialized: Host = serialized.parse().unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_roundtrip_domain() {
        let original = Host::Domain("api.example.net".to_string());
        let serialized = original.to_string();
        let deserialized: Host = serialized.parse().unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_from_ipaddr_v4() {
        let ip = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
        let host = Host::from(ip);
        match host {
            Host::Ipv4(addr) => assert_eq!(addr, Ipv4Addr::new(172, 16, 0, 1)),
            _ => panic!("Expected IPv4 variant"),
        }
    }

    #[test]
    fn test_from_ipaddr_v6() {
        let ip = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
        let host = Host::from(ip);
        match host {
            Host::Ipv6(addr) => assert_eq!(addr, Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)),
            _ => panic!("Expected IPv6 variant"),
        }
    }

    #[test]
    fn test_from_array_ipv4() {
        let bytes = [10, 0, 0, 1];
        let host = Host::from(bytes);
        match host {
            Host::Ipv4(addr) => assert_eq!(addr, Ipv4Addr::new(10, 0, 0, 1)),
            _ => panic!("Expected IPv4 variant"),
        }
    }

    #[test]
    fn test_ord_comparison() {
        let host1 = Host::Domain("a.example.com".to_string());
        let host2 = Host::Domain("b.example.com".to_string());
        let host3 = Host::Ipv4(Ipv4Addr::new(1, 1, 1, 1));
        let host4 = Host::Ipv4(Ipv4Addr::new(2, 2, 2, 2));

        assert!(host1 < host2);
        assert!(host3 < host4);
        // Domain variants should have consistent ordering with IP variants
        assert!(host1.partial_cmp(&host3).is_some());
    }

    #[test]
    fn test_clone_and_equality() {
        let original = Host::Domain("clone-test.example.com".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);

        let different = Host::Domain("different.example.com".to_string());
        assert_ne!(original, different);
    }

    #[test]
    fn test_multiaddr_protocol_conversion() {
        use multiaddr::Protocol;

        let domain_host = Host::Domain("test.com".to_string());
        let protocol = Protocol::from(&domain_host);
        if let Protocol::Dns4(cow_str) = protocol {
            assert_eq!(cow_str, "test.com");
        } else {
            panic!("Expected Dns4 protocol");
        }

        let ipv4_host = Host::Ipv4(Ipv4Addr::new(1, 2, 3, 4));
        let protocol = Protocol::from(&ipv4_host);
        if let Protocol::Ip4(addr) = protocol {
            assert_eq!(addr, Ipv4Addr::new(1, 2, 3, 4));
        } else {
            panic!("Expected Ip4 protocol");
        }

        let ipv6_host = Host::Ipv6(Ipv6Addr::LOCALHOST);
        let protocol = Protocol::from(&ipv6_host);
        if let Protocol::Ip6(addr) = protocol {
            assert_eq!(addr, Ipv6Addr::LOCALHOST);
        } else {
            panic!("Expected Ip6 protocol");
        }
    }

    #[test]
    fn test_serde_serialization() {
        let host = Host::Domain("serialize-test.example.com".to_string());
        let serialized = serde_json::to_string(&host).unwrap();
        let deserialized: Host = serde_json::from_str(&serialized).unwrap();
        assert_eq!(host, deserialized);
    }

    #[test]
    fn test_special_domains() {
        // Test some special/edge case domains
        let cases = vec![
            ("localhost", true),       // Should resolve
            ("127.0.0.1", true),       // Already an IP, but valid as domain too
            ("0.0.0.0", true),         // Valid IP
            ("255.255.255.255", true), // Valid IP
        ];

        for (domain_str, should_parse) in cases {
            let result: Result<Host, _> = domain_str.parse();
            assert_eq!(
                result.is_ok(),
                should_parse,
                "Failed for domain: {}",
                domain_str
            );
        }
    }
}
