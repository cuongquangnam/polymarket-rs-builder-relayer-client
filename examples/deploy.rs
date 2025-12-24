use dotenv::dotenv;
use polymarket_client_sdk::auth::builder::{Builder, Config};
use polymarket_client_sdk::auth::Credentials;
use reqwest::Client;
use rs_builder_relayer_client::RelayClient;
use std::env;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    println!("starting...");

    let relayer_url =
        env::var("RELAYER_URL").unwrap_or_else(|_| "https://relayer-v2.polymarket.com".to_string());
    let chain_id: u64 = env::var("CHAIN_ID")
        .unwrap_or_else(|_| "137".to_string())
        .parse()?;
    let pk = env::var("PK")?;

    let builder_config = Builder::new(
        // api_key: env::var("BUILDER_API_KEY")?,
        // secret: env::var("BUILDER_SECRET")?,
        // passphrase: env::var("BUILDER_PASS_PHRASE")?,
        Config::Local(Credentials::new(
            Uuid::parse_str(&env::var("BUILDER_API_KEY")?).unwrap(),
            env::var("BUILDER_SECRET").unwrap(),
            env::var("BUILDER_PASS_PHRASE").unwrap(),
        )),
        Client::new(),
    );

    let client = RelayClient::new(&relayer_url, chain_id, Some(&pk), Some(builder_config))?;

    let resp = client.deploy()?;
    println!("Deploy response: {:?}", resp);

    let awaited_txn = resp.wait()?;
    println!("Awaited transaction: {:?}", awaited_txn);

    Ok(())
}
