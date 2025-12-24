use crate::models::{OperationType, SafeTransaction};
use ethabi::ethereum_types::U256;
use ethabi::{encode, Token};
use ethers::types::Address;

/// Manually pack data according to eth_abi.packed.encode_packed format
/// Format: [uint8, address, uint256, uint256, bytes]
/// - uint8: 1 byte (no padding)
/// - address: 20 bytes (no padding)
/// - uint256: 32 bytes (big-endian)
/// - uint256: 32 bytes (big-endian)
/// - bytes: variable length (raw bytes, no length prefix)
fn encode_packed(
    operation: u8,
    address: &Address,
    value: U256,
    data_len: U256,
    data_bytes: &[u8],
) -> Vec<u8> {
    let mut packed = Vec::new();

    // uint8: 1 byte
    packed.push(operation);

    // address: 20 bytes (from Address)
    packed.extend_from_slice(address.as_bytes());

    // uint256: 32 bytes (big-endian)
    let mut value_bytes = [0u8; 32];
    value.to_big_endian(&mut value_bytes);
    packed.extend_from_slice(&value_bytes);

    // uint256: 32 bytes (big-endian) for data length
    let mut len_bytes = [0u8; 32];
    data_len.to_big_endian(&mut len_bytes);
    packed.extend_from_slice(&len_bytes);

    // bytes: variable length (raw bytes)
    packed.extend_from_slice(data_bytes);

    packed
}

pub fn create_safe_multisend_transaction(
    txns: &[SafeTransaction],
    safe_multisend_address: &Address,
) -> SafeTransaction {
    if txns.len() == 1 {
        return txns[0].clone();
    }

    let mut encoded_txns = Vec::new();

    for tx in txns {
        let data_bytes = if tx.data.starts_with("0x") {
            hex::decode(&tx.data[2..]).unwrap_or_default()
        } else {
            hex::decode(&tx.data).unwrap_or_default()
        };

        let value = U256::from_dec_str(&tx.value).unwrap_or_default();
        let data_len = U256::from(data_bytes.len());

        // Pack: [uint8, address, uint256, uint256, bytes]
        let packed_tx = encode_packed(tx.operation.as_u8(), &tx.to, value, data_len, &data_bytes);
        encoded_txns.push(packed_tx);
    }

    let concatenated_txns: Vec<u8> = encoded_txns.into_iter().flatten().collect();
    let multisend_data = encode(&[Token::Bytes(concatenated_txns)]);

    // keccak(text="multiSend(bytes)")[:4] = 0x8d80ff0a
    let function_selector = hex::decode("8d80ff0a").unwrap();
    let mut full_data = function_selector;
    full_data.extend_from_slice(&multisend_data);

    SafeTransaction {
        to: *safe_multisend_address,
        operation: OperationType::DelegateCall,
        data: format!("0x{}", hex::encode(&full_data)),
        value: "0".to_string(),
    }
}
