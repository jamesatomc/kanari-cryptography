//! Kanari RPC Server
//!
//! JSON-RPC server for Kanari blockchain using Axum framework

use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use kanari_move_runtime::BlockchainEngine;
use kanari_rpc_api::*;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

/// RPC server state
#[derive(Clone)]
pub struct RpcServerState {
    pub engine: Arc<BlockchainEngine>,
}

impl RpcServerState {
    pub fn new(engine: Arc<BlockchainEngine>) -> Self {
        Self { engine }
    }
}

/// Create RPC server router
pub fn create_router(state: RpcServerState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", post(handle_rpc))
        .route("/rpc", post(handle_rpc))
        .layer(cors)
        .with_state(state)
}

/// Handle RPC request
async fn handle_rpc(
    State(state): State<RpcServerState>,
    Json(request): Json<RpcRequest>,
) -> impl IntoResponse {
    info!("RPC request: method={}, id={}", request.method, request.id);

    let response = match request.method.as_str() {
        methods::GET_ACCOUNT => handle_get_account(&state, &request).await,
        methods::GET_BALANCE => handle_get_balance(&state, &request).await,
        methods::GET_BLOCK => handle_get_block(&state, &request).await,
        methods::GET_BLOCK_HEIGHT => handle_get_block_height(&state, &request).await,
        methods::GET_STATS => handle_get_stats(&state, &request).await,
        methods::SUBMIT_TRANSACTION => handle_submit_transaction(&state, &request).await,
        _ => RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError::method_not_found(&request.method)),
            id: request.id,
        },
    };

    (StatusCode::OK, Json(response))
}

/// Handle get account request
async fn handle_get_account(state: &RpcServerState, request: &RpcRequest) -> RpcResponse {
    let address: String = match serde_json::from_value(request.params.clone()) {
        Ok(addr) => addr,
        Err(e) => {
            return RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(RpcError::invalid_params(e.to_string())),
                id: request.id,
            };
        }
    };

    match state.engine.get_account_info(&address) {
        Some(info) => {
            let account_info = AccountInfo {
                address: info.address,
                balance: info.balance,
                sequence_number: info.sequence_number,
                modules: info.modules,
            };
            RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(account_info).unwrap()),
                error: None,
                id: request.id,
            }
        }
        None => RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError::internal_error("Account not found")),
            id: request.id,
        },
    }
}

/// Handle get balance request
async fn handle_get_balance(state: &RpcServerState, request: &RpcRequest) -> RpcResponse {
    let address: String = match serde_json::from_value(request.params.clone()) {
        Ok(addr) => addr,
        Err(e) => {
            return RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(RpcError::invalid_params(e.to_string())),
                id: request.id,
            };
        }
    };

    match state.engine.get_account_info(&address) {
        Some(info) => RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!(info.balance)),
            error: None,
            id: request.id,
        },
        None => RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!(0)),
            error: None,
            id: request.id,
        },
    }
}

/// Handle get block request
async fn handle_get_block(state: &RpcServerState, request: &RpcRequest) -> RpcResponse {
    let height: u64 = match serde_json::from_value(request.params.clone()) {
        Ok(h) => h,
        Err(e) => {
            return RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(RpcError::invalid_params(e.to_string())),
                id: request.id,
            };
        }
    };

    match state.engine.get_block(height) {
        Some(block) => {
            let block_info = BlockInfo {
                height: block.height,
                timestamp: block.timestamp,
                hash: block.hash.clone(),
                prev_hash: block.prev_hash,
                tx_count: block.tx_count,
                state_root: hex::encode(&block.hash), // Use block hash as state root placeholder
            };
            RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(block_info).unwrap()),
                error: None,
                id: request.id,
            }
        }
        None => RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError::internal_error("Block not found")),
            id: request.id,
        },
    }
}

/// Handle get block height request
async fn handle_get_block_height(state: &RpcServerState, request: &RpcRequest) -> RpcResponse {
    let stats = state.engine.get_stats();
    RpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(serde_json::json!(stats.height)),
        error: None,
        id: request.id,
    }
}

/// Handle get stats request
async fn handle_get_stats(state: &RpcServerState, request: &RpcRequest) -> RpcResponse {
    let stats = state.engine.get_stats();
    let blockchain_stats = BlockchainStats {
        height: stats.height,
        total_blocks: stats.total_blocks as u64,
        total_transactions: stats.total_transactions as u64,
        pending_transactions: stats.pending_transactions,
        total_accounts: stats.total_accounts,
        total_supply: stats.total_supply,
    };
    RpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(serde_json::to_value(blockchain_stats).unwrap()),
        error: None,
        id: request.id,
    }
}

/// Handle submit transaction request
async fn handle_submit_transaction(state: &RpcServerState, request: &RpcRequest) -> RpcResponse {
    use kanari_move_runtime::SignedTransaction;
    use kanari_types::address::Address;
    use std::str::FromStr;

    let tx_data: SignedTransactionData = match serde_json::from_value(request.params.clone()) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse transaction data: {}", e);
            return RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(RpcError::invalid_params(format!(
                    "Invalid transaction data: {}",
                    e
                ))),
                id: request.id,
            };
        }
    };

    // Parse sender address
    let sender = match Address::from_str(&tx_data.sender) {
        Ok(addr) => addr,
        Err(e) => {
            error!("Invalid sender address: {}", e);
            return RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(RpcError::invalid_params(format!(
                    "Invalid sender address: {}",
                    e
                ))),
                id: request.id,
            };
        }
    };

    // Parse recipient address if present
    let recipient = if let Some(ref recipient_str) = tx_data.recipient {
        match Address::from_str(recipient_str) {
            Ok(addr) => Some(addr),
            Err(e) => {
                error!("Invalid recipient address: {}", e);
                return RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(RpcError::invalid_params(format!(
                        "Invalid recipient address: {}",
                        e
                    ))),
                    id: request.id,
                };
            }
        }
    } else {
        None
    };

    // Create Transaction based on type
    use kanari_move_runtime::Transaction;
    let transaction = if let (Some(recipient), Some(amount)) = (recipient, tx_data.amount) {
        Transaction::Transfer {
            from: sender.to_string(),
            to: recipient.to_string(),
            amount,
            gas_limit: tx_data.gas_limit,
            gas_price: tx_data.gas_price,
        }
    } else {
        error!("Invalid transaction type - only transfers supported currently");
        return RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError::invalid_params(
                "Only transfer transactions are supported",
            )),
            id: request.id,
        };
    };

    // Create SignedTransaction
    let mut signed_tx = SignedTransaction::new(transaction);

    // Set signature if present
    if let Some(sig) = tx_data.signature {
        signed_tx.signature = Some(sig);
    }

    // Submit transaction to blockchain
    match state.engine.submit_transaction(signed_tx) {
        Ok(tx_hash) => {
            let tx_hash_hex = hex::encode(&tx_hash);
            info!("Transaction submitted successfully: {}", tx_hash_hex);
            let result = serde_json::json!({
                "hash": tx_hash_hex,
                "status": "pending"
            });
            RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: request.id,
            }
        }
        Err(e) => {
            error!("Failed to submit transaction: {}", e);
            RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(RpcError::internal_error(format!(
                    "Transaction submission failed: {}",
                    e
                ))),
                id: request.id,
            }
        }
    }
}

/// Start RPC server
pub async fn start_server(engine: Arc<BlockchainEngine>, addr: &str) -> Result<()> {
    let state = RpcServerState::new(engine);
    let app = create_router(state);

    info!("Starting RPC server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
