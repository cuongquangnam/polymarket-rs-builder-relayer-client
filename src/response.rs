use crate::models::RelayerTransactionState;
use serde_json::Value;
use std::fmt::{Debug, Error, Formatter};

pub struct ClientRelayerTransactionResponse<'a> {
    pub transaction_id: Option<String>,
    pub transaction_hash: Option<String>,
    pub hash: Option<String>,
    pub client: &'a crate::client::RelayClient,
}

impl<'a> ClientRelayerTransactionResponse<'a> {
    pub fn new(
        transaction_id: Option<String>,
        transaction_hash: Option<String>,
        client: &'a crate::client::RelayClient,
    ) -> Self {
        let hash = transaction_hash.clone();
        ClientRelayerTransactionResponse {
            transaction_id,
            transaction_hash,
            hash,
            client,
        }
    }

    pub fn get_transaction(&self) -> Result<Value, crate::errors::RelayerApiException> {
        if let Some(ref id) = self.transaction_id {
            self.client.get_transaction(id)
        } else {
            Err(crate::errors::RelayerApiException::new(
                None,
                "No transaction ID".to_string(),
            ))
        }
    }

    pub fn wait(&self) -> Result<Option<Value>, crate::errors::RelayerApiException> {
        if self.transaction_id.is_none() {
            return Ok(None);
        }

        let transaction_id = self.transaction_id.as_ref().unwrap().clone();
        self.client.poll_until_state(
            &transaction_id,
            &[
                RelayerTransactionState::StateMined.as_str(),
                RelayerTransactionState::StateConfirmed.as_str(),
            ],
            Some(RelayerTransactionState::StateFailed.as_str()),
            Some(30),
            Some(2000),
        )
    }
}

impl<'a> Debug for ClientRelayerTransactionResponse<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "ClientRelayerTransactionResponse {{ transaction_id: {:?}, transaction_hash: {:?}, hash: {:?} }}", self.transaction_id, self.transaction_hash, self.hash)
    }
}
