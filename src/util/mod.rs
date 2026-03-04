pub mod platform;

use std::net::IpAddr;
use anyhow::Result;

/// Resolve a hostname to an IP address. If already an IP, returns it directly.
pub async fn resolve_host(host: &str) -> Result<IpAddr> {
    // Try parsing as IP first
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ip);
    }

    // DNS lookup
    let addrs = tokio::net::lookup_host(format!("{host}:0")).await?;
    for addr in addrs {
        return Ok(addr.ip());
    }

    Err(anyhow::anyhow!("Could not resolve hostname: {host}"))
}
