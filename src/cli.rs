use anyhow::Result;
use clap::{Parser, Subcommand};
use std::time::{Duration, Instant};
use crate::provider::ProviderMap;


#[derive(Parser)]
#[command(name = "rpc-router")]
#[command(about = "Solana RPC Load Balancer & Router")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Start,
    Benchmark,
}

pub async fn run(command: Commands, providers: ProviderMap) -> Result<()> {
    match command {
        Commands::Start => {
            println!("starting rpc router...");
        }
        Commands::Benchmark => {
            println!("Benchmarking all providers...\n");
            println!("{:<20} {:<12} {:<10}", "Provider", "Latency(ms)", "Status");
            println!("{}", "-".repeat(45));

            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()?;
            let payload = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "getHealth",
                "params": [],
                "id": 1
            });

            let mut best_provider = String::new();
        let mut best_latency = u128::MAX;
            for entry in providers.iter() {
                let p = entry.value();
                 let start = Instant::now();
                let result = client.post(&p.url).json(&payload).send().await;
                let latency = start.elapsed().as_millis();
                let status = match result {
                    Ok(res) if res.status().is_success() => {
                        if latency < best_latency {
                            best_latency = latency;
                            best_provider = p.name.clone();
                        }
                        "healthy"
                    }
                    _ => "unhealthy",
                };
                println!("{:<20} {:<12} {:<10}", p.name, latency, status);
            }

            println!("{}", "-".repeat(45));
            if !best_provider.is_empty() {
                println!("best provider: {} ({}ms)", best_provider, best_latency);
            }
        }
    }
    Ok(())
}