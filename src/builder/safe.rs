use crate::builder::derive::derive;
use crate::config::ContractConfig;
use crate::constants::ZERO_ADDRESS;
use crate::conversion::ToAlloy;
use crate::encode::safe::create_safe_multisend_transaction;
use crate::model::safe_tx::SafeTx;
use crate::models::{
    OperationType, SafeTransaction, SafeTransactionArgs, SignatureParams, SplitSig,
    TransactionRequest, TransactionType,
};
use alloy::dyn_abi::Eip712Domain;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolStruct;
use alloy_signer::SignerSync;
use ethers::types::{Address, U256};
use std::str::FromStr;

pub fn aggregate_transaction(
    txns: &[SafeTransaction],
    safe_multisend: &Address,
) -> SafeTransaction {
    if txns.len() == 1 {
        return txns[0].clone();
    } else {
        create_safe_multisend_transaction(txns, safe_multisend)
    }
}

pub fn split_signature(sig_hex: &str) -> Result<SplitSig, String> {
    let sig = if sig_hex.starts_with("0x") {
        hex::decode(&sig_hex[2..])
    } else {
        hex::decode(sig_hex)
    }
    .map_err(|e| format!("Invalid hex: {}", e))?;

    if sig.len() != 65 {
        return Err(format!(
            "Invalid signature length: expected 65 bytes, got {}",
            sig.len()
        ));
    }

    let r = U256::from_big_endian(&sig[0..32]);
    let s = U256::from_big_endian(&sig[32..64]);
    let v_raw = sig[64];

    let v = if v_raw == 0 || v_raw == 1 {
        v_raw + 31
    } else if v_raw == 27 || v_raw == 28 {
        v_raw + 4
    } else {
        return Err(format!(
            "Invalid signature 'v' (expected 0,1,27,28), got {}",
            v_raw
        ));
    };

    Ok(SplitSig { r, s, v })
}

pub fn split_and_pack_sig(sig_hex: &str) -> Result<String, String> {
    let split_sig = split_signature(sig_hex)?;
    let mut packed = Vec::new();
    let mut r_bytes = [0u8; 32];
    split_sig.r.to_big_endian(&mut r_bytes);
    packed.extend_from_slice(&r_bytes);
    let mut s_bytes = [0u8; 32];
    split_sig.s.to_big_endian(&mut s_bytes);
    packed.extend_from_slice(&s_bytes);
    packed.push(split_sig.v);
    Ok(format!("0x{}", hex::encode(&packed)))
}

