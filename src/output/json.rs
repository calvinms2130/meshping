use crate::ping::PingReply;
use crate::ping::stats::PingStats;
use crate::scan::{ScanResult, PortState};
use crate::discover::HostInfo;
use crate::trace::TraceHop;
use crate::lookup::LookupResult;
use super::OutputFormatter;
use serde_json::json;

pub struct JsonFormatter {
    replies: std::cell::RefCell<Vec<serde_json::Value>>,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self {
            replies: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl OutputFormatter for JsonFormatter {
    fn ping_header(&self, _host: &str, _ip: &str) {}

    fn ping_reply(&self, reply: &PingReply, seq: u32) {
        let val = match reply {
            PingReply::Ok { rtt_ms } => json!({
                "seq": seq,
                "rtt_ms": rtt_ms,
                "status": "ok"
            }),
            PingReply::Timeout => json!({
                "seq": seq,
                "status": "timeout"
            }),
            PingReply::Error(msg) => json!({
                "seq": seq,
                "status": "error",
                "error": msg
            }),
        };
        self.replies.borrow_mut().push(val);
    }

    fn ping_summary(&self, stats: &PingStats) {
        let output = json!({
            "host": stats.host,
            "ip": stats.ip,
            "sent": stats.sent,
            "received": stats.received,
            "lost": stats.lost,
            "loss_percent": stats.loss_percent(),
            "rtt": {
                "min_ms": stats.min_ms,
                "avg_ms": stats.avg_ms,
                "max_ms": stats.max_ms,
                "stddev_ms": stats.stddev_ms,
                "jitter_ms": stats.jitter_ms,
            },
            "replies": *self.replies.borrow(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }

    fn scan_result(&self, result: &ScanResult) {
        let ports: Vec<serde_json::Value> = result
            .ports
            .iter()
            .filter(|p| p.state == PortState::Open)
            .map(|p| {
                json!({
                    "port": p.port,
                    "state": "open",
                    "service": p.service,
                })
            })
            .collect();

        let output = json!({
            "host": result.host,
            "ip": result.ip,
            "ports": ports,
            "scan_time_ms": result.elapsed_ms,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }

    fn discover_result(&self, hosts: &[HostInfo], subnet: &str, elapsed_ms: u64) {
        let hosts_json: Vec<serde_json::Value> = hosts
            .iter()
            .map(|h| {
                json!({
                    "ip": h.ip,
                    "rtt_ms": h.rtt_ms,
                    "hostname": h.hostname,
                })
            })
            .collect();

        let output = json!({
            "subnet": subnet,
            "hosts": hosts_json,
            "total_up": hosts.len(),
            "elapsed_ms": elapsed_ms,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }

    fn trace_result(&self, hops: &[TraceHop], host: &str, ip: &str) {
        let hops_json: Vec<serde_json::Value> = hops
            .iter()
            .map(|h| {
                json!({
                    "ttl": h.ttl,
                    "ip": h.ip,
                    "rtt_ms": h.rtt_ms,
                    "hostname": h.hostname,
                })
            })
            .collect();

        let output = json!({
            "host": host,
            "ip": ip,
            "hops": hops_json,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }

    fn lookup_result(&self, result: &LookupResult) {
        let output = json!({
            "query": result.query,
            "a_records": result.a_records.iter().map(|r| json!({"value": r.value, "ttl": r.ttl})).collect::<Vec<_>>(),
            "aaaa_records": result.aaaa_records.iter().map(|r| json!({"value": r.value, "ttl": r.ttl})).collect::<Vec<_>>(),
            "mx_records": result.mx_records.iter().map(|r| json!({"value": r.value, "ttl": r.ttl})).collect::<Vec<_>>(),
            "ptr_records": result.ptr_records.iter().map(|r| json!({"value": r.value, "ttl": r.ttl})).collect::<Vec<_>>(),
            "elapsed_ms": result.elapsed_ms,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
}
