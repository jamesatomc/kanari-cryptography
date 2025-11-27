use crate::blockchain::{Block, Blockchain, Transaction};
use crate::move_runtime::MoveRuntime;
use crate::state::StateManager;
use crate::gas::{GasMeter, GasOperation};
use anyhow::Result;
use move_core_types::{account_address::AccountAddress, language_storage::ModuleId};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Complete blockchain engine with Move VM integration
pub struct BlockchainEngine {
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub state: Arc<RwLock<StateManager>>,
    pub move_runtime: Arc<RwLock<MoveRuntime>>,
    pub pending_txs: Arc<RwLock<Vec<Transaction>>>,
}

impl BlockchainEngine {
    pub fn new() -> Result<Self> {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let state = Arc::new(RwLock::new(StateManager::new()));
        let move_runtime = Arc::new(RwLock::new(MoveRuntime::new()?));
        let pending_txs = Arc::new(RwLock::new(Vec::new()));

        Ok(Self {
            blockchain,
            state,
            move_runtime,
            pending_txs,
        })
    }

    /// Add transaction to pending pool
    pub fn submit_transaction(&self, tx: Transaction) -> Result<Vec<u8>> {
        let tx_hash = tx.hash();
        let mut pending = self.pending_txs.write().unwrap();
        pending.push(tx);
        Ok(tx_hash)
    }

