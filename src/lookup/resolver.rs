use std::time::Instant;
use anyhow::Result;
use hickory_resolver::TokioAsyncResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};

use super::{LookupResult, DnsRecord};

pub async fn forward_lookup(host: &str) -> Result<LookupResult> {
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    let start = Instant::now();

    let mut result = LookupResult {
        query: host.to_string(),
        a_records: Vec::new(),
        aaaa_records: Vec::new(),
        mx_records: Vec::new(),
        ptr_records: Vec::new(),
        elapsed_ms: 0,
    };

    // A records
    if let Ok(response) = resolver.lookup_ip(host).await {
        for ip in response.iter() {
            match ip {
                std::net::IpAddr::V4(v4) => {
                    result.a_records.push(DnsRecord {
                        value: v4.to_string(),
                        ttl: 0,
                    });
                }
                std::net::IpAddr::V6(v6) => {
                    result.aaaa_records.push(DnsRecord {
                        value: v6.to_string(),
                        ttl: 0,
                    });
                }
            }
        }
    }

    // MX records
    if let Ok(response) = resolver.mx_lookup(host).await {
        for mx in response.iter() {
            result.mx_records.push(DnsRecord {
                value: mx.exchange().to_string(),
                ttl: 0,
            });
        }
    }

    result.elapsed_ms = start.elapsed().as_millis() as u64;
    Ok(result)
}

pub async fn reverse_lookup(ip: &str) -> Result<LookupResult> {
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    let start = Instant::now();

    let addr: std::net::IpAddr = ip.parse()?;

    let mut result = LookupResult {
        query: ip.to_string(),
        a_records: Vec::new(),
        aaaa_records: Vec::new(),
        mx_records: Vec::new(),
        ptr_records: Vec::new(),
        elapsed_ms: 0,
    };

    if let Ok(response) = resolver.reverse_lookup(addr).await {
        for name in response.iter() {
            result.ptr_records.push(DnsRecord {
                value: name.to_string(),
                ttl: 0,
            });
        }
    }

    result.elapsed_ms = start.elapsed().as_millis() as u64;
    Ok(result)
}
