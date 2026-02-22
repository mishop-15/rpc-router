use crate::provider::ProviderMap;

pub fn select_provider(providers: &ProviderMap) -> Option<String> {
    providers
        .iter()
        .filter(|p| p.healthy && !p.cool_off)
        .min_by_key(|p| p.score())
        .map(|p| p.url.clone())
}

pub fn get_all_healthy(providers: &ProviderMap) -> Vec<String> {
    providers
        .iter()
        .filter(|p| p.healthy)
        .map(|p| p.url.clone())
        .collect()
}

pub fn get_fastest(providers: &ProviderMap) -> Option<String> {
    providers
        .iter()
        .filter(|p| p.healthy && !p.cool_off)
        .min_by_key(|p| p.average_latency)
        .map(|p| p.url.clone())
}

pub fn route(providers: &ProviderMap, method: &str) -> Vec<String> {
    match method {
        "getLatestBlockhash" | "sendTransaction" => get_all_healthy(providers), 
        "getAccountInfo" => match get_fastest(providers) {                       
            Some(url) => vec![url],
            None => vec![],
        },
        _ => match select_provider(providers) {                                  
            Some(url) => vec![url],
            None => vec![],
        },
    }
}