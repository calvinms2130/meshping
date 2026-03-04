pub mod resolver;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LookupResult {
    pub query: String,
    pub a_records: Vec<DnsRecord>,
    pub aaaa_records: Vec<DnsRecord>,
    pub mx_records: Vec<DnsRecord>,
    pub ptr_records: Vec<DnsRecord>,
    pub elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct DnsRecord {
    pub value: String,
    pub ttl: u32,
}
