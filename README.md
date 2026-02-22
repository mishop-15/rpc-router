# solana-rpc-router

A smart Solana RPC load balancer in Rust. Self-hostable alternative to Tensor's SmartRPC.

## Problem

Single RPC provider = single point of failure. If it goes down or rate limits you, your app breaks.

## How it works
```
your app → rpc-router → helius (51ms, score: 17)   ← picked
                      → quicknode (411ms, score: 205)
                      → public (101ms, score: 101)
```

Score = latency / weight. Lower score wins. Providers are pinged every 5 seconds and scores update automatically.

## Routing strategies

| Method | Strategy | Reason |
|---|---|---|
| getLatestBlockhash | broadcast | need most recent blockhash across all providers |
| sendTransaction | broadcast | faster confirmation, Solana deduplicates by signature |
| everything else | smart | best score wins |

Note: broadcast sends the same signed transaction to all providers simultaneously. Solana deduplicates by signature so it only executes once — no double spend risk.

## Setup
```bash
git clone https://github.com/mishop-15/rpc-router
cd rpc-router
cp config.toml config.toml  # update with your API keys
cargo run -- start
```

`config.toml`:
```toml
[settings]
port = 8899

[[providers]]
name = "helius"
url = "https://devnet.helius-rpc.com/?api-key=YOUR_KEY"
weight = 3

[[providers]]
name = "quicknode"
url = "https://your-quicknode-url"
weight = 2

[[providers]]
name = "solana-public"
url = "https://api.devnet.solana.com"
weight = 1
```

Weight is a trust multiplier. Higher weight = more trusted provider. A provider with weight 3 and 100ms latency scores 33, beating a weight 1 provider at 90ms scoring 90.

## CLI
```bash
cargo run -- start      # starts proxy on localhost:8899
cargo run -- benchmark  # tests all providers, shows real latency
```

## Drop-in replacement

Point any Solana app to `localhost:8899` instead of your RPC URL:
```rust
// before
let client = RpcClient::new("https://devnet.helius-rpc.com/?api-key=xxx");

// after
let client = RpcClient::new("http://localhost:8899");

// router handles everything automatically
let blockhash = client.get_latest_blockhash()?;       // broadcast
let sig = client.send_and_confirm_transaction(&tx)?;  // broadcast
let balance = client.get_balance(&pubkey)?;           // smart routing
```

## As a Rust crate
```toml
[dependencies]
solana-rpc-router = "0.1"
```
```rust
let router = RpcRouter::new().await?;

// smart routing — picks best provider by score
router.request("getBalance", json!(["ADDRESS"])).await?;

// broadcast — all providers simultaneously, highest slot wins
router.broadcast("getLatestBlockhash", json!([])).await?;

// failover — tries best provider first, moves to next on failure
router.failover("getAccountInfo", json!(["ADDRESS"])).await?;

// live provider health
router.health();
```

## Stats endpoint
```bash
curl http://localhost:8899/stats
```
```json
{
  "providers": [
    { "name": "helius", "healthy": true, "latency": 51, "score": 17, "weight": 3 },
    { "name": "quicknode", "healthy": true, "latency": 411, "score": 205, "weight": 2 },
    { "name": "solana-public", "healthy": false, "cooloff": true, "errors": 3 }
  ]
}
```

## Motivation

SmartRPC by Tensor is a great load balancer built for TypeScript on top of @solana/web3.js. There was no Rust equivalent. This project is that a Rust implementation of the same idea, built as a self-hostable HTTP proxy and a native Rust crate.

## License

MIT
