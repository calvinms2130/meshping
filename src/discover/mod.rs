pub mod subnet;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HostInfo {
    pub ip: String,
    pub rtt_ms: f64,
    pub hostname: Option<String>,
}
