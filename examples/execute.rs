use dotenv::dotenv;
use ethabi::{Token, encode};
use ethers::core::utils::keccak256;
use ethers::types::Address;
use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::auth::builder::{Builder, Config};
use reqwest::Client;
use rs_builder_relayer_client::{OperationType, RelayClient, SafeTransaction};
use std::env;
use std::str::FromStr;
use uuid::Uuid;

fn function_selector(signature: &str) -> Vec<u8> {
    keccak256(signature.as_bytes())[..4].to_vec()
}

fn encode_approve(spender: &Address, amount: &str) -> String {
    let selector = function_selector("approve(address,uint256)");
    let amount_u256 = ethabi::ethereum_types::U256::from_dec_str(amount).unwrap();

    let encoded_args = encode(&[Token::Address(*spender), Token::Uint(amount_u256)]);

    let mut full_data = selector;
    full_data.extend_from_slice(&encoded_args);
    format!("0x{}", hex::encode(&full_data))
}

fn create_usdc_approve_txn(token: &str, spender: &str) -> SafeTransaction {
    let token_addr = Address::from_str(token).unwrap();
    let spender_addr = Address::from_str(spender).unwrap();

    let data = encode_approve(
        &spender_addr,
        "115792089237316195423570985008687907853269984665640564039457584007913129639935",
    );
    println!("data: {}", data);

    SafeTransaction {
        to: token_addr,
        operation: OperationType::Call,
        data,
        value: "0".to_string(),
    }
}

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
        Config::Local(Credentials::new(
            Uuid::parse_str(&env::var("BUILDER_API_KEY")?).unwrap(),
            env::var("BUILDER_SECRET").unwrap(),
            env::var("BUILDER_PASS_PHRASE").unwrap(),
        )),
        Client::new(),
    );

    let client = RelayClient::new(&relayer_url, chain_id, Some(&pk), Some(builder_config))?;

    let usdc = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
    let ctf = "0x4d97dcd97ec945f40cf65f87097ace5ea0476045";
    let txn = create_usdc_approve_txn(usdc, ctf);

    let resp = client.execute(&[txn.clone(), txn], Some("approve USDC on CTF"))?;
    println!("Execute response: {:?}", resp);

    let awaited_txn = resp.wait()?;
    println!("Awaited transaction: {:?}", awaited_txn);

    Ok(())
}
