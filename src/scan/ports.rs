use crate::error::MeshpingError;
use anyhow::Result;

/// Parse a port range string into a list of ports.
/// Supports: "80", "80,443,8080", "1-1024", "80,443,8000-9000"
pub fn parse_port_range(input: &str) -> Result<Vec<u16>> {
    let mut ports = Vec::new();

    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let bounds: Vec<&str> = part.splitn(2, '-').collect();
            if bounds.len() != 2 {
                return Err(MeshpingError::InvalidPortRange(part.to_string()).into());
            }
            let start: u16 = bounds[0]
                .parse()
                .map_err(|_| MeshpingError::InvalidPortRange(part.to_string()))?;
            let end: u16 = bounds[1]
                .parse()
                .map_err(|_| MeshpingError::InvalidPortRange(part.to_string()))?;
            if start > end {
                return Err(MeshpingError::InvalidPortRange(format!("{start} > {end}")).into());
            }
            for p in start..=end {
                ports.push(p);
            }
        } else {
            let port: u16 = part
                .parse()
                .map_err(|_| MeshpingError::InvalidPortRange(part.to_string()))?;
            ports.push(port);
        }
    }

    if ports.is_empty() {
        return Err(MeshpingError::InvalidPortRange(input.to_string()).into());
    }

    Ok(ports)
}

/// Get the well-known service name for a port.
pub fn service_name(port: u16) -> Option<&'static str> {
    match port {
        20 => Some("ftp-data"),
        21 => Some("ftp"),
        22 => Some("ssh"),
        23 => Some("telnet"),
        25 => Some("smtp"),
        53 => Some("dns"),
        80 => Some("http"),
        110 => Some("pop3"),
        111 => Some("rpcbind"),
        135 => Some("msrpc"),
        139 => Some("netbios"),
        143 => Some("imap"),
        443 => Some("https"),
        445 => Some("smb"),
        465 => Some("smtps"),
        587 => Some("submission"),
        993 => Some("imaps"),
        995 => Some("pop3s"),
        1433 => Some("mssql"),
        1521 => Some("oracle"),
        3306 => Some("mysql"),
        3389 => Some("rdp"),
        5432 => Some("postgresql"),
        5672 => Some("amqp"),
        5900 => Some("vnc"),
        6379 => Some("redis"),
        8080 => Some("http-alt"),
        8443 => Some("https-alt"),
        9090 => Some("prometheus"),
        9200 => Some("elasticsearch"),
        27017 => Some("mongodb"),
        _ => None,
    }
}

/// Return the top N most commonly scanned ports.
pub fn top_ports(n: usize) -> Vec<u16> {
    let common: Vec<u16> = vec![
        21, 22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445, 465,
        587, 993, 995, 1433, 1521, 3306, 3389, 5432, 5672, 5900, 6379,
        8080, 8443, 9090, 9200, 27017,
    ];
    common.into_iter().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_port() {
        let ports = parse_port_range("80").unwrap();
        assert_eq!(ports, vec![80]);
    }

    #[test]
    fn parse_comma_separated() {
        let ports = parse_port_range("80,443,8080").unwrap();
        assert_eq!(ports, vec![80, 443, 8080]);
    }

    #[test]
    fn parse_range() {
        let ports = parse_port_range("1-5").unwrap();
        assert_eq!(ports, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn parse_mixed() {
        let ports = parse_port_range("22,80,8000-8002").unwrap();
        assert_eq!(ports, vec![22, 80, 8000, 8001, 8002]);
    }

    #[test]
    fn parse_full_range() {
        let ports = parse_port_range("1-1024").unwrap();
        assert_eq!(ports.len(), 1024);
        assert_eq!(ports[0], 1);
        assert_eq!(ports[1023], 1024);
    }

    #[test]
    fn parse_invalid_range() {
        assert!(parse_port_range("abc").is_err());
    }

    #[test]
    fn parse_reversed_range() {
        assert!(parse_port_range("1024-1").is_err());
    }

    #[test]
    fn service_name_known() {
        assert_eq!(service_name(80), Some("http"));
        assert_eq!(service_name(443), Some("https"));
        assert_eq!(service_name(22), Some("ssh"));
        assert_eq!(service_name(3306), Some("mysql"));
    }

    #[test]
    fn service_name_unknown() {
        assert_eq!(service_name(12345), None);
    }

    #[test]
    fn top_ports_count() {
        let ports = top_ports(10);
        assert_eq!(ports.len(), 10);
    }
}
