use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

/// Attempt to read a service banner from an open TCP connection.
pub async fn grab_banner(mut stream: TcpStream, timeout: Duration) -> Option<String> {
    let mut buf = [0u8; 256];
    match tokio::time::timeout(timeout, stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            let raw = String::from_utf8_lossy(&buf[..n]);
            let cleaned: String = raw
                .chars()
                .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                .take(64)
                .collect();
            if cleaned.trim().is_empty() {
                None
            } else {
                Some(cleaned.trim().to_string())
            }
        }
        _ => None,
    }
}
