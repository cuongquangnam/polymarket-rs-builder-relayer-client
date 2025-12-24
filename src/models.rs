use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Call = 0,
    DelegateCall = 1,
}

impl OperationType {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone)]
pub struct SafeTransaction {
    pub to: Address,
    pub operation: OperationType,
    pub data: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Safe,
    SafeCreate,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Safe => "SAFE",
            TransactionType::SafeCreate => "SAFE-CREATE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureParams {
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasPrice")]
    pub gas_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "safeTxnGas")]
    pub safe_txn_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "baseGas")]
    pub base_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasToken")]
    pub gas_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "refundReceiver")]
    pub refund_receiver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "paymentToken")]
    pub payment_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "payment")]
    pub payment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "paymentReceiver")]
    pub payment_receiver: Option<String>,
}

impl SignatureParams {
    pub fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        if let Some(ref gas_price) = self.gas_price {
            map.insert("gasPrice".to_string(), Value::String(gas_price.clone()));
        }
        if let Some(ref operation) = self.operation {
            map.insert("operation".to_string(), Value::String(operation.clone()));
        }
        if let Some(ref safe_txn_gas) = self.safe_txn_gas {
            map.insert(
                "safeTxnGas".to_string(),
                Value::String(safe_txn_gas.clone()),
            );
        }
        if let Some(ref base_gas) = self.base_gas {
            map.insert("baseGas".to_string(), Value::String(base_gas.clone()));
        }
        if let Some(ref gas_token) = self.gas_token {
            map.insert("gasToken".to_string(), Value::String(gas_token.clone()));
        }
        if let Some(ref refund_receiver) = self.refund_receiver {
            map.insert(
                "refundReceiver".to_string(),
                Value::String(refund_receiver.clone()),
            );
        }
        if let Some(ref payment_token) = self.payment_token {
            map.insert(
                "paymentToken".to_string(),
                Value::String(payment_token.clone()),
            );
        }
        if let Some(ref payment) = self.payment {
            map.insert("payment".to_string(), Value::String(payment.clone()));
        }
        if let Some(ref payment_receiver) = self.payment_receiver {
            map.insert(
                "paymentReceiver".to_string(),
                Value::String(payment_receiver.clone()),
            );
        }
        Value::Object(map)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionRequest {
    #[serde(rename = "type")]
    pub transaction_type: String,
    #[serde(rename = "from")]
    pub from_address: String,
    pub to: String,
    #[serde(rename = "proxyWallet")]
    pub proxy: String,
    pub data: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(rename = "signatureParams")]
    pub signature_params: SignatureParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SafeTransactionArgs {
    pub from_address: Address,
    pub nonce: String,
    pub chain_id: u64,
    pub transactions: Vec<SafeTransaction>,
}

#[derive(Debug, Clone)]
pub struct SafeCreateTransactionArgs {
    pub from_address: Address,
    pub chain_id: u64,
    pub payment_token: Address,
    pub payment: String,
    pub payment_receiver: Address,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayerTransactionState {
    StateNew,
    StateExecuted,
    StateMined,
    StateInvalid,
    StateConfirmed,
    StateFailed,
}

impl RelayerTransactionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelayerTransactionState::StateNew => "STATE_NEW",
            RelayerTransactionState::StateExecuted => "STATE_EXECUTED",
            RelayerTransactionState::StateMined => "STATE_MINED",
            RelayerTransactionState::StateInvalid => "STATE_INVALID",
            RelayerTransactionState::StateConfirmed => "STATE_CONFIRMED",
            RelayerTransactionState::StateFailed => "STATE_FAILED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "STATE_NEW" => Some(RelayerTransactionState::StateNew),
            "STATE_EXECUTED" => Some(RelayerTransactionState::StateExecuted),
            "STATE_MINED" => Some(RelayerTransactionState::StateMined),
            "STATE_INVALID" => Some(RelayerTransactionState::StateInvalid),
            "STATE_CONFIRMED" => Some(RelayerTransactionState::StateConfirmed),
            "STATE_FAILED" => Some(RelayerTransactionState::StateFailed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SplitSig {
    pub r: U256,
    pub s: U256,
    pub v: u8,
}
