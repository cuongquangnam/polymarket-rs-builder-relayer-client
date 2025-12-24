use crate::errors::RelayerClientException;
use crate::utils::prepend_zx;
use ethers::core::utils::keccak256;
use ethers::prelude::*;
use ethers::signers::{LocalWallet, Signer as EthersSigner};
use ethers::types::Address;
use std::str::FromStr;

pub struct Signer {
    wallet: LocalWallet,
    chain_id: u64,
}

impl Signer {
    pub fn new(private_key: &str, chain_id: u64) -> Result<Self, RelayerClientException> {
        let wallet = LocalWallet::from_str(private_key)
            .map_err(|e| RelayerClientException::new(format!("Invalid private key: {}", e)))?
            .with_chain_id(chain_id);

        Ok(Signer { wallet, chain_id })
    }

    pub fn address(&self) -> Address {
        self.wallet.address()
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub fn sign(&self, message_hash: &[u8; 32]) -> Result<String, RelayerClientException> {
        // Sign the hash directly
        let sig = self
            .wallet
            .sign_hash(H256::from_slice(message_hash))
            .map_err(|e| RelayerClientException::new(format!("Signing failed: {}", e)))?;
        Ok(prepend_zx(&sig.to_string()))
    }

    pub fn sign_eip712_struct_hash(
        &self,
        message_hash: &[u8; 32],
    ) -> Result<String, RelayerClientException> {
        // Apply EIP-191 prefix: "\x19Ethereum Signed Message:\n32" + hash
        let prefix = b"\x19Ethereum Signed Message:\n32";
        let mut prefixed = Vec::with_capacity(prefix.len() + 32);
        prefixed.extend_from_slice(prefix);
        prefixed.extend_from_slice(message_hash);

        let hash = keccak256(&prefixed);
        let sig = self
            .wallet
            .sign_hash(H256::from_slice(&hash))
            .map_err(|e| RelayerClientException::new(format!("EIP712 signing failed: {}", e)))?;
        Ok(prepend_zx(&sig.to_string()))
    }
}
