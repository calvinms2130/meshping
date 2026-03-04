mod cli;
mod error;
mod output;
mod ping;
mod scan;
mod discover;
mod trace;
mod lookup;
mod util;

use std::time::{Duration, Instant};
use clap::Parser;
use console::style;

use cli::{Cli, Commands};
use output::OutputFormatter;
use output::table::TableFormatter;
use output::json::JsonFormatter;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.no_color {
        console::set_colors_enabled(false);
    }

    let result = match &cli.command {
        Commands::Ping(args) => cmd_ping(&cli, args).await,
        Commands::Scan(args) => cmd_scan(&cli, args).await,
        Commands::Discover(args) => cmd_discover(&cli, args).await,
        Commands::Trace(args) => cmd_trace(&cli, args).await,
        Commands::Lookup(args) => cmd_lookup(&cli, args).await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}

async fn cmd_ping(cli: &Cli, args: &cli::PingArgs) -> anyhow::Result<()> {
    let ip = util::resolve_host(&args.host).await?;
    let timeout = Duration::from_secs_f64(args.timeout);

    let formatter: Box<dyn OutputFormatter> = if cli.json {
        Box::new(JsonFormatter::new())
    } else {
        Box::new(TableFormatter::new())
    };

    formatter.ping_header(&args.host, &ip.to_string());

    let mut replies = Vec::new();
    let count = if args.count == 0 { u32::MAX } else { args.count };

    for seq in 1..=count {
        let reply = ping::ping_once(ip, timeout).await;
        formatter.ping_reply(&reply, seq);
        replies.push(reply);

        if seq < count {
            tokio::time::sleep(Duration::from_secs_f64(args.interval)).await;
        }
    }

    let stats = ping::stats::PingStats::from_results(&args.host, &ip.to_string(), &replies);
    formatter.ping_summary(&stats);

    Ok(())
}

async fn cmd_scan(cli: &Cli, args: &cli::ScanArgs) -> anyhow::Result<()> {
    let ip = util::resolve_host(&args.host).await?;
    let ports = scan::ports::parse_port_range(&args.ports)?;
    let timeout = Duration::from_millis(args.timeout);
    let total = ports.len();

    let formatter: Box<dyn OutputFormatter> = if cli.json {
        Box::new(JsonFormatter::new())
    } else {
        Box::new(TableFormatter::new())
    };

    if !cli.json {
        println!(
            "\n {} Scanning {} ({}) ports {}\n",
            style("meshping").cyan().bold(),
            style(&args.host).bold(),
            style(ip).dim(),
            style(&args.ports).dim()
        );
    }

    let pb = if !cli.json {
        let pb = indicatif::ProgressBar::new(total as u64);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("  [{bar:40.cyan/dim}] {pos}/{len} ports scanned")
                .unwrap()
                .progress_chars("##-"),
        );
        Some(pb)
    } else {
        None
    };

    let start = Instant::now();
    let results = scan::tcp_connect::scan_ports(ip, ports, timeout, args.concurrency, args.service).await;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    if let Some(pb) = &pb {
        pb.finish_and_clear();
    }

    let scan_result = scan::ScanResult {
        host: args.host.clone(),
        ip: ip.to_string(),
        ports: results,
        elapsed_ms,
    };

    formatter.scan_result(&scan_result);
    Ok(())
}

async fn cmd_discover(cli: &Cli, args: &cli::DiscoverArgs) -> anyhow::Result<()> {
    let timeout = Duration::from_millis(args.timeout);

    let formatter: Box<dyn OutputFormatter> = if cli.json {
        Box::new(JsonFormatter::new())
    } else {
        Box::new(TableFormatter::new())
    };

    if !cli.json {
        println!(
            "\n {} Discovering hosts on {}\n",
            style("meshping").cyan().bold(),
            style(&args.subnet).bold()
        );
    }

    let pb = if !cli.json {
        let pb = indicatif::ProgressBar::new(0);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("  [{bar:40.cyan/dim}] {pos}/{len} hosts probed")
                .unwrap()
                .progress_chars("##-"),
        );
        Some(std::sync::Arc::new(pb))
    } else {
        None
    };

    let pb_clone = pb.clone();
    let start = Instant::now();

    let hosts = discover::subnet::discover_hosts(
        &args.subnet,
        timeout,
        args.concurrency,
        move |done, total| {
            if let Some(ref pb) = pb_clone {
                pb.set_length(total as u64);
                pb.set_position(done as u64);
            }
        },
    )
    .await?;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    formatter.discover_result(&hosts, &args.subnet, elapsed_ms);
    Ok(())
}

async fn cmd_trace(cli: &Cli, args: &cli::TraceArgs) -> anyhow::Result<()> {
    let ip = util::resolve_host(&args.host).await?;

    let formatter: Box<dyn OutputFormatter> = if cli.json {
        Box::new(JsonFormatter::new())
    } else {
        Box::new(TableFormatter::new())
    };

    if !cli.json {
        println!(
            "\n {} Tracing route to {} ({})...\n",
            style("meshping").cyan().bold(),
            style(&args.host).bold(),
            style(ip).dim()
        );
    }

    let hops = trace::trace_route(&args.host, args.max_hops, args.timeout).await?;
    formatter.trace_result(&hops, &args.host, &ip.to_string());
    Ok(())
}

async fn cmd_lookup(cli: &Cli, args: &cli::LookupArgs) -> anyhow::Result<()> {
    let formatter: Box<dyn OutputFormatter> = if cli.json {
        Box::new(JsonFormatter::new())
    } else {
        Box::new(TableFormatter::new())
    };

    let result = if args.reverse {
        lookup::resolver::reverse_lookup(&args.host).await?
    } else {
        lookup::resolver::forward_lookup(&args.host).await?
    };

    formatter.lookup_result(&result);
    Ok(())
}
