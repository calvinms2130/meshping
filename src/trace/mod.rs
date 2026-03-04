pub mod hop;
pub use hop::TraceHop;

use anyhow::Result;
use tokio::process::Command;

/// Trace route to a host using the system traceroute/tracert command.
pub async fn trace_route(host: &str, max_hops: u8, timeout_secs: f64) -> Result<Vec<TraceHop>> {
    let output = if cfg!(target_os = "windows") {
        Command::new("tracert")
            .args([
                "-h",
                &max_hops.to_string(),
                "-w",
                &((timeout_secs * 1000.0) as u64).to_string(),
                host,
            ])
            .output()
            .await?
    } else {
        Command::new("traceroute")
            .args([
                "-m",
                &max_hops.to_string(),
                "-w",
                &timeout_secs.to_string(),
                host,
            ])
            .output()
            .await?
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let hops = parse_traceroute_output(&stdout);
    Ok(hops)
}

fn parse_traceroute_output(output: &str) -> Vec<TraceHop> {
    let mut hops = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // Skip header lines
        if trimmed.is_empty()
            || trimmed.starts_with("Tracing")
            || trimmed.starts_with("traceroute")
            || trimmed.starts_with("Trace complete")
            || trimmed.starts_with("over a maximum")
        {
            continue;
        }

        // Try to parse hop number at start of line
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let ttl: u8 = match parts[0].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };

        // Check for timeout line (all asterisks)
        if trimmed.contains("Request timed out") || parts.iter().all(|p| *p == "*" || p.parse::<u8>().is_ok()) {
            hops.push(TraceHop {
                ttl,
                ip: None,
                rtt_ms: None,
                hostname: None,
            });
            continue;
        }

        // Extract IP address (look for pattern like x.x.x.x or [x.x.x.x])
        let mut ip = None;
        let mut hostname = None;
        let mut rtt_ms = None;

        for (i, part) in parts.iter().enumerate() {
            // IP in brackets (Windows format): [1.2.3.4]
            if part.starts_with('[') && part.ends_with(']') {
                ip = Some(part.trim_matches(|c| c == '[' || c == ']').to_string());
            }
            // Bare IP address
            else if part.chars().filter(|c| *c == '.').count() == 3
                && part.chars().all(|c| c.is_ascii_digit() || c == '.')
            {
                ip = Some(part.to_string());
            }
            // RTT value (number followed by "ms")
            else if part.ends_with("ms") || (i + 1 < parts.len() && parts[i + 1] == "ms") {
                let num_str = part.trim_end_matches("ms");
                if let Ok(ms) = num_str.parse::<f64>() {
                    if rtt_ms.is_none() {
                        rtt_ms = Some(ms);
                    }
                }
            } else if let Ok(ms) = part.parse::<f64>() {
                if i + 1 < parts.len() && parts[i + 1] == "ms" && rtt_ms.is_none() {
                    rtt_ms = Some(ms);
                }
            }
            // Hostname (contains letters and dots, not a number)
            else if part.contains('.') && part.chars().any(|c| c.is_ascii_alphabetic()) {
                hostname = Some(part.to_string());
            }
        }

        hops.push(TraceHop {
            ttl,
            ip,
            rtt_ms,
            hostname,
        });
    }

    hops
}
