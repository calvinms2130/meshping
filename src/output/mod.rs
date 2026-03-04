pub mod json;
pub mod table;

use crate::ping::PingReply;
use crate::ping::stats::PingStats;
use crate::scan::ScanResult;
use crate::discover::HostInfo;
use crate::trace::TraceHop;
use crate::lookup::LookupResult;

pub trait OutputFormatter {
    fn ping_reply(&self, reply: &PingReply, seq: u32);
    fn ping_summary(&self, stats: &PingStats);
    fn ping_header(&self, host: &str, ip: &str);
    fn scan_result(&self, result: &ScanResult);
    fn discover_result(&self, hosts: &[HostInfo], subnet: &str, elapsed_ms: u64);
    fn trace_result(&self, hops: &[TraceHop], host: &str, ip: &str);
    fn lookup_result(&self, result: &LookupResult);
}
