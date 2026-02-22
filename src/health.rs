use std::time::Instant;
use std::time::Duration;
use reqwest::Client;
use serde_json::json;
use crate::provider::ProviderMap;
use crate::config::load_config;

pub fn mask_url(url: &str) -> String {
     if let Some(idx) = url.find("api-key=") {
        return format!("{}api-key=***", &url[..idx]);
    }
    if url.contains("quiknode.pro") {
        if let Some(idx) = url.rfind('/') {
            let second_last = url[..idx].rfind('/');
            if let Some(start) = second_last {
                return format!("{}/***/", &url[..start]);
            }
        }
    }

    url.to_string()
}
pub async fn start_health_checker(providers: ProviderMap) -> anyhow::Result<()> {
    let config = load_config()?;
    let client = Client::new();
    loop {
        for provider in &config.providers {
            let payload = json!({
                "jsonrpc": "2.0",
                "method": "getHealth",
                "params": [],
                "id": 1
            });
            let start = Instant::now();
            let response = client
                .post(&provider.url)
                .json(&payload)
                .send()
                .await;
            let latency = start.elapsed().as_millis();
            match response {
                Ok(res) if res.status().is_success() => {
                    if let Some(mut state) = providers.get_mut(&provider.name) {
                        state.healthy = true;
                        state.error_count = 0;
                        state.cool_off = false;
                        state.average_latency = if state.average_latency == 0 {
                            latency
                        } else {
                            (state.average_latency * 4 + latency) / 5
                        };
                    }
                    println!("{} is healthy | latency: {}ms | url: {}", 
                        provider.name, latency, mask_url(&provider.url));
                }
                _ => {
                    if let Some(mut state) = providers.get_mut(&provider.name) {
                        state.error_count += 1;
                        if state.error_count >= 3 {
                            state.healthy = false;
                            state.cool_off = true;
                            println!("{} in cooloff! will retry next cycle", provider.name);
                        }
                    }
                    println!("{} is unhealthy", provider.name);
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}