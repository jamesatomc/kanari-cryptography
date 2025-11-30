//! Kanari RPC API Definitions
//!
//! Defines request/response types and RPC methods for Kanari blockchain

use serde::{Deserialize, Serialize};

/// RPC request wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

/// RPC response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: u64,
}

/// RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl RpcError {
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: msg.into(),
            data: None,
        }
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
            data: None,
        }
    }

    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method.into()),
            data: None,
        }
    }
}

/// Account info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u64,
    pub sequence_number: u64,
    pub modules: Vec<String>,
}

/// Block info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub height: u64,
    pub timestamp: u64,
    pub hash: String,
    pub prev_hash: String,
    pub tx_count: usize,
    pub state_root: String,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub hash: String,
    pub status: String,
    pub block_height: Option<u64>,
    pub gas_used: Option<u64>,
}

/// Blockchain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub height: u64,
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub pending_transactions: usize,
    pub total_accounts: usize,
    pub total_supply: u64,
}

/// Submit transaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub transaction: SignedTransactionData,
}

/// Signed transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransactionData {
    pub sender: String,
    pub recipient: Option<String>,
    pub amount: Option<u64>,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub sequence_number: u64,
    pub signature: Option<Vec<u8>>,
}

/// Publish module request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishModuleRequest {
    pub sender: String,
    pub module_bytes: Vec<u8>,
    pub module_name: String,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub sequence_number: u64,
    pub signature: Option<Vec<u8>>,
}

/// Call function request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFunctionRequest {
    pub sender: String,
    pub package: String,
    pub module: String,
    pub function: String,
    pub type_args: Vec<String>,
    pub args: Vec<Vec<u8>>,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub sequence_number: u64,
    pub signature: Option<Vec<u8>>,
}

/// Contract info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub address: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub functions: Vec<String>,
}

/// RPC Methods
pub mod methods {
    pub const GET_ACCOUNT: &str = "kanari_getAccount";
    pub const GET_BALANCE: &str = "kanari_getBalance";
    pub const GET_BLOCK: &str = "kanari_getBlock";
    pub const GET_BLOCK_HEIGHT: &str = "kanari_getBlockHeight";
    pub const GET_TRANSACTION: &str = "kanari_getTransaction";
    pub const SUBMIT_TRANSACTION: &str = "kanari_submitTransaction";
    pub const GET_STATS: &str = "kanari_getStats";
    pub const ESTIMATE_GAS: &str = "kanari_estimateGas";

    // Contract operations
    pub const PUBLISH_MODULE: &str = "kanari_publishModule";
    pub const CALL_FUNCTION: &str = "kanari_callFunction";
    pub const GET_CONTRACT: &str = "kanari_getContract";
    pub const LIST_CONTRACTS: &str = "kanari_listContracts";
}
