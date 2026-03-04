use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;

use super::{PortResult, PortState};
use super::ports::service_name;
use super::service::grab_banner;

pub async fn scan_ports(
    ip: IpAddr,
    ports: Vec<u16>,
    timeout: Duration,
    concurrency: usize,
    detect_service: bool,
) -> Vec<PortResult> {
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut tasks = Vec::with_capacity(ports.len());

    for port in ports {
        let sem = semaphore.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let addr = SocketAddr::new(ip, port);

            let state = match tokio::time::timeout(timeout, TcpStream::connect(&addr)).await {
                Ok(Ok(stream)) => {
                    let banner = if detect_service {
                        grab_banner(stream, timeout).await
                    } else {
                        None
                    };
                    let svc = banner.or_else(|| service_name(port).map(|s| s.to_string()));
                    return PortResult {
                        port,
                        state: PortState::Open,
                        service: svc,
                    };
                }
                _ => PortState::Closed,
            };

            PortResult {
                port,
                state,
                service: None,
            }
        }));
    }

    let mut results = Vec::with_capacity(tasks.len());
    for task in tasks {
        if let Ok(result) = task.await {
            results.push(result);
        }
    }

    results.sort_by_key(|r| r.port);
    results
}
