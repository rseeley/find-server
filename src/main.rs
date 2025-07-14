use std::net::IpAddr;
use std::str::FromStr;
use tokio::time::{Duration, timeout};

use futures::{StreamExt, stream};
use reqwest::Client;
use tokio;

const HIGHEST_IP_QUADRANT_VALUE: usize = 254;
const PARALLEL_REQUESTS: usize = 254;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

async fn check_ip(client: &Client, ip: IpAddr) -> Option<IpAddr> {
    let resp = timeout(
        REQUEST_TIMEOUT,
        client.get(format!("http://{}:8096", ip)).send(),
    )
    .await;

    match resp {
        Ok(Ok(r)) if r.status().is_success() => Some(ip),
        _ => None,
    }
}

#[tokio::main]
async fn main() {
    let ips: Vec<IpAddr> = (1..=HIGHEST_IP_QUADRANT_VALUE)
        .map(|n| format!("10.0.0.{}", n))
        .filter_map(|s| IpAddr::from_str(&s).ok())
        .collect();

    let client = Client::new();

    let bodies = stream::iter(ips)
        .map(|ip| {
            let client = client.clone();

            tokio::spawn(async move { check_ip(&client, ip).await })
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

    println!("Reachable IPs:");
    for ip in valid_ips {
        println!("  http://{}", ip);
    }
}

// TODO: add clap to allow IP pattern and port to be configurable
