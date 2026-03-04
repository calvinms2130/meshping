use console::style;
use comfy_table::{Table, ContentArrangement, presets::NOTHING};

use crate::ping::PingReply;
use crate::ping::stats::PingStats;
use crate::scan::{ScanResult, PortResult, PortState};
use crate::discover::HostInfo;
use crate::trace::TraceHop;
use crate::lookup::LookupResult;
use super::OutputFormatter;

pub struct TableFormatter;

impl TableFormatter {
    pub fn new() -> Self {
        Self
    }

    fn brand() -> String {
        format!(" {} ", style("meshping").cyan().bold())
    }
}

impl OutputFormatter for TableFormatter {
    fn ping_header(&self, host: &str, ip: &str) {
        println!(
            "{} PING {} ({})\n",
            Self::brand(),
            style(host).bold(),
            style(ip).dim()
        );
        let mut table = Table::new();
        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                style("  #").dim().to_string(),
                style("RTT").dim().to_string(),
                style("Status").dim().to_string(),
            ]);
        println!("{table}");
    }

    fn ping_reply(&self, reply: &PingReply, seq: u32) {
        let (rtt_str, status) = match reply {
            PingReply::Ok { rtt_ms } => {
                let rtt = if *rtt_ms > 500.0 {
                    style(format!("{:.1} ms", rtt_ms)).red().to_string()
                } else if *rtt_ms > 100.0 {
                    style(format!("{:.1} ms", rtt_ms)).yellow().to_string()
                } else {
                    style(format!("{:.1} ms", rtt_ms)).to_string()
                };
                (rtt, style("OK").green().bold().to_string())
            }
            PingReply::Timeout => {
                ("—".to_string(), style("TIMEOUT").red().to_string())
            }
            PingReply::Error(msg) => {
                ("—".to_string(), style(format!("ERR: {msg}")).red().to_string())
            }
        };
        println!("  {}   {:>10}    {}", style(seq).dim(), rtt_str, status);
    }

    fn ping_summary(&self, stats: &PingStats) {
        println!();
        println!(
            " {} {}",
            style("──").dim(),
            style("Statistics").bold()
        );
        println!(
            "  Sent: {}  Received: {}  Lost: {} ({:.1}%)",
            style(stats.sent).bold(),
            style(stats.received).green(),
            if stats.lost > 0 {
                style(stats.lost).red().to_string()
            } else {
                style(stats.lost).to_string()
            },
            stats.loss_percent()
        );
        if stats.received > 0 {
            println!(
                "  RTT  min/avg/max/stddev = {:.1}/{:.1}/{:.1}/{:.1} ms",
                stats.min_ms, stats.avg_ms, stats.max_ms, stats.stddev_ms
            );
            if stats.received > 1 {
                println!("  Jitter: {:.1} ms", stats.jitter_ms);
            }
        }
        println!();
    }

    fn scan_result(&self, result: &ScanResult) {
        println!();
        println!(
            " {} {}",
            style("──").dim(),
            style("Open Ports").bold()
        );

        let open_ports: Vec<&PortResult> = result
            .ports
            .iter()
            .filter(|p| p.state == PortState::Open)
            .collect();

        if open_ports.is_empty() {
            println!("  No open ports found.");
        } else {
            let mut table = Table::new();
            table
                .load_preset(NOTHING)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    style("  PORT").dim().to_string(),
                    style("STATE").dim().to_string(),
                    style("SERVICE").dim().to_string(),
                ]);

            for port in &open_ports {
                table.add_row(vec![
                    format!("  {}", style(port.port).cyan()),
                    style("open").green().bold().to_string(),
                    port.service.clone().unwrap_or_default(),
                ]);
            }
            println!("{table}");
        }

        let closed = result.ports.len() - open_ports.len();
        println!();
        println!(
            " {} {}",
            style("──").dim(),
            style("Summary").bold()
        );
        println!(
            "  {} open, {} closed/filtered",
            style(open_ports.len()).green().bold(),
            closed
        );
        println!(
            "  Scan completed in {:.1}s",
            result.elapsed_ms as f64 / 1000.0
        );
        println!();
    }

    fn discover_result(&self, hosts: &[HostInfo], subnet: &str, elapsed_ms: u64) {
        println!();
        println!(
            " {} {}",
            style("──").dim(),
            style("Live Hosts").bold()
        );

        if hosts.is_empty() {
            println!("  No live hosts found.");
        } else {
            let mut table = Table::new();
            table
                .load_preset(NOTHING)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    style("  IP").dim().to_string(),
                    style("RTT").dim().to_string(),
                    style("HOSTNAME").dim().to_string(),
                ]);

            for host in hosts {
                table.add_row(vec![
                    format!("  {}", style(&host.ip).cyan()),
                    format!("{:.1} ms", host.rtt_ms),
                    host.hostname.clone().unwrap_or_else(|| "(no PTR)".to_string()),
                ]);
            }
            println!("{table}");
        }

        println!();
        println!(
            " {} {}",
            style("──").dim(),
            style("Summary").bold()
        );
        println!(
            "  {} hosts up on {}",
            style(hosts.len()).green().bold(),
            subnet
        );
        println!("  Completed in {:.1}s", elapsed_ms as f64 / 1000.0);
        println!();
    }

    fn trace_result(&self, hops: &[TraceHop], host: &str, ip: &str) {
        println!(
            "\n{} Traceroute to {} ({})\n",
            Self::brand(),
            style(host).bold(),
            style(ip).dim()
        );

        let mut table = Table::new();
        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                style("  HOP").dim().to_string(),
                style("IP").dim().to_string(),
                style("RTT").dim().to_string(),
                style("HOSTNAME").dim().to_string(),
            ]);

        for hop in hops {
            let rtt = if let Some(ms) = hop.rtt_ms {
                format!("{:.1} ms", ms)
            } else {
                "*".to_string()
            };
            table.add_row(vec![
                format!("  {}", style(hop.ttl).dim()),
                hop.ip.clone().unwrap_or_else(|| "*".to_string()),
                rtt,
                hop.hostname.clone().unwrap_or_default(),
            ]);
        }
        println!("{table}");
        println!();
    }

    fn lookup_result(&self, result: &LookupResult) {
        println!(
            "\n{} DNS Lookup: {}\n",
            Self::brand(),
            style(&result.query).bold()
        );

        if !result.a_records.is_empty() {
            println!(
                " {} {}",
                style("──").dim(),
                style("A Records").bold()
            );
            for record in &result.a_records {
                println!("  {}          TTL: {}", style(record.value.clone()).cyan(), record.ttl);
            }
            println!();
        }

        if !result.aaaa_records.is_empty() {
            println!(
                " {} {}",
                style("──").dim(),
                style("AAAA Records").bold()
            );
            for record in &result.aaaa_records {
                println!("  {}   TTL: {}", style(record.value.clone()).cyan(), record.ttl);
            }
            println!();
        }

        if !result.mx_records.is_empty() {
            println!(
                " {} {}",
                style("──").dim(),
                style("MX Records").bold()
            );
            for record in &result.mx_records {
                println!("  {}   TTL: {}", style(record.value.clone()).cyan(), record.ttl);
            }
            println!();
        }

        if !result.ptr_records.is_empty() {
            println!(
                " {} {}",
                style("──").dim(),
                style("PTR Records").bold()
            );
            for record in &result.ptr_records {
                println!("  {}", style(record.value.clone()).cyan());
            }
            println!();
        }

        println!(
            "  Resolved in {} ms",
            style(result.elapsed_ms).green()
        );
        println!();
    }
}
