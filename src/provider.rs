use crate::config::Provider;
use std::sync::Arc;
use dashmap::DashMap;

pub type ProviderMap = Arc<DashMap<String, ProviderState>>;

#[derive(Debug, Clone)]
pub struct ProviderState {
    pub name: String,
    pub url: String,
    pub weight: u64,
    pub healthy: bool,
    pub average_latency: u128,
    pub error_count: i64,
    pub cool_off: bool,
}

impl ProviderState {
    pub fn new(provider: &Provider) -> Self {
        Self { name: provider.name.clone(), url: provider.url.clone(), weight: provider.weight, healthy: true, average_latency: 0, error_count: 0, cool_off: false }
    }
     pub fn score(&self) -> u128 {
        if self.average_latency == 0 {
            return u128::MAX; 
        }
        self.average_latency / self.weight as u128
    }
}

pub fn create_provider_map(providers: &[crate::config::Provider]) -> ProviderMap {
        let map = DashMap::new();
        for provider in providers{
            map.insert(provider.name.clone(), ProviderState::new(provider));
        }
        Arc::new(map)
    }