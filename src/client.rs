use crate::builder::{
    build_safe_create_transaction_request, build_safe_transaction_request, derive,
};
use crate::config::{ContractConfig, get_contract_config};
use crate::constants::ZERO_ADDRESS;
use crate::endpoints::{
    GET_DEPLOYED, GET_NONCE, GET_TRANSACTION, GET_TRANSACTIONS, SUBMIT_TRANSACTION,
};
use crate::errors::{RelayerApiException, RelayerClientException};
use crate::http_helpers::RequestData;
use crate::http_helpers::{get, post};
use crate::models::{
    SafeCreateTransactionArgs, SafeTransaction, SafeTransactionArgs, TransactionType,
};
use crate::response::ClientRelayerTransactionResponse;
use alloy::signers::Signer;
use alloy::signers::local::PrivateKeySigner;
use ethers::types::Address;
use polymarket_client_sdk::auth::{Kind, builder::Builder};
use reqwest::header::HeaderMap;
use reqwest::{Body, Method, Request};
use serde_json::Value;
use std::str::FromStr;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;
use url::Url;

use crate::conversion::ToEthers;
pub struct RelayClient {
    relayer_url: String,
    chain_id: u64,
    contract_config: ContractConfig,
    signer: Option<PrivateKeySigner>,
    builder_config: Option<Builder>,
}

impl RelayClient {
    pub fn new(
        relayer_url: &str,
        chain_id: u64,
        private_key: Option<&str>,
        builder_config: Option<Builder>,
    ) -> Result<Self, RelayerClientException> {
        let relayer_url = if relayer_url.ends_with('/') {
            relayer_url[..relayer_url.len() - 1].to_string()
        } else {
            relayer_url.to_string()
        };

        let contract_config = get_contract_config(chain_id)?;

        let signer = if let Some(pk) = private_key {
            let mut signer = pk.parse::<PrivateKeySigner>().unwrap();
            signer.set_chain_id(Some(chain_id));
            Some(signer)
        } else {
            None
        };

        Ok(RelayClient {
            relayer_url,
            chain_id,
            contract_config,
            signer,
            builder_config,
        })
    }

    pub fn get_nonce(
        &self,
        signer_address: &Address,
        signer_type: &str,
    ) -> Result<Value, RelayerApiException> {
        let url = format!(
            "{}{}?address={:?}&type={}",
            self.relayer_url, GET_NONCE, signer_address, signer_type
        );
        get(&url, None, None)
    }

    pub fn get_transaction(&self, transaction_id: &str) -> Result<Value, RelayerApiException> {
        let url = format!(
            "{}{}?id={}",
            self.relayer_url, GET_TRANSACTION, transaction_id
        );
        get(&url, None, None)
    }

    pub fn get_transactions(&self) -> Result<Value, RelayerApiException> {
        let url = format!("{}{}", self.relayer_url, GET_TRANSACTIONS);
        get(&url, None, None)
    }

    pub fn get_deployed(&self, safe_address: &Address) -> Result<bool, RelayerApiException> {
        let url = format!(
            "{}{}?address={:?}",
            self.relayer_url, GET_DEPLOYED, safe_address
        );
        let response = get(&url, None, None)?;

        if let Some(deployed) = response.get("deployed") {
            if let Some(deployed_bool) = deployed.as_bool() {
                return Ok(deployed_bool);
            }
        }
        Ok(false)
    }

    pub fn execute(
        &self,
        transactions: &[SafeTransaction],
        metadata: Option<&str>,
    ) -> Result<ClientRelayerTransactionResponse, RelayerClientException> {
        self.assert_signer_needed()?;
        self.assert_builder_creds_needed()?;

        let safe_address = self.get_expected_safe()?;

        let deployed = self.get_deployed(&safe_address).map_err(|e| {
            RelayerClientException::new(format!("Failed to check deployment: {}", e))
        })?;

        if !deployed {
            return Err(RelayerClientException::new(format!(
                "expected safe {} is not deployed",
                safe_address
            )));
        }

        let from_address = self.signer.as_ref().unwrap().address();

        let nonce_payload = self
            .get_nonce(&from_address.to_ethers(), TransactionType::Safe.as_str())
            .map_err(|e| RelayerClientException::new(format!("Failed to get nonce: {}", e)))?;

        let nonce = nonce_payload
            .get("nonce")
            .and_then(|n| n.as_str())
            .ok_or_else(|| RelayerClientException::new("invalid nonce payload received"))?
            .to_string();

        let safe_args = SafeTransactionArgs {
            from_address: from_address.to_ethers(),
            nonce: nonce.clone(),
            chain_id: self.chain_id,
            transactions: transactions.to_vec(),
        };

        println!("none is {:?}", nonce);

        let txn_request = build_safe_transaction_request(
            self.signer.as_ref().unwrap(),
            &safe_args,
            &self.contract_config,
            metadata,
        )
        .map_err(|e| RelayerClientException::new(format!("Failed to build transaction: {}", e)))?;

        let resp = self._post_request(SUBMIT_TRANSACTION, &txn_request)?;

        let transaction_id = resp
            .get("transactionID")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let transaction_hash = resp
            .get("transactionHash")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(ClientRelayerTransactionResponse::new(
            transaction_id,
            transaction_hash,
            self,
        ))
    }