pub fn create_safe_signature(
    signer: &PrivateKeySigner,
    struct_hash: &str,
) -> Result<String, String> {
    let hash = if struct_hash.starts_with("0x") {
        hex::decode(&struct_hash[2..])
    } else {
        hex::decode(struct_hash)
    }
    .map_err(|e| format!("Invalid hash: {}", e))?;

    if hash.len() != 32 {
        return Err("Invalid hash length".to_string());
    }

    let hash_array: [u8; 32] = hash
        .try_into()
        .map_err(|_| "Failed to convert hash to [u8; 32]")?;

    signer
        .sign_message_sync(&hash_array)
        .map_err(|e| format!("Signing failed: {}", e))
        .map(|sig| sig.to_string())
}
pub fn create_struct_hash(
    chain_id: u64,
    safe: &Address,
    to: &Address,
    value: &str,
    data: &str,
    operation: OperationType,
    safe_tx_gas: &str,
    base_gas: &str,
    gas_price: &str,
    gas_token: &Address,
    refund_receiver: &Address,
    nonce: &str,
) -> Result<String, String> {
    let value_u256 = U256::from_dec_str(value).map_err(|e| format!("Invalid value: {}", e))?;
    let safe_tx_gas_u256 =
        U256::from_dec_str(safe_tx_gas).map_err(|e| format!("Invalid safe_tx_gas: {}", e))?;
    let base_gas_u256 =
        U256::from_dec_str(base_gas).map_err(|e| format!("Invalid base_gas: {}", e))?;
    let gas_price_u256 =
        U256::from_dec_str(gas_price).map_err(|e| format!("Invalid gas_price: {}", e))?;
    let nonce_u256 = U256::from_dec_str(nonce).map_err(|e| format!("Invalid nonce: {}", e))?;

    let data_bytes = if data.starts_with("0x") {
        hex::decode(&data[2..]).unwrap_or_default()
    } else {
        hex::decode(data).unwrap_or_default()
    };

    let safe_tx = SafeTx {
        to: to.to_alloy(),
        value: value_u256.to_alloy(),
        data: data_bytes.into(),
        operation: operation.as_u8(),
        safeTxGas: safe_tx_gas_u256.to_alloy(),
        baseGas: base_gas_u256.to_alloy(),
        gasPrice: gas_price_u256.to_alloy(),
        gasToken: gas_token.to_alloy(),
        refundReceiver: refund_receiver.to_alloy(),
        nonce: nonce_u256.to_alloy(),
    };

    // Create domain separator for EIP-712
    // This is simplified - full implementation would use proper EIP-712 domain encoding
    let domain = Eip712Domain {
        chain_id: Some(chain_id.to_alloy()),
        verifying_contract: Some(safe.to_alloy()),
        ..Eip712Domain::default()
    };

    Ok(safe_tx.eip712_signing_hash(&domain).to_string())
}
pub fn build_safe_transaction_request(
    signer: &PrivateKeySigner,
    args: &SafeTransactionArgs,
    config: &ContractConfig,
    metadata: Option<&str>,
) -> Result<TransactionRequest, String> {
    let factory = config.safe_factory;
    let multisend = config.safe_multisend;
    let transaction = aggregate_transaction(&args.transactions, &multisend);
    let safe_txn_gas = "0";
    let base_gas = "0";
    let gas_price = "0";
    let gas_token = Address::from_str(ZERO_ADDRESS).unwrap();
    let refund_receiver = Address::from_str(ZERO_ADDRESS).unwrap();
    let safe_address = derive(&args.from_address, &factory);

    let struct_hash = create_struct_hash(
        args.chain_id,
        &safe_address,
        &transaction.to,
        &transaction.value,
        &transaction.data,
        transaction.operation,
        safe_txn_gas,
        base_gas,
        gas_price,
        &gas_token,
        &refund_receiver,
        &args.nonce,
    )?;

    let sig = create_safe_signature(signer, &struct_hash)?;
    let packed_sig = split_and_pack_sig(&sig)?;

    let sig_params = SignatureParams {
        gas_price: Some(gas_price.to_string()),
        operation: Some(transaction.operation.as_u8().to_string()),
        safe_txn_gas: Some(safe_txn_gas.to_string()),
        base_gas: Some(base_gas.to_string()),
        gas_token: Some(format!("{}", gas_token.to_alloy())), // Display trait provides checksummed format
        refund_receiver: Some(format!("{}", refund_receiver.to_alloy())), // Display trait provides checksummed format
        payment_token: None,
        payment: None,
        payment_receiver: None,
    };

    Ok(TransactionRequest {
        transaction_type: TransactionType::Safe.as_str().to_string(),
        from_address: format!("{}", args.from_address.to_alloy()), // Display trait provides checksummed format
        to: format!("{}", transaction.to.to_alloy()), // Display trait provides checksummed format
        proxy: format!("{}", safe_address.to_alloy()), // Display trait provides checksummed format
        value: Some(transaction.value),
        data: transaction.data,
        nonce: Some(args.nonce.clone()),
        signature: packed_sig,
        signature_params: sig_params,
        metadata: metadata.map(|s| s.to_string()),
    })
}

#[test]
fn test_create_struct_hash() {
    let struct_hash = create_struct_hash(
        137,
        &Address::from_str("0x202056c7f3f3d24310e11d9099b6c796bcb63b53").unwrap(),
        &Address::from_str("0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761").unwrap(),
        "0",
        "0x8d80ff0a00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000132002791bca1f2de4661ed88a30c99a7a9449aa8417400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000044095ea7b30000000000000000000000004d97dcd97ec945f40cf65f87097ace5ea0476045ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff002791bca1f2de4661ed88a30c99a7a9449aa8417400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000044095ea7b30000000000000000000000004d97dcd97ec945f40cf65f87097ace5ea0476045ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000",
        OperationType::DelegateCall,
        "0",
        "0",
        "0",
        &Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
        &Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
        "461",
    ).unwrap();
    assert_eq!(
        struct_hash,
        "0xb067b60440424df66ce7491afa39f5ba1977ac0a8466cbb017280cc01304698f"
    );
}
