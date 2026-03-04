use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TraceHop {
    pub ttl: u8,
    pub ip: Option<String>,
    pub rtt_ms: Option<f64>,
    pub hostname: Option<String>,
}
