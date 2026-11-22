use axum::http::{HeaderMap, header};
use std::net::{IpAddr, SocketAddr};

/// Explicit listener origins, never derived from a request or forwarded headers.
pub(crate) struct RequestPolicy {
    hosts: Vec<String>,
    origins: Vec<String>,
}

impl RequestPolicy {
    pub(crate) fn new(port: u16, lan_urls: &[String]) -> Self {
        let mut hosts = vec![format!("localhost:{port}"), format!("127.0.0.1:{port}")];
        hosts.extend(lan_urls.iter().filter_map(|url| {
            let authority = url.strip_prefix("http://")?;
            let address: SocketAddr = authority.parse().ok()?;
            (address.is_ipv4() && address.port() == port).then(|| authority.to_owned())
        }));
        // Browsers serialize the default HTTP port without an explicit :80.
        if port == 80 {
            let bare: Vec<_> = hosts
                .iter()
                .filter_map(|h| h.strip_suffix(":80").map(str::to_owned))
                .collect();
            hosts.extend(bare);
        }
        let origins = hosts.iter().map(|host| format!("http://{host}")).collect();
        Self { hosts, origins }
    }

    pub(crate) fn require_own_origin(&self, headers: &HeaderMap) -> Result<(), &'static str> {
        let host = single_header(headers, header::HOST).ok_or("Missing or invalid Host")?;
        if !self
            .hosts
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(host))
        {
            return Err("Untrusted Host");
        }
        if headers.contains_key(header::ORIGIN) {
            let origin = single_header(headers, header::ORIGIN).ok_or("Invalid Origin")?;
            if !self
                .origins
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(origin))
            {
                return Err("Untrusted Origin");
            }
        }
        Ok(())
    }
}

fn single_header(headers: &HeaderMap, name: header::HeaderName) -> Option<&str> {
    let mut values = headers.get_all(name).iter();
    let value = values.next()?.to_str().ok()?;
    values.next().is_none().then_some(value)
}

pub(crate) fn is_json(headers: &HeaderMap) -> bool {
    single_header(headers, header::CONTENT_TYPE)
        .and_then(|value| value.split(';').next())
        .is_some_and(|media_type| media_type.trim().eq_ignore_ascii_case("application/json"))
}

/// Check the TCP peer for host-only operations, including IPv4-mapped IPv6 loopback.
/// HTTP handlers map rejection to 403 before reading the request body.
pub fn require_loopback(peer: SocketAddr) -> Result<(), &'static str> {
    let local = match peer.ip() {
        IpAddr::V4(ip) => ip.is_loopback(),
        IpAddr::V6(ip) => {
            ip.is_loopback() || ip.to_ipv4_mapped().is_some_and(|ip| ip.is_loopback())
        }
    };
    if local {
        Ok(())
    } else {
        Err("Only the host can create, load, or save games. Use localhost on the host computer.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_is_required_and_duplicate_headers_fail_closed() {
        let policy = RequestPolicy::new(7878, &[]);
        let mut headers = HeaderMap::new();
        assert!(policy.require_own_origin(&headers).is_err());
        headers.insert(header::HOST, "localhost:7878".parse().unwrap());
        assert!(policy.require_own_origin(&headers).is_ok());
        headers.append(header::HOST, "localhost:7878".parse().unwrap());
        assert!(policy.require_own_origin(&headers).is_err());
        headers.insert(header::HOST, "localhost:7878".parse().unwrap());
        headers.insert(header::ORIGIN, "http://localhost:7878".parse().unwrap());
        assert!(policy.require_own_origin(&headers).is_ok());
        headers.append(header::ORIGIN, "http://localhost:7878".parse().unwrap());
        assert!(policy.require_own_origin(&headers).is_err());

        assert!(!is_json(&headers));
        headers.insert(
            header::CONTENT_TYPE,
            "application/json; charset=utf-8".parse().unwrap(),
        );
        assert!(is_json(&headers));
        headers.append(header::CONTENT_TYPE, "text/plain".parse().unwrap());
        assert!(!is_json(&headers));
    }

    #[test]
    fn default_http_port_accepts_browser_serialized_origins() {
        let policy = RequestPolicy::new(80, &["http://192.168.1.2:80".into()]);
        for name in ["localhost", "127.0.0.1", "192.168.1.2"] {
            for host in [name.to_owned(), format!("{name}:80")] {
                let mut headers = HeaderMap::new();
                headers.insert(header::HOST, host.parse().unwrap());
                headers.insert(header::ORIGIN, format!("http://{name}").parse().unwrap());
                assert!(policy.require_own_origin(&headers).is_ok());
                assert!(
                    RequestPolicy::new(7878, &[])
                        .require_own_origin(&headers)
                        .is_err()
                );
            }
        }
    }

    #[test]
    fn host_guard_rejects_non_loopback_addresses() {
        for peer in [
            "192.168.1.20:1234",
            "10.0.0.2:1234",
            "0.0.0.0:1234",
            "[::]:1234",
            "[2001:db8::1]:1234",
            "[::ffff:192.168.1.20]:1234",
        ] {
            assert!(require_loopback(peer.parse().unwrap()).is_err());
        }
        for peer in [
            "127.0.0.1:1234",
            "127.1.2.3:1234",
            "[::1]:1234",
            "[::ffff:127.0.0.1]:1234",
        ] {
            assert_eq!(require_loopback(peer.parse().unwrap()), Ok(()));
        }
    }
}
