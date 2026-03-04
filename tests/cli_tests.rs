use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_output() {
    Command::cargo_bin("meshping")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("network diagnostic toolkit"));
}

#[test]
fn test_version() {
    Command::cargo_bin("meshping")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("meshping"));
}

#[test]
fn test_ping_help() {
    Command::cargo_bin("meshping")
        .unwrap()
        .args(["ping", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ICMP echo"));
}

#[test]
fn test_scan_help() {
    Command::cargo_bin("meshping")
        .unwrap()
        .args(["scan", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TCP ports"));
}

#[test]
fn test_discover_help() {
    Command::cargo_bin("meshping")
        .unwrap()
        .args(["discover", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("live hosts"));
}

#[test]
fn test_trace_help() {
    Command::cargo_bin("meshping")
        .unwrap()
        .args(["trace", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("route"));
}

#[test]
fn test_lookup_help() {
    Command::cargo_bin("meshping")
        .unwrap()
        .args(["lookup", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DNS"));
}

#[test]
fn test_no_subcommand_shows_help() {
    Command::cargo_bin("meshping")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_scan_with_mock_server() {
    // Bind a TCP listener to verify the scanner can find open ports
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    Command::cargo_bin("meshping")
        .unwrap()
        .args(["scan", "127.0.0.1", "-p", &port.to_string(), "-t", "1000"])
        .assert()
        .success()
        .stdout(predicate::str::contains("open"));
}

#[test]
fn test_scan_json_output() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    Command::cargo_bin("meshping")
        .unwrap()
        .args([
            "scan", "127.0.0.1",
            "-p", &port.to_string(),
            "-t", "1000",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"port\""));
}

#[test]
fn test_ping_localhost() {
    Command::cargo_bin("meshping")
        .unwrap()
        .args(["ping", "127.0.0.1", "-c", "1"])
        .assert()
        .success();
}

#[test]
fn test_lookup_localhost() {
    Command::cargo_bin("meshping")
        .unwrap()
        .args(["lookup", "localhost"])
        .assert()
        .success();
}
