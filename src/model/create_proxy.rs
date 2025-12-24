use crate::model::base::BaseEIP712Model;
use ethers::types::{Address, U256};
use ethabi::{encode, Token};

pub struct CreateProxy {
    pub payment_token: Address,
    pub payment: U256,
    pub payment_receiver: Address,
}

impl BaseEIP712Model for CreateProxy {
    fn signable_bytes(&self, _domain: &[u8]) -> Vec<u8> {
        let tokens = vec![
            Token::Address(self.payment_token),
            Token::Uint(self.payment),
            Token::Address(self.payment_receiver),
        ];
        encode(&tokens)
    }
}

