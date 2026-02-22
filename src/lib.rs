pub mod config;
pub mod provider;
pub mod router;
pub mod health;
pub mod proxy;
pub mod cli;

use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use provider::ProviderMap;

pub struct RpcRouter {
    providers: ProviderMap,
    client: Client,
}
impl RpcRouter {
    pub async fn new() -> Result<Self> {
        let config = config::load_config()?;
        let providers = provider::create_provider_map(&config.providers);

        let providers_health = providers.clone();
        tokio::spawn(async move {
            health::start_health_checker(providers_health).await.unwrap();
        });
        Ok(Self { providers, client: Client::new() })
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value> {
        let urls = router::route(&self.providers, method);
      let url = urls.first().ok_or(anyhow::anyhow!("no healthy providers"))?;
      let body = serde_json::json!({"jsonrpc":"2.0","method":method,"params":params,"id":1});
        Ok(self.client.post(url).json(&body).send().await?.json::<Value>().await?)
    }

    pub async fn broadcast(&self, method: &str, params: Value) -> Result<Value> {
    let body = serde_json::json!({"jsonrpc":"2.0","method":method,"params":params,"id":1});
    let bytes = axum::body::Bytes::from(serde_json::to_vec(&body)?);
      let result = proxy::broadcast(&self.client, &self.providers, bytes)
        .await.ok_or(anyhow::anyhow!("all providers failed"))?;

    Ok(serde_json::from_slice(&result)?)
    }
    pub async fn failover(&self, method: &str, params: Value) -> Result<Value> {
        let mut providers: Vec<_> = self.providers
            .iter()
            .filter(|p| p.healthy && !p.cool_off)
            .map(|p| (p.score(), p.url.clone()))
            .collect();
        providers.sort_by_key(|(score, _)| *score);
    let body = serde_json::json!({"jsonrpc":"2.0","method":method,"params":params,"id":1});
        for (_, url) in providers {
            if let Ok(res) = self.client.post(&url).json(&body).send().await {
                if let Ok(json) = res.json::<Value>().await {
                    return Ok(json);
                }
            }
        }
        Err(anyhow::anyhow!("all providers failed"))
    }
    pub fn health(&self) -> Vec<serde_json::Value> {
    self.providers.iter().map(|p| {
        serde_json::json!({
            "name": p.name,
            "healthy": p.healthy,
            "latency": p.average_latency,
            "score": if p.average_latency == 0 { 0 } else { p.score() },
        })
    }).collect()
}
}