    pub fn deploy(&self) -> Result<ClientRelayerTransactionResponse, RelayerClientException> {
        self.assert_signer_needed()?;
        self.assert_builder_creds_needed()?;

        let safe_address = self.get_expected_safe()?;
        let deployed = self.get_deployed(&safe_address).map_err(|e| {
            RelayerClientException::new(format!("Failed to check deployment: {}", e))
        })?;

        if deployed {
            return Err(RelayerClientException::new(format!(
                "safe {} is already deployed!",
                safe_address
            )));
        }

        let from_address = self.signer.as_ref().unwrap().address();
        let zero_address = Address::from_str(ZERO_ADDRESS).unwrap();

        let args = SafeCreateTransactionArgs {
            from_address: from_address.to_ethers(),
            chain_id: self.chain_id,
            payment_token: zero_address,
            payment: "0".to_string(),
            payment_receiver: zero_address,
        };

        let txn_request = build_safe_create_transaction_request(
            self.signer.as_ref().unwrap(),
            &args,
            &self.contract_config,
        )
        .map_err(|e| {
            RelayerClientException::new(format!("Failed to build create transaction: {}", e))
        })?;

        let resp = self._post_request(SUBMIT_TRANSACTION, &txn_request)?;

        let transaction_id = resp
            .get("transactionID")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let transaction_hash = resp
            .get("transactionHash")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(ClientRelayerTransactionResponse::new(
            transaction_id,
            transaction_hash,
            self,
        ))
    }

    pub fn poll_until_state(
        &self,
        transaction_id: &str,
        states: &[&str],
        fail_state: Option<&str>,
        max_polls: Option<usize>,
        poll_frequency: Option<u64>,
    ) -> Result<Option<Value>, RelayerApiException> {
        let target_states: std::collections::HashSet<&str> = states.iter().cloned().collect();
        let poll_limit = max_polls.unwrap_or(10);
        let poll_frequency_ms = poll_frequency.unwrap_or(2000).max(1000);

        println!(
            "Waiting for transaction {} matching states: {:?}...",
            transaction_id, target_states
        );

        for _ in 0..poll_limit {
            let transactions = self.get_transaction(transaction_id)?;

            if let Some(txn_array) = transactions.as_array() {
                if let Some(txn) = txn_array.first() {
                    if let Some(txn_state) = txn.get("state").and_then(|s| s.as_str()) {
                        if target_states.contains(txn_state) {
                            return Ok(Some(txn.clone()));
                        }
                        if let Some(fail) = fail_state {
                            if txn_state == fail {
                                let txn_hash = txn
                                    .get("transactionHash")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown");
                                eprintln!(
                                    "txn {} failed onchain, transaction_hash: {}!",
                                    transaction_id, txn_hash
                                );
                                return Ok(None);
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(poll_frequency_ms));
        }

        println!(
            "Transaction {} not found or not in given states, timing out!",
            transaction_id
        );
        Ok(None)
    }

    fn _post_request(
        &self,
        request_path: &str,
        body: &crate::models::TransactionRequest,
    ) -> Result<Value, RelayerClientException> {
        let builder_headers = self._generate_builder_headers("POST", request_path, Some(body))?;
        let url = format!("{}{}", self.relayer_url, request_path);

        post(
            &url,
            Some(builder_headers),
            Some(&RequestData::TransactionRequest(body.clone())),
        )
        .map_err(|e| RelayerClientException::new(format!("API request failed: {}", e)))
    }

    fn _generate_builder_headers(
        &self,
        method: &str,
        request_path: &str,
        body: Option<&crate::models::TransactionRequest>,
    ) -> Result<HeaderMap, RelayerClientException> {
        let body_str = body.map(|b| {
            serde_json::to_string(b)
                .unwrap_or_default()
                .replace("\"", "'")
                .replace(":'", ": '")
                .replace(",'", ", '")
                .replace(":{", ": {")
        });
        let request_url = format!("{}{}", self.relayer_url, request_path);
        let mut request = Request::new(
            Method::from_str(method).unwrap(),
            Url::parse(&request_url).unwrap(),
        );
        request
            .body_mut()
            .replace(Body::from(body_str.clone().unwrap_or_default()));
        let rt = Runtime::new().unwrap();
        ///// Timestamp in seconds since [`std::time::UNIX_EPOCH`]
        let headers = rt
            .block_on(
                self.builder_config.as_ref().unwrap().extra_headers(
                    &request,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| {
                            RelayerClientException::new(format!(
                                "Failed to get duration since UNIX_EPOCH: {}",
                                e
                            ))
                        })?
                        .as_secs() as i64,
                ),
            )
            .map_err(|e| {
                RelayerClientException::new(format!("Failed to generate builder headers: {}", e))
            })?;
        Ok(headers)
    }

    pub fn get_expected_safe(&self) -> Result<Address, RelayerClientException> {
        self.assert_signer_needed()?;
        let addr = self.signer.as_ref().unwrap().address();
        Ok(derive(
            &addr.to_ethers(),
            &self.contract_config.safe_factory,
        ))
    }

    fn assert_signer_needed(&self) -> Result<(), RelayerClientException> {
        if self.signer.is_none() {
            return Err(RelayerClientException::new(
                "signer is required for this endpoint",
            ));
        }
        Ok(())
    }

    fn assert_builder_creds_needed(&self) -> Result<(), RelayerClientException> {
        if self.builder_config.is_none() {
            return Err(RelayerClientException::new(
                "builder credentials are required for this endpoint",
            ));
        }
        Ok(())
    }
}
