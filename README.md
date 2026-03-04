# meshping — Cross-Platform Network Diagnostic Toolkit

[![CI](https://github.com/calvinms2130/meshping/actions/workflows/ci.yml/badge.svg)](https://github.com/calvinms2130/meshping/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Ping, scan, discover, trace, and lookup — all in one fast, colorful CLI. Built in Rust for speed and cross-platform support. No admin/root required for most features.

---

## Features

- **Ping** — ICMP echo with RTT statistics, jitter, standard deviation, and packet loss tracking
- **Port Scan** — Async TCP connect scanner with concurrent connections, progress bar, and service detection
- **Host Discovery** — Subnet sweep to find all live hosts on a network
- **Traceroute** — Trace the network path to any destination
- **DNS Lookup** — Forward (A, AAAA, MX) and reverse (PTR) DNS resolution
- **JSON Export** — Machine-readable output for scripting and automation (`--json`)
- **Colored Output** — Status-aware colors: green for OK, yellow for slow, red for timeout/error
- **Cross-Platform** — Works on Windows, Linux, and macOS without admin privileges

---

## Install

### From source

```bash
git clone https://github.com/calvinms2130/meshping.git
cd meshping
cargo build --release
# Binary at: target/release/meshping
```

### From crates.io (coming soon)

```bash
cargo install meshping
```

---

## Usage

### Ping a host

```bash
# Ping google.com 4 times (default)
meshping ping google.com

# Ping with custom count and interval
meshping ping 8.8.8.8 -c 10 -i 0.5

# Continuous ping
meshping ping google.com -c 0
```

**Output:**
```
 meshping  PING google.com (142.250.80.46)

  #       RTT    Status
  1    12.4 ms    OK
  2    11.8 ms    OK
  3    13.1 ms    OK
  4    12.0 ms    OK

 ── Statistics
  Sent: 4  Received: 4  Lost: 0 (0.0%)
  RTT  min/avg/max/stddev = 11.8/12.3/13.1/0.5 ms
  Jitter: 0.7 ms
```

### Scan ports

```bash
# Scan default ports (1-1024)
meshping scan example.com

# Scan specific ports
meshping scan localhost -p 80,443,8080,3000

# Scan a range with service detection
meshping scan 192.168.1.1 -p 1-65535 --service

# Increase concurrency for faster scans
meshping scan example.com -p 1-10000 -c 500
```

**Output:**
```
 meshping  Scanning example.com (93.184.216.34) ports 1-1024

  [########################################] 1024/1024 ports scanned

 ── Open Ports
  PORT     STATE    SERVICE
  80       open     http
  443      open     https

 ── Summary
  2 open, 1022 closed/filtered
  Scan completed in 3.2s
```

### Discover hosts on your network

```bash
# Scan your local subnet
meshping discover 192.168.1.0/24

# Faster with higher concurrency
meshping discover 10.0.0.0/24 -c 100 -t 500
```

**Output:**
```
 meshping  Discovering hosts on 192.168.1.0/24

  [########################################] 254/254 hosts probed

 ── Live Hosts
  IP               RTT        HOSTNAME
  192.168.1.1      1.2 ms     router.local
  192.168.1.10     0.8 ms     desktop-pc.local
  192.168.1.42     1.5 ms     phone.local

 ── Summary
  3 hosts up on 192.168.1.0/24
  Completed in 5.1s
```

### Trace route

```bash
meshping trace google.com
meshping trace 8.8.8.8 -m 15 -t 3.0
```

**Output:**
```
 meshping  Traceroute to google.com (142.250.80.46)

  HOP  IP               RTT        HOSTNAME
  1    192.168.1.1      1.2 ms     router.local
  2    10.0.0.1         5.4 ms     isp-gateway.net
  3    72.14.237.89     8.1 ms
  4    142.250.80.46    12.3 ms    nrt12s51-in-f14.1e100.net
```

### DNS lookup

```bash
# Forward lookup
meshping lookup google.com

# Reverse lookup (IP → hostname)
meshping lookup -r 8.8.8.8
```

**Output:**
```
 meshping  DNS Lookup: google.com

 ── A Records
  142.250.80.46          TTL: 3600

 ── AAAA Records
  2404:6800:4004:824::200e   TTL: 3600

  Resolved in 12 ms
```

### JSON output (for scripting)

```bash
# Any command supports --json
meshping ping google.com -c 2 --json
meshping scan localhost -p 80,443 --json
meshping lookup example.com --json

# Pipe to jq for filtering
meshping scan example.com -p 1-1024 --json | jq '.ports[] | select(.state == "open")'
```

**JSON output example:**
```json
{
  "host": "example.com",
  "ip": "93.184.216.34",
  "ports": [
    { "port": 80, "state": "open", "service": "http" },
    { "port": 443, "state": "open", "service": "https" }
  ],
  "scan_time_ms": 3200,
  "timestamp": "2026-03-04T15:30:00Z"
}
```

---

## Command Reference

```
meshping <COMMAND> [OPTIONS]

Commands:
  ping       Send ICMP echo requests to a host
  scan       Scan TCP ports on a target host
  discover   Discover live hosts on a local subnet
  trace      Trace the route to a destination host
  lookup     DNS lookup (forward and reverse)

Global Options:
  -j, --json       Output results as JSON
  --no-color       Disable colored output
  -v, --verbose    Increase output verbosity
  -h, --help       Print help
  -V, --version    Print version
```

| Command | Key Options |
|---------|------------|
| `ping` | `-c` count, `-i` interval (s), `-t` timeout (s) |
| `scan` | `-p` ports (e.g. "80,443" or "1-1024"), `-c` concurrency, `-t` timeout (ms), `--service` |
| `discover` | `-c` concurrency, `-t` timeout (ms) |
| `trace` | `-m` max hops, `-t` timeout (s) |
| `lookup` | `-r` reverse lookup |

---

## Architecture

```
src/
├── main.rs              # Entry point, command dispatch
├── cli.rs               # Clap derive CLI definition (all subcommands)
├── error.rs             # Custom error types (thiserror)
├── output/
│   ├── mod.rs           # OutputFormatter trait
│   ├── table.rs         # Colored terminal tables (console + comfy-table)
│   └── json.rs          # JSON serialization (serde_json)
├── ping/
│   ├── mod.rs           # ICMP ping dispatcher
│   ├── fallback.rs      # System ping command parser (no admin needed)
│   └── stats.rs         # RTT statistics: min/max/avg/stddev/jitter
├── scan/
│   ├── mod.rs           # Scan result types
│   ├── tcp_connect.rs   # Async TCP scanner with semaphore concurrency
│   ├── ports.rs         # Port range parser + well-known service DB
│   └── service.rs       # Banner/service detection
├── discover/
│   ├── mod.rs           # Host discovery types
│   └── subnet.rs        # CIDR parsing + concurrent ping sweep
├── trace/
│   ├── mod.rs           # Traceroute via system command + output parser
│   └── hop.rs           # Per-hop data structures
├── lookup/
│   ├── mod.rs           # DNS result types
│   └── resolver.rs      # hickory-resolver (A, AAAA, MX, PTR)
└── util/
    ├── mod.rs           # Hostname resolution helper
    └── platform.rs      # Platform detection, privilege checks
```

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| **System ping fallback** | Raw ICMP sockets require admin on Windows. Parsing `ping.exe` / `ping` output works without elevation on all platforms. |
| **Async TCP scanning** | `tokio` + `Semaphore` enables scanning thousands of ports concurrently without overwhelming the OS socket limit. |
| **Trait-based output** | `OutputFormatter` trait allows switching between colored tables and JSON with zero code duplication. |
| **hickory-resolver** | Pure Rust async DNS — no dependency on system resolver libraries. |
| **No unsafe code** | Entire codebase is safe Rust. |

---

## Platform Notes

| Feature | Windows | Linux | macOS |
|---------|---------|-------|-------|
| Ping | No admin | No root | No root |
| Port Scan | No admin | No root | No root |
| Host Discovery | No admin | No root | No root |
| Traceroute | Uses `tracert` | Uses `traceroute` | Uses `traceroute` |
| Colored Output | Windows Terminal, PowerShell | All terminals | All terminals |

---

## Testing

25 unit tests + 12 integration tests across all modules:

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --bin meshping
```

| Module | Tests | Coverage |
|--------|-------|----------|
| `ping/stats` | 7 | RTT min/max/avg/stddev, jitter, packet loss, edge cases |
| `ping/fallback` | 4 | Windows/Linux output parsing, sub-1ms, timeout detection |
| `scan/ports` | 10 | Range parsing, comma-separated, mixed, invalid input, service DB |
| `discover/subnet` | 4 | CIDR /24 /30 /32 iteration, invalid CIDR handling |
| `cli_tests` (integration) | 12 | All subcommands --help, ping localhost, scan mock server, JSON output |

CI runs on **3 platforms** via GitHub Actions: ubuntu, windows, macos.

---

## Tech Stack

| Component | Crate |
|-----------|-------|
| CLI Framework | `clap` v4 (derive API) |
| Async Runtime | `tokio` (full features) |
| DNS Resolver | `hickory-resolver` |
| Terminal Colors | `console` |
| ASCII Tables | `comfy-table` |
| Progress Bars | `indicatif` |
| CIDR Parsing | `ipnetwork` |
| Serialization | `serde` + `serde_json` |
| Error Handling | `anyhow` + `thiserror` |
| Timestamps | `chrono` |

---

## Use Cases

| Scenario | Command |
|----------|---------|
| Check if a server is up | `meshping ping api.example.com` |
| Find open ports on a host | `meshping scan 192.168.1.1 -p 1-65535` |
| Map all devices on your WiFi | `meshping discover 192.168.1.0/24` |
| Debug network routing issues | `meshping trace slow-server.com` |
| Verify DNS configuration | `meshping lookup myapp.com` |
| CI/CD health checks | `meshping ping api.prod.com -c 1 --json` |
| Security audit script | `meshping scan target.com -p 1-1024 --json \| jq '.ports[]'` |

---

## What This Project Demonstrates

| Skill | Implementation |
|-------|---------------|
| **Rust / Systems Programming** | Async networking, platform-specific compilation (`#[cfg]`), zero unsafe code |
| **Async Concurrency** | Tokio runtime, semaphore-based connection limiting, parallel host sweeps |
| **CLI Design** | Clap derive API, global flags, subcommands, colored output, JSON mode |
| **Cross-Platform Engineering** | Windows/Linux/macOS support, system command fallbacks, terminal compatibility |
| **Protocol Knowledge** | ICMP, TCP connect scanning, DNS resolution (A/AAAA/MX/PTR), traceroute TTL |
| **Software Architecture** | Trait-based output abstraction, modular crate structure, clean separation of concerns |
| **Testing** | 37 tests (unit + integration), CI on 3 OS platforms |
| **DevOps** | GitHub Actions CI/CD, cross-platform build matrix |

---

## License

MIT

---

**Built by [calvinms2130](https://github.com/calvinms2130)**
