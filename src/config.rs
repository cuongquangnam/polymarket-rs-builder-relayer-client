use crate::errors::RelayerClientException;
use ethers::types::Address;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ContractConfig {
    pub safe_factory: Address,
    pub safe_multisend: Address,
}

lazy_static::lazy_static! {
    static ref CONFIG: HashMap<u64, ContractConfig> = {
        let mut map = HashMap::new();
        map.insert(
            137,
            ContractConfig {
                safe_factory: "0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b"
                    .parse()
                    .unwrap(),
                safe_multisend: "0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761"
                    .parse()
                    .unwrap(),
            },
        );
        map.insert(
            80002,
            ContractConfig {
                safe_factory: "0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b"
                    .parse()
                    .unwrap(),
                safe_multisend: "0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761"
                    .parse()
                    .unwrap(),
            },
        );
        map
    };
}

pub fn get_contract_config(chain_id: u64) -> Result<ContractConfig, RelayerClientException> {
    CONFIG
        .get(&chain_id)
        .cloned()
        .ok_or_else(|| RelayerClientException::new(format!("Invalid chainID: {}", chain_id)))
}
