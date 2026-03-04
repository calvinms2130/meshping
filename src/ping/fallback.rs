use std::time::Duration;
use tokio::process::Command;

use super::PingReply;

/// Ping using the system `ping` command. Works without admin on all platforms.
pub async fn system_ping_once(host: &str, timeout: Duration) -> PingReply {
    let timeout_ms = timeout.as_millis() as u64;

    let result = if cfg!(target_os = "windows") {
        Command::new("ping")
            .args(["-n", "1", "-w", &timeout_ms.to_string(), host])
            .output()
            .await
    } else {
        let timeout_secs = (timeout_ms as f64 / 1000.0).ceil() as u64;
        Command::new("ping")
            .args(["-c", "1", "-W", &timeout_secs.to_string(), host])
            .output()
            .await
    };

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                parse_rtt(&stdout)
            } else {
                if stdout.contains("timed out") || stdout.contains("100% packet loss") || stdout.contains("100% loss") {
                    PingReply::Timeout
                } else {
                    PingReply::Error(format!("ping failed: {}", stdout.lines().last().unwrap_or("unknown error")))
                }
            }
        }
        Err(e) => PingReply::Error(format!("Failed to execute ping: {e}")),
    }
}

fn parse_rtt(output: &str) -> PingReply {
    // Windows: "Reply from x.x.x.x: bytes=32 time=12ms TTL=56"
    // Windows <1ms: "Reply from x.x.x.x: bytes=32 time<1ms TTL=128"
    // Linux/macOS: "64 bytes from x.x.x.x: icmp_seq=1 ttl=56 time=12.3 ms"
    for line in output.lines() {
        // Windows format: time=NNms or time<1ms
        if let Some(pos) = line.find("time=") {
            let after = &line[pos + 5..];
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
            if let Ok(ms) = num_str.parse::<f64>() {
                return PingReply::Ok { rtt_ms: ms };
            }
        }
        if line.contains("time<1ms") {
            return PingReply::Ok { rtt_ms: 0.5 };
        }
        // Linux/macOS format: time=NN.N ms
        if let Some(pos) = line.find("time=") {
            let after = &line[pos + 5..];
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
            if let Ok(ms) = num_str.parse::<f64>() {
                return PingReply::Ok { rtt_ms: ms };
            }
        }
    }

    // Check for timeout indicators
    if output.contains("Request timed out") || output.contains("100% packet loss") {
        return PingReply::Timeout;
    }

    PingReply::Ok { rtt_ms: 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_windows_output() {
        let output = "Reply from 142.250.80.46: bytes=32 time=12ms TTL=56";
        match parse_rtt(output) {
            PingReply::Ok { rtt_ms } => assert!((rtt_ms - 12.0).abs() < 0.1),
            other => panic!("Expected Ok, got {:?}", other),
        }
    }

    #[test]
    fn parse_windows_sub_1ms() {
        let output = "Reply from 127.0.0.1: bytes=32 time<1ms TTL=128";
        match parse_rtt(output) {
            PingReply::Ok { rtt_ms } => assert!(rtt_ms < 1.0),
            other => panic!("Expected Ok, got {:?}", other),
        }
    }

    #[test]
    fn parse_linux_output() {
        let output = "64 bytes from 142.250.80.46: icmp_seq=1 ttl=56 time=12.3 ms";
        match parse_rtt(output) {
            PingReply::Ok { rtt_ms } => assert!((rtt_ms - 12.3).abs() < 0.1),
            other => panic!("Expected Ok, got {:?}", other),
        }
    }

    #[test]
    fn parse_timeout_output() {
        let output = "Request timed out.";
        match parse_rtt(output) {
            PingReply::Timeout => {}
            other => panic!("Expected Timeout, got {:?}", other),
        }
    }
}
