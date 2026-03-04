pub mod tcp_connect;
pub mod ports;
pub mod service;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub host: String,
    pub ip: String,
    pub ports: Vec<PortResult>,
    pub elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct PortResult {
    pub port: u16,
    pub state: PortState,
    pub service: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum PortState {
    Open,
    Closed,
}
