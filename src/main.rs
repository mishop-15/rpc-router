mod config;
mod router;
mod provider;
mod health;
mod proxy;
mod cli;

use clap::Parser;
use cli::{Cli, Commands};
use config::load_config;
use provider::create_provider_map;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = load_config()?;
    let providers = create_provider_map(&config.providers);

    match cli.command {
        Commands::Start => {
            let port = std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(config.settings.port);
            
        let providers_health = providers.clone();
            tokio::spawn(async move {
                health::start_health_checker(providers_health).await.unwrap();
            });
            proxy::start_proxy(providers, port).await?;
        }
        Commands::Benchmark => {
        cli::run(Commands::Benchmark, providers).await?;
        }
    }
    Ok(())
}