use crate::builder::derive::derive;
use crate::config::ContractConfig;
use crate::constants::SAFE_FACTORY_NAME;
use crate::model::base::BaseEIP712Model;
use crate::model::create_proxy::CreateProxy;
use crate::models::{
    SafeCreateTransactionArgs, SignatureParams, TransactionRequest, TransactionType,
};
use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::B256;
use alloy_signer::SignerSync;
use ethers::types::{Address, U256};

fn make_domain(name: &str, verifying_contract: &Address, chain_id: u64) -> Vec<u8> {
    // Simplified domain encoding
    let mut domain = Vec::new();
    domain.extend_from_slice(name.as_bytes());
    domain.extend_from_slice(verifying_contract.as_bytes());
    domain.extend_from_slice(&chain_id.to_be_bytes());
    domain
}

pub fn create_safe_create_struct_hash(
    safe_factory: &Address,
    chain_id: u64,
    payment_token: &Address,
    payment: &str,
    payment_receiver: &Address,
) -> Result<String, String> {
    let payment_u256 =
        U256::from_dec_str(payment).map_err(|e| format!("Invalid payment: {}", e))?;

    let create_proxy = CreateProxy {
        payment_token: *payment_token,
        payment: payment_u256,
        payment_receiver: *payment_receiver,
    };

    let domain = make_domain(SAFE_FACTORY_NAME, safe_factory, chain_id);
    Ok(create_proxy.generate_struct_hash(&domain))
}

pub fn create_safe_create_signature(
    signer: &PrivateKeySigner,
    safe_factory: &Address,
    chain_id: u64,
    payment_token: &Address,
    payment: &str,
    payment_receiver: &Address,
) -> Result<String, String> {
    let struct_hash = create_safe_create_struct_hash(
        safe_factory,
        chain_id,
        payment_token,
        payment,
        payment_receiver,
    )?;

    let hash = if struct_hash.starts_with("0x") {
        hex::decode(&struct_hash[2..])
    } else {
        hex::decode(&struct_hash)
    }
    .map_err(|e| format!("Invalid hash: {}", e))?;

    if hash.len() != 32 {
        return Err("Invalid hash length".to_string());
    }

    let hash_array: [u8; 32] = hash.try_into().unwrap();
    signer
        .sign_hash_sync(&B256::from_slice(&hash_array))
        .map_err(|e| format!("Signing failed: {}", e))
        .map(|sig| sig.to_string())
}

pub fn build_safe_create_transaction_request(
    signer: &PrivateKeySigner,
    args: &SafeCreateTransactionArgs,
    config: &ContractConfig,
) -> Result<TransactionRequest, String> {
    let factory = config.safe_factory;
    let safe_address = derive(&args.from_address, &factory);

    let sig = create_safe_create_signature(
        signer,
        &factory,
        args.chain_id,
        &args.payment_token,
        &args.payment,
        &args.payment_receiver,
    )?;

    let sig_params = SignatureParams {
        gas_price: None,
        operation: None,
        safe_txn_gas: None,
        base_gas: None,
        gas_token: None,
        refund_receiver: None,
        payment_token: Some(args.payment_token.to_string()),
        payment: Some(args.payment.clone()),
        payment_receiver: Some(args.payment_receiver.to_string()),
    };

    Ok(TransactionRequest {
        transaction_type: TransactionType::SafeCreate.as_str().to_string(),
        from_address: args.from_address.to_string(),
        to: factory.to_string(),
        proxy: safe_address.to_string(),
        data: "0x".to_string(),
        signature: sig,
        signature_params: sig_params,
        value: None,
        nonce: None,
        metadata: None,
    })
}
