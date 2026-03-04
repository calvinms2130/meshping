use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum MeshpingError {
    #[error("DNS resolution failed for '{host}': {reason}")]
    DnsError { host: String, reason: String },

    #[error("Connection to {host}:{port} failed: {reason}")]
    ConnectionError {
        host: String,
        port: u16,
        reason: String,
    },

    #[error("Ping failed for '{host}': {reason}")]
    PingError { host: String, reason: String },

    #[error("Invalid port range: {0}")]
    InvalidPortRange(String),

    #[error("Invalid CIDR notation: {0}")]
    InvalidCidr(String),

    #[error("Command execution failed: {0}")]
    CommandError(String),

    #[error("Timeout after {0} seconds")]
    Timeout(f64),

    #[error("{0}")]
    Other(String),
}
