use clap::Parser;
use futures::{StreamExt, stream};
use reqwest::Client;
use std::net::IpAddr;
use std::str::FromStr;
use tokio;
use tokio::time::{Duration, timeout};

const HIGHEST_IP_QUADRANT_VALUE: usize = 254;
const PARALLEL_REQUESTS: usize = 254;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Cli {
    // IP pattern
    #[arg(short, long, default_value_t = String::from("10.0.0.{}"))]
    ip_pattern: String,

    // Port to look for
    #[arg(short, long, default_value_t = 80)]
    port: u16,
}

async fn check_ip(client: &Client, ip: IpAddr, port: u16) -> Option<IpAddr> {
    let resp = timeout(
        REQUEST_TIMEOUT,
        client
            .get(format!("http://{}:{}", ip, port.to_string().as_str()))
            .send(),
    )
    .await;

    match resp {
        Ok(Ok(r)) if r.status().is_success() => Some(ip),
        _ => None,
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let ips: Vec<IpAddr> = (1..=HIGHEST_IP_QUADRANT_VALUE)
        .map(|n| args.ip_pattern.replace("{}", n.to_string().as_str()))
        .filter_map(|s| IpAddr::from_str(&s).ok())
        .collect();

    let client = Client::new();

    let bodies = stream::iter(ips)
        .map(|ip| {
            let client = client.clone();

            tokio::spawn(async move { check_ip(&client, ip, args.port).await })
        })
        .buffer_unordered(PARALLEL_REQUESTS);

    let valid_ips = bodies
        .filter_map(|body| async move {
            match body {
                Ok(Some(ip)) => Some(ip),
                _ => None,
            }
        })
        .collect::<Vec<_>>()
        .await;

    if valid_ips.len() == 0 {
        println!("No reachable IPs!");
        return;
    }

    let port_string = args.port.to_string();
    let port_str = port_string.as_str();

    println!("Reachable IPs (on port {}):", port_str);
    for ip in valid_ips {
        println!("  http://{}", ip);
    }
}

// TODO: add clap to allow IP pattern and port to be configurable
