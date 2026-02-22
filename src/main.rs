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
            let providers_health = providers.clone();
            tokio::spawn(async move {
                health::start_health_checker(providers_health).await.unwrap();
            });
            proxy::start_proxy(providers, config.settings.port).await?;
        }
        Commands::Benchmark => {
        cli::run(Commands::Benchmark, providers).await?;
        }
    }
    Ok(())
}