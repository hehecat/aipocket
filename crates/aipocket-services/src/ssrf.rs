use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use url::{Host, Url};

#[derive(Debug, Default)]
struct PublicDnsResolver;

impl Resolve for PublicDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_owned();
        Box::pin(async move {
            let addresses = tokio::net::lookup_host((host.as_str(), 0))
                .await
                .map_err(boxed_error)?
                .collect::<Vec<_>>();
            checked_addresses(&host, addresses).map_err(boxed_error)
        })
    }
}

fn boxed_error(error: io::Error) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(error)
}

fn checked_addresses(host: &str, addresses: Vec<SocketAddr>) -> io::Result<Addrs> {
    if addresses.is_empty() {
        return Err(policy_error(format!(
            "DNS returned no addresses for {host}"
        )));
    }
    if let Some(address) = addresses.iter().find(|address| !is_public_ip(address.ip())) {
        return Err(policy_error(format!(
            "destination {host} resolved to blocked address {}",
            address.ip()
        )));
    }
    Ok(Box::new(addresses.into_iter()))
}

fn policy_error(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message.into())
}

pub(crate) fn validate_destination(url: &Url) -> Result<(), String> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err("destination must use http or https".into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("destination URL credentials are not allowed".into());
    }
    match url.host() {
        Some(Host::Ipv4(address)) if !is_public_ip(IpAddr::V4(address)) => {
            Err(format!("destination address {address} is blocked"))
        }
        Some(Host::Ipv6(address)) if !is_public_ip(IpAddr::V6(address)) => {
            Err(format!("destination address {address} is blocked"))
        }
        Some(_) => Ok(()),
        None => Err("destination host is required".into()),
    }
}

pub(crate) fn validate_target(target: &str) -> Result<Url, String> {
    let url = Url::parse(target).map_err(|error| format!("invalid destination URL: {error}"))?;
    validate_destination(&url)?;
    Ok(url)
}

pub(crate) fn restricted_client(
    timeout_seconds: f64,
    max_redirects: usize,
) -> Result<reqwest::Client, String> {
    let timeout = Duration::try_from_secs_f64(timeout_seconds)
        .map_err(|error| format!("invalid probe timeout {timeout_seconds}: {error}"))?;
    if timeout.is_zero() {
        return Err("probe timeout must be greater than zero".into());
    }
    restricted_client_with_resolver(timeout, max_redirects, Arc::new(PublicDnsResolver))
        .map_err(|error| error.to_string())
}

fn restricted_client_with_resolver<R>(
    timeout: Duration,
    max_redirects: usize,
    resolver: Arc<R>,
) -> Result<reqwest::Client, reqwest::Error>
where
    R: Resolve + 'static,
{
    let redirects = reqwest::redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() >= max_redirects {
            return attempt.error("probe redirect limit exceeded");
        }
        match validate_destination(attempt.url()) {
            Ok(()) => attempt.follow(),
            Err(error) => attempt.error(format!("blocked probe redirect: {error}")),
        }
    });
    reqwest::Client::builder()
        .timeout(timeout)
        .redirect(redirects)
        .dns_resolver(resolver)
        .no_proxy()
        .build()
}

fn is_public_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_public_ipv4(address),
        IpAddr::V6(address) => is_public_ipv6(address),
    }
}

fn is_public_ipv4(address: Ipv4Addr) -> bool {
    let [a, b, c, _] = address.octets();
    !(a == 0
        || a == 10
        || a == 100 && (64..=127).contains(&b)
        || a == 127
        || a == 169 && b == 254
        || a == 172 && (16..=31).contains(&b)
        || a == 192 && b == 0 && c == 0
        || a == 192 && b == 0 && c == 2
        || a == 192 && b == 168
        || a == 198 && (b == 18 || b == 19)
        || a == 198 && b == 51 && c == 100
        || a == 203 && b == 0 && c == 113
        || a >= 224)
}

fn is_public_ipv6(address: Ipv6Addr) -> bool {
    if let Some(mapped) = address.to_ipv4_mapped() {
        return is_public_ipv4(mapped);
    }
    let segments = address.segments();
    // Accept global unicast only, excluding the documentation prefix 2001:db8::/32.
    (segments[0] & 0xe000) == 0x2000 && !(segments[0] == 0x2001 && segments[1] == 0x0db8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use std::collections::HashMap;

    #[derive(Debug)]
    struct StaticResolver {
        addresses: HashMap<String, Vec<SocketAddr>>,
    }

    impl Resolve for StaticResolver {
        fn resolve(&self, name: Name) -> Resolving {
            let host = name.as_str().to_owned();
            let addresses = self.addresses.get(&host).cloned().unwrap_or_default();
            Box::pin(async move { checked_addresses(&host, addresses).map_err(boxed_error) })
        }
    }

    fn socket(ip: &str, port: u16) -> SocketAddr {
        SocketAddr::new(ip.parse().unwrap(), port)
    }

    #[test]
    fn blocks_literal_ipv4_destination_classes() {
        for address in [
            "0.0.0.0",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.0.1",
            "127.0.0.1",
            "100.64.0.1",
            "169.254.1.1",
            "192.0.2.1",
            "198.18.0.1",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "240.0.0.1",
        ] {
            assert!(
                validate_target(&format!("http://{address}/probe")).is_err(),
                "{address}"
            );
        }
        assert!(validate_target("https://8.8.8.8/probe").is_ok());
    }

    #[test]
    fn blocks_literal_ipv6_destination_classes_and_mapped_ipv4() {
        for address in [
            "::",
            "::1",
            "fc00::1",
            "fd12::1",
            "fe80::1",
            "ff02::1",
            "::ffff:127.0.0.1",
        ] {
            assert!(
                validate_target(&format!("http://[{address}]/probe")).is_err(),
                "{address}"
            );
        }
        assert!(validate_target("https://[2606:4700:4700::1111]/probe").is_ok());
    }

    #[test]
    fn rejects_non_http_missing_host_and_url_credentials() {
        for target in [
            "file:///etc/passwd",
            "mailto:user@example.com",
            "http://",
            "http://user:pass@example.com",
        ] {
            assert!(validate_target(target).is_err(), "{target}");
        }
    }

    #[test]
    fn rejects_invalid_probe_timeouts_without_panicking() {
        for timeout in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(restricted_client(timeout, 1).is_err(), "timeout={timeout}");
        }
    }

    #[test]
    fn dns_policy_rejects_empty_private_ipv4_ipv6_and_mixed_answers() {
        assert!(checked_addresses("empty.test", vec![]).is_err());
        assert!(checked_addresses("v4.test", vec![socket("10.0.0.1", 0)]).is_err());
        assert!(checked_addresses("v6.test", vec![socket("fd00::1", 0)]).is_err());
        assert!(
            checked_addresses(
                "mixed.test",
                vec![socket("93.184.216.34", 0), socket("127.0.0.1", 0)]
            )
            .is_err()
        );
        assert!(
            checked_addresses(
                "public.test",
                vec![
                    socket("93.184.216.34", 0),
                    socket("2606:4700:4700::1111", 0)
                ]
            )
            .is_ok()
        );
    }

    #[tokio::test]
    async fn blocks_redirect_hop_before_private_dns_connection() {
        let app = Router::new().route(
            "/",
            get(|| async {
                (
                    axum::http::StatusCode::FOUND,
                    [(axum::http::header::LOCATION, "http://127.0.0.1/secret")],
                )
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let resolver = StaticResolver {
            addresses: HashMap::from([("public.test".into(), vec![socket("127.0.0.1", port)])]),
        };
        // The test-only resolver maps the first hostname to loopback after policy checking is
        // deliberately bypassed for that entry by using reqwest's explicit resolve override.
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::custom(
                |attempt| match validate_destination(attempt.url()) {
                    Ok(()) => attempt.follow(),
                    Err(error) => attempt.error(error),
                },
            ))
            .resolve("public.test", socket("127.0.0.1", port))
            .dns_resolver(Arc::new(resolver))
            .no_proxy()
            .build()
            .unwrap();
        let error = client.get("http://public.test/").send().await.unwrap_err();
        assert!(error.is_redirect());
    }
}
