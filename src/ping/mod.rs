pub mod fallback;
pub mod stats;

use std::net::IpAddr;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum PingReply {
    Ok { rtt_ms: f64 },
    Timeout,
    Error(String),
}

pub async fn ping_once(ip: IpAddr, timeout: Duration) -> PingReply {
    fallback::system_ping_once(&ip.to_string(), timeout).await
}
