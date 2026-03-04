use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "meshping",
    version,
    about = "A cross-platform network diagnostic toolkit",
    long_about = "Ping, scan, discover, trace, and lookup — all in one fast, colorful CLI."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output results as JSON
    #[arg(short, long, global = true)]
    pub json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Increase output verbosity
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Send ICMP echo requests to a host
    Ping(PingArgs),

    /// Scan TCP ports on a target host
    Scan(ScanArgs),

    /// Discover live hosts on a local subnet
    Discover(DiscoverArgs),

    /// Trace the route to a destination host
    Trace(TraceArgs),

    /// DNS lookup (forward and reverse)
    Lookup(LookupArgs),
}

#[derive(clap::Args)]
pub struct PingArgs {
    /// Target host (IP address or hostname)
    pub host: String,

    /// Number of pings to send (0 = infinite)
    #[arg(short, long, default_value_t = 4)]
    pub count: u32,

    /// Interval between pings in seconds
    #[arg(short, long, default_value_t = 1.0)]
    pub interval: f64,

    /// Timeout per ping in seconds
    #[arg(short, long, default_value_t = 2.0)]
    pub timeout: f64,
}

#[derive(clap::Args)]
pub struct ScanArgs {
    /// Target host (IP address or hostname)
    pub host: String,

    /// Port range (e.g. "80,443" or "1-1024")
    #[arg(short, long, default_value = "1-1024")]
    pub ports: String,

    /// Timeout per port in milliseconds
    #[arg(short, long, default_value_t = 500)]
    pub timeout: u64,

    /// Maximum concurrent connections
    #[arg(short, long, default_value_t = 200)]
    pub concurrency: usize,

    /// Detect services on open ports
    #[arg(long)]
    pub service: bool,
}

#[derive(clap::Args)]
pub struct DiscoverArgs {
    /// Subnet in CIDR notation (e.g. "192.168.1.0/24")
    pub subnet: String,

    /// Timeout per host in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    pub timeout: u64,

    /// Maximum concurrent probes
    #[arg(short, long, default_value_t = 50)]
    pub concurrency: usize,
}

#[derive(clap::Args)]
pub struct TraceArgs {
    /// Target host (IP address or hostname)
    pub host: String,

    /// Maximum number of hops
    #[arg(short, long, default_value_t = 30)]
    pub max_hops: u8,

    /// Timeout per hop in seconds
    #[arg(short, long, default_value_t = 2.0)]
    pub timeout: f64,
}

#[derive(clap::Args)]
pub struct LookupArgs {
    /// Hostname or IP address to look up
    pub host: String,

    /// Perform reverse DNS lookup (IP -> hostname)
    #[arg(short, long)]
    pub reverse: bool,
}
