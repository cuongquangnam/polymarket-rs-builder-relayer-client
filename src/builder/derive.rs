use crate::constants::SAFE_INIT_CODE_HASH;
use ethabi::Token;
use ethers::core::utils::keccak256;
use ethers::types::Address;
use std::str::FromStr;

pub fn get_create2_address(
    bytecode_hash: &str,
    from_address: &Address,
    salt: &[u8; 32],
) -> Address {
    let mut bytecode_hash = bytecode_hash.to_string();
    if bytecode_hash.starts_with("0x") {
        bytecode_hash = bytecode_hash[2..].to_string();
    }

    let bytecode_hash_bytes = hex::decode(&bytecode_hash).expect("Invalid bytecode hash");

    let from_bytes = from_address.as_bytes();

    let mut input = Vec::with_capacity(1 + 20 + 32 + bytecode_hash_bytes.len());
    input.push(0xff);
    input.extend_from_slice(from_bytes);
    input.extend_from_slice(salt);
    input.extend_from_slice(&bytecode_hash_bytes);

    let hash = keccak256(&input);
    Address::from_slice(&hash[12..32])
}

pub fn derive(address: &Address, safe_factory: &Address) -> Address {
    // Encode address as ABI parameter
    let encoded = ethabi::encode(&[Token::Address(*address)]);

    // Salt is keccak256 of encoded address
    let salt = keccak256(&encoded);
    let salt_array: [u8; 32] = salt.into();

    get_create2_address(SAFE_INIT_CODE_HASH, safe_factory, &salt_array)
}
