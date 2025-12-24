# rs-builder-relayer-client

Rust client library for interacting with the Polymarket Relayer infrastructure

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rs-builder-relayer-client = { path = "../rs-builder-relayer-client" }
```

Or from crates.io (when published):

```toml
[dependencies]
rs-builder-relayer-client = "0.0.1"
```

## Configuration

Create a `.env` file with credentials:

```env
RELAYER_URL=https://relayer-v2-staging.polymarket.dev/
CHAIN_ID=80002
PK=your_private_key_here
BUILDER_API_KEY=your_api_key
BUILDER_SECRET=your_api_secret
BUILDER_PASS_PHRASE=your_passphrase
```

## Usage

### Deploy a Safe

```rust
use rs_builder_relayer_client::{BuilderConfig, RelayClient};
use std::env;

let relayer_url = env::var("RELAYER_URL")?;
let chain_id: u64 = env::var("CHAIN_ID")?.parse()?;
let pk = env::var("PK")?;

let builder_config = BuilderConfig {
    api_key: env::var("BUILDER_API_KEY")?,
    secret: env::var("BUILDER_SECRET")?,
    passphrase: env::var("BUILDER_PASS_PHRASE")?,
};

let client = RelayClient::new(&relayer_url, chain_id, Some(&pk), Some(builder_config))?;
let resp = client.deploy()?;
let awaited_txn = resp.wait()?;
```

### Execute Transactions

```rust
use rs_builder_relayer_client::{OperationType, SafeTransaction};

let txn = SafeTransaction {
    to: address,
    operation: OperationType::Call,
    data: "0x...".to_string(),
    value: "0".to_string(),
};

let resp = client.execute(&[txn], Some("metadata"))?;
let awaited_txn = resp.wait()?;
```

## Examples

See the `examples/` directory for complete examples:

- `deploy.rs` - Deploy a new Safe
- `execute.rs` - Execute transactions on a Safe

Run examples with:

```bash
cargo run --example deploy
cargo run --example execute
```

## Features

- Deploy Safe wallets
- Execute Safe transactions
- Poll transaction status
- Builder API authentication
- EIP-712 signing support

## License

MIT

