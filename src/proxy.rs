use std::net::SocketAddr;
use axum::{Router, routing::post,extract::State,body::Bytes,response::Response, Json};
use reqwest::Client;
use futures::future::join_all;
use tower_http::cors::{CorsLayer, Any};
use tokio::net::TcpListener;
use crate::provider::ProviderMap;
use crate::router;

#[derive(Clone)]
pub struct AppState {
    pub providers: ProviderMap,
    pub client: Client,
}
pub async fn start_proxy(providers: ProviderMap, port: u16) -> anyhow::Result<()> {
    let state = AppState { providers, client: Client::new() };
    let listener = TcpListener::bind(
        SocketAddr::from(([0, 0, 0, 0], port))
    ).await?;
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    println!("proxy on http://0.0.0.0:{}", port);
    axum::serve(listener, Router::new()
        .route("/", post(handler))
        .route("/stats", axum::routing::get(stats))
        .layer(cors)
        .with_state(state)
    ).await?;
    Ok(())
}

pub async fn handler(State(s): State<AppState>, body: Bytes) -> Response {
    let method = serde_json::from_slice::<serde_json::Value>(&body)
        .ok()
        .and_then(|j| j["method"].as_str().map(String::from))
        .unwrap_or_default();

    println!("[{}] routing", method);

    let result = match method.as_str() {
        "getLatestBlockhash" | "sendTransaction" => {
            broadcast(&s.client, &s.providers, body).await
                .map(|b| (b, "broadcast".to_string()))
        }
        _ => retry(&s.client, &s.providers, &method, body).await,
    };
    match result {
        Some((b, provider)) => Response::builder()
            .status(200)
            .header("X-Routed-Via", provider)
            .header("Access-Control-Expose-Headers", "X-Routed-Via")
            .body(b.into()).unwrap(),
        None => Response::builder()
            .status(503)
            .body("no healthy providers".into()).unwrap(),
    }
}

pub async fn broadcast(client: &Client, providers: &ProviderMap, body: Bytes) -> Option<Bytes> {
    let responses = join_all(
        router::get_all_healthy(providers).iter().map(|url| {
            let (client, body, url) = (client.clone(), body.clone(), url.clone());
            async move { client.post(&url).body(body).header("Content-Type", "application/json").send().await }
        })
    ).await;
  let mut best: Option<(u64, Bytes)> = None;
    for res in responses.into_iter().flatten() {
        if res.status().is_success() {
            if let Ok(bytes) = res.bytes().await {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    let slot = json["result"]["context"]["slot"].as_u64().unwrap_or(0);
                    if slot == 0 {
                        return Some(bytes);
                    }
                    match &best {
                        None => best = Some((slot, bytes)),
                        Some((best_slot, _)) if slot > *best_slot => best = Some((slot, bytes)),
                        _ => {}
                    }
                }
            }
        }
    }
    best.map(|(_, bytes)| bytes)
}

pub async fn retry(client: &Client, providers: &ProviderMap, method: &str, body: Bytes) -> Option<(Bytes, String)> {
    for url in router::route(providers, method) {
        if let Ok(res) = client.post(&url).body(body.clone()).header("Content-Type", "application/json").send().await {
            if res.status().is_success() {
                if let Ok(bytes) = res.bytes().await {
                  let name = providers.iter()
                        .find(|p| p.url == url)
                        .map(|p| p.name.clone())
                        .unwrap_or(url.clone());
                    println!("[{}] success via {}", method, name);
                    return Some((bytes, name));
                }
            }
        }
        println!("[{}] failed, trying next...", method);
    }
    None
}

pub async fn stats(State(s): State<AppState>) -> Json<serde_json::Value> {
    let providers: Vec<serde_json::Value> = s.providers.iter().map(|p| {
        serde_json::json!({
            "name": p.name,
            "healthy": p.healthy,
            "latency": p.average_latency,
            "errors": p.error_count,
            "cooloff": p.cool_off,
            "weight": p.weight,
            "score": p.score(),
        })
    }).collect();
    Json(serde_json::json!({ "providers": providers }))
}