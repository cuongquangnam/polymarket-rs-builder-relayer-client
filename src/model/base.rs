use crate::utils::prepend_zx;
use ethers::core::utils::keccak256;

pub trait BaseEIP712Model {
    fn signable_bytes(&self, domain: &[u8]) -> Vec<u8>;
    
    fn generate_struct_hash(&self, domain: &[u8]) -> String {
        let bytes = self.signable_bytes(domain);
        let hash = keccak256(&bytes);
        prepend_zx(&hex::encode(hash))
    }
}