    /// Execute a single transaction
    fn execute_transaction(&self, tx: &Transaction) -> Result<u64> {
        // Create gas meter
        let mut gas_meter = GasMeter::new(tx.gas_limit(), tx.gas_price());

        match tx {
            Transaction::PublishModule {
                sender,
                module_bytes,
                module_name,
                ..
            } => {
                // Calculate gas for publishing
                let gas_op = GasOperation::PublishModule {
                    module_size: module_bytes.len(),
                };
                gas_meter.consume(gas_op.gas_units())?;

                let addr = AccountAddress::from_hex_literal(sender)?;
                let mut runtime = self.move_runtime.write().unwrap();
                runtime.publish_module(module_bytes.clone(), addr)?;

                // Update state
                let mut state = self.state.write().unwrap();
                let account = state.get_or_create_account(sender.clone());
                account.add_module(module_name.clone());
                account.increment_sequence();

                // Deduct gas cost from sender
                let gas_cost = gas_meter.total_cost();
                if account.balance < gas_cost {
                    anyhow::bail!("Insufficient balance for gas");
                }
                account.balance -= gas_cost;

                // Send gas to DAO
                state.collect_gas(gas_cost)?;
            }

            Transaction::ExecuteFunction {
                sender,
                module,
                function,
                type_args,
                args,
                ..
            } => {
                // Calculate gas for function execution
                let gas_op = GasOperation::ExecuteFunction { complexity: 1 };
                gas_meter.consume(gas_op.gas_units())?;

                // Parse module ID
                let parts: Vec<&str> = module.split("::").collect();
                if parts.len() != 2 {
                    anyhow::bail!("Invalid module format. Expected: address::module");
                }

                let addr = AccountAddress::from_hex_literal(parts[0])?;
                let module_id = ModuleId::new(
                    addr,
                    move_core_types::identifier::Identifier::new(parts[1])?,
                );

                // Parse type args
                let type_tags: Vec<move_core_types::language_storage::TypeTag> = type_args
                    .iter()
                    .filter_map(|s| {
                        if s == "u64" {
                            Some(move_core_types::language_storage::TypeTag::U64)
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut runtime = self.move_runtime.write().unwrap();
                runtime.execute_entry_function(&module_id, function, type_tags, args.clone())?;

                // Update state and deduct gas
                let mut state = self.state.write().unwrap();
                let account = state.get_or_create_account(sender.clone());
                account.increment_sequence();

                let gas_cost = gas_meter.total_cost();
                if account.balance < gas_cost {
                    anyhow::bail!("Insufficient balance for gas");
                }
                account.balance -= gas_cost;

                // Send gas to DAO
                state.collect_gas(gas_cost)?;
            }

            Transaction::Transfer { from, to, amount, .. } => {
                // Calculate gas for transfer
                let gas_op = GasOperation::Transfer;
                gas_meter.consume(gas_op.gas_units())?;

                let mut state = self.state.write().unwrap();

                // Check if sender has enough for transfer + gas
                let gas_cost = gas_meter.total_cost();
                let total_required = amount.saturating_add(gas_cost);

                let sender_balance = state
                    .get_account(from)
                    .map(|acc| acc.balance)
                    .unwrap_or(0);

                if sender_balance < total_required {
                    anyhow::bail!(
                        "Insufficient balance: need {} (amount: {}, gas: {}) but have {}",
                        total_required,
                        amount,
                        gas_cost,
                        sender_balance
                    );
                }

                // Perform transfer
                state.transfer(from, to, *amount)?;

                // Deduct gas from sender
                if let Some(sender) = state.accounts.get_mut(from) {
                    sender.balance -= gas_cost;
                }

                // Send gas to DAO
                state.collect_gas(gas_cost)?;
            }
        }

        Ok(gas_meter.gas_used)
    }

    /// Mine/produce a new block with pending transactions
    pub fn produce_block(&self) -> Result<BlockInfo> {
        let mut pending = self.pending_txs.write().unwrap();
        
        if pending.is_empty() {
            anyhow::bail!("No pending transactions");
        }

        let transactions = pending.drain(..).collect::<Vec<_>>();
        let tx_count = transactions.len();

        // Execute all transactions
        let mut executed = 0;
        let mut failed = 0;
        let mut _total_gas_used = 0u64;
        for tx in &transactions {
            match self.execute_transaction(tx) {
                Ok(gas_used) => {
                    executed += 1;
                    _total_gas_used += gas_used;
                }
                Err(e) => {
                    eprintln!("Transaction execution failed: {:?}", e);
                    failed += 1;
                }
            }
        }

        // Create new block
        let mut chain = self.blockchain.write().unwrap();
        let prev_hash = chain.latest_block().hash();
        let height = chain.height() + 1;

        let block = Block::new(height, prev_hash, transactions);
        let block_hash = block.hash();
        
        chain.add_block(block)?;

        Ok(BlockInfo {
            height,
            hash: hex::encode(&block_hash),
            tx_count,
            executed,
            failed,
        })
    }

    /// Get blockchain stats
    pub fn get_stats(&self) -> BlockchainStats {
        let chain = self.blockchain.read().unwrap();
        let state = self.state.read().unwrap();
        let pending = self.pending_txs.read().unwrap();

        BlockchainStats {
            height: chain.height(),
            total_blocks: chain.blocks.len(),
            total_transactions: chain.get_transaction_count(),
            pending_transactions: pending.len(),
            total_accounts: state.account_count(),
            total_supply: state.total_supply,
        }
    }

    /// Get account info
    pub fn get_account_info(&self, address: &str) -> Option<AccountInfo> {
        let state = self.state.read().unwrap();
        state.get_account(address).map(|acc| AccountInfo {
            address: acc.address.clone(),
            balance: acc.balance,
            sequence_number: acc.sequence_number,
            modules: acc.modules.clone(),
        })
    }

    /// Get block by height
    pub fn get_block(&self, height: u64) -> Option<BlockData> {
        let chain = self.blockchain.read().unwrap();
        chain.get_block(height).map(|block| BlockData {
            height: block.header.height,
            timestamp: block.header.timestamp,
            hash: hex::encode(&block.hash()),
            prev_hash: hex::encode(&block.header.prev_hash),
            tx_count: block.transactions.len(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub height: u64,
    pub total_blocks: usize,
    pub total_transactions: usize,
    pub pending_transactions: usize,
    pub total_accounts: usize,
    pub total_supply: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u64,
    pub sequence_number: u64,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub height: u64,
    pub timestamp: u64,
    pub hash: String,
    pub prev_hash: String,
    pub tx_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub height: u64,
    pub hash: String,
    pub tx_count: usize,
    pub executed: usize,
    pub failed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = BlockchainEngine::new().unwrap();
        let stats = engine.get_stats();
        assert_eq!(stats.height, 0);
        assert_eq!(stats.total_blocks, 1);
    }

    #[test]
    fn test_submit_transaction() {
        let engine = BlockchainEngine::new().unwrap();
        let tx = Transaction::new_transfer(
            "0x1".to_string(),
            "0x2".to_string(),
            1000,
        );

        engine.submit_transaction(tx).unwrap();
        let stats = engine.get_stats();
        assert_eq!(stats.pending_transactions, 1);
    }
}
