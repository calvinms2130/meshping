use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use ipnetwork::IpNetwork;
use tokio::sync::Semaphore;
use anyhow::Result;

use crate::error::MeshpingError;
use crate::ping::{ping_once, PingReply};
use super::HostInfo;

/// Parse a CIDR string and return all host IPs (excluding network and broadcast).
pub fn cidr_to_hosts(cidr: &str) -> Result<Vec<IpAddr>> {
    let network: IpNetwork = cidr
        .parse()
        .map_err(|_| MeshpingError::InvalidCidr(cidr.to_string()))?;

    let hosts: Vec<IpAddr> = network.iter().collect();

    // For /32 or /128, return as-is; otherwise skip network + broadcast
    if network.prefix() >= 31 {
        Ok(hosts)
    } else {
        Ok(hosts.into_iter().skip(1).rev().skip(1).rev().collect())
    }
}

/// Discover live hosts on a subnet via concurrent ping sweep.
pub async fn discover_hosts(
    cidr: &str,
    timeout: Duration,
    concurrency: usize,
    progress_cb: impl Fn(usize, usize) + Send + Sync + 'static,
) -> Result<Vec<HostInfo>> {
    let hosts = cidr_to_hosts(cidr)?;
    let total = hosts.len();
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let progress_cb = Arc::new(progress_cb);

    let mut tasks = Vec::with_capacity(total);
    let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    for ip in hosts {
        let sem = semaphore.clone();
        let completed = completed.clone();
        let progress_cb = progress_cb.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let reply = ping_once(ip, timeout).await;
            let done = completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            progress_cb(done, total);

            match reply {
                PingReply::Ok { rtt_ms } => Some(HostInfo {
                    ip: ip.to_string(),
                    rtt_ms,
                    hostname: None, // Reverse DNS done separately
                }),
                _ => None,
            }
        }));
    }

    let mut live_hosts = Vec::new();
    for task in tasks {
        if let Ok(Some(host)) = task.await {
            live_hosts.push(host);
        }
    }

    live_hosts.sort_by(|a, b| {
        let a_ip: IpAddr = a.ip.parse().unwrap();
        let b_ip: IpAddr = b.ip.parse().unwrap();
        a_ip.cmp(&b_ip)
    });

    Ok(live_hosts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cidr_24_gives_254_hosts() {
        let hosts = cidr_to_hosts("192.168.1.0/24").unwrap();
        assert_eq!(hosts.len(), 254);
        assert_eq!(hosts[0].to_string(), "192.168.1.1");
        assert_eq!(hosts[253].to_string(), "192.168.1.254");
    }

    #[test]
    fn cidr_32_gives_1_host() {
        let hosts = cidr_to_hosts("10.0.0.1/32").unwrap();
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].to_string(), "10.0.0.1");
    }

    #[test]
    fn cidr_30_gives_2_hosts() {
        let hosts = cidr_to_hosts("10.0.0.0/30").unwrap();
        assert_eq!(hosts.len(), 2);
    }

    #[test]
    fn invalid_cidr() {
        assert!(cidr_to_hosts("not-a-cidr").is_err());
    }
}
