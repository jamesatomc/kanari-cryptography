use crate::blockchain::{Block, Blockchain, Transaction, SignedTransaction};
use crate::move_runtime::MoveRuntime;
use crate::state::StateManager;
use crate::gas::{GasMeter, GasOperation};
use crate::changeset::ChangeSet;
use anyhow::{Context, Result};
use move_core_types::{account_address::AccountAddress, language_storage::ModuleId};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use kanari_types::address::Address as KanariAddress;

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

    /// Add signed transaction to pending pool after verifying signature
    pub fn submit_transaction(&self, signed_tx: SignedTransaction) -> Result<Vec<u8>> {
        // Verify signature before accepting transaction
        if !signed_tx.verify_signature()? {
            anyhow::bail!("Invalid transaction signature");
        }
        
        let tx_hash = signed_tx.hash();
        let mut pending = self.pending_txs.write().unwrap();
        pending.push(signed_tx.transaction);
        Ok(tx_hash)
    }

    /// Execute a single transaction and return ChangeSet
    /// This is the correct way: Move VM produces ChangeSet, StateManager applies it
    fn execute_transaction(&self, tx: &Transaction) -> Result<ChangeSet> {
        // 1. Pre-flight validation: Check sequence number
        let sender_addr = AccountAddress::from_hex_literal(tx.sender_address())?;
        {
            let state = self.state.read().unwrap();
            state.validate_sequence(&sender_addr, tx.sequence_number())
                .context("Sequence number validation failed")?;
        }

        // 2. Calculate gas and check balance
        let mut gas_meter = GasMeter::new(tx.gas_limit(), tx.gas_price());
        let mut changeset = ChangeSet::new();

        match tx {
            Transaction::PublishModule {
                sender,
                module_bytes,
                module_name: _,
                ..
            } => {
                // Calculate gas for publishing
                let gas_op = GasOperation::PublishModule {
                    module_size: module_bytes.len(),
                };
                gas_meter.consume(gas_op.gas_units())?;

                let addr = AccountAddress::from_hex_literal(sender)?;
                
                // Check if sender has enough balance for gas
                let gas_cost = gas_meter.total_cost();
                {
                    let state = self.state.read().unwrap();
                    let balance = state.get_account(&addr)
                        .map(|acc| acc.balance)
                        .unwrap_or(0);
                    if balance < gas_cost {
                        changeset.mark_failed(format!("Insufficient balance for gas: need {}, have {}", gas_cost, balance));
                        
                        // CRITICAL: Even pre-flight failures must deduct gas and increment sequence
                        let sender_change = changeset.get_or_create_change(addr);
                        sender_change.increment_sequence(); // Prevent replay
                        sender_change.debit(gas_cost);
                        
                        let dao_addr = AccountAddress::from_hex_literal(KanariAddress::DAO_ADDRESS)?;
                        changeset.collect_gas(dao_addr, gas_cost);
                        changeset.set_gas_used(gas_meter.gas_used);
                        return Ok(changeset);
                    }
                }

                // Execute Move VM
                let mut runtime = self.move_runtime.write().unwrap();
                let move_changeset = match runtime.publish_module(module_bytes.clone(), addr) {
                    Ok(cs) => cs,
                    Err(e) => {
                        changeset.mark_failed(format!("Module publish failed: {}", e));
                        
                        // CRITICAL: Even for failed transactions, deduct gas and increment sequence
                        let sender_change = changeset.get_or_create_change(addr);
                        sender_change.increment_sequence(); // Prevent replay
                        sender_change.debit(gas_cost);
                        
                        let dao_addr = AccountAddress::from_hex_literal(KanariAddress::DAO_ADDRESS)?;
                        changeset.collect_gas(dao_addr, gas_cost);
                        changeset.set_gas_used(gas_meter.gas_used);
                        return Ok(changeset);
                    }
                };

                // Merge Move VM ChangeSet with gas/sequence changes
                changeset.merge(move_changeset);
                
                // CRITICAL: Increment sequence and deduct gas for successful transaction
                let sender_change = changeset.get_or_create_change(addr);
                sender_change.increment_sequence(); // Prevent replay attacks
                sender_change.debit(gas_cost);
                
                // Credit gas to DAO
                let dao_addr = AccountAddress::from_hex_literal(KanariAddress::DAO_ADDRESS)?;
                changeset.collect_gas(dao_addr, gas_cost);
                
                changeset.set_gas_used(gas_meter.gas_used);
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

                let sender_addr = AccountAddress::from_hex_literal(sender)?;
                let gas_cost = gas_meter.total_cost();
                
                // Check balance
                {
                    let state = self.state.read().unwrap();
                    let balance = state.get_account(&sender_addr)
                        .map(|acc| acc.balance)
                        .unwrap_or(0);
                    if balance < gas_cost {
                        changeset.mark_failed(format!("Insufficient balance for gas: need {}, have {}", gas_cost, balance));
                        
                        // CRITICAL: Even pre-flight failures must deduct gas and increment sequence
                        let sender_change = changeset.get_or_create_change(sender_addr);
                        sender_change.increment_sequence(); // Prevent replay
                        sender_change.debit(gas_cost);
                        
                        let dao_addr = AccountAddress::from_hex_literal(KanariAddress::DAO_ADDRESS)?;
                        changeset.collect_gas(dao_addr, gas_cost);
                        changeset.set_gas_used(gas_meter.gas_used);
                        return Ok(changeset);
                    }
                }

                // Parse module ID
                let parts: Vec<&str> = module.split("::").collect();
                if parts.len() != 2 {
                    changeset.mark_failed("Invalid module format. Expected: address::module".to_string());
                    changeset.set_gas_used(0);
                    return Ok(changeset);
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

                // Execute Move VM
                let mut runtime = self.move_runtime.write().unwrap();
                let move_changeset = match runtime.execute_entry_function(&module_id, function, type_tags, args.clone()) {
                    Ok(cs) => cs,
                    Err(e) => {
                        changeset.mark_failed(format!("Function execution failed: {}", e));
                        
                        // CRITICAL: Even for failed transactions, deduct gas and increment sequence
                        let sender_change = changeset.get_or_create_change(sender_addr);
                        sender_change.increment_sequence(); // Prevent replay
                        sender_change.debit(gas_cost);
                        
                        let dao_addr = AccountAddress::from_hex_literal(KanariAddress::DAO_ADDRESS)?;
                        changeset.collect_gas(dao_addr, gas_cost);
                        changeset.set_gas_used(gas_meter.gas_used);
                        return Ok(changeset);
                    }
                };

                // Merge Move VM ChangeSet with gas/sequence changes
                changeset.merge(move_changeset);

                // Build ChangeSet: increment sequence
                let sender_change = changeset.get_or_create_change(sender_addr);
                sender_change.increment_sequence();
                sender_change.debit(gas_cost);

                // Credit gas to DAO
                let dao_addr = AccountAddress::from_hex_literal(KanariAddress::DAO_ADDRESS)?;
                changeset.collect_gas(dao_addr, gas_cost);
                
                changeset.set_gas_used(gas_meter.gas_used);
            }

            Transaction::Transfer { from, to, amount, .. } => {
                // Calculate gas for transfer
                let gas_op = GasOperation::Transfer;
                gas_meter.consume(gas_op.gas_units())?;

                let from_addr = AccountAddress::from_hex_literal(from)?;
                let to_addr = AccountAddress::from_hex_literal(to)?;
                let gas_cost = gas_meter.total_cost();
                let total_required = amount.saturating_add(gas_cost);

                // Check balance
                {
                    let state = self.state.read().unwrap();
                    let balance = state.get_account(&from_addr)
                        .map(|acc| acc.balance)
                        .unwrap_or(0);
                    if balance < total_required {
                        changeset.mark_failed(format!(
                            "Insufficient balance: need {} (amount: {}, gas: {}) but have {}",
                            total_required, amount, gas_cost, balance
                        ));
                        
                        // CRITICAL: Even if balance check fails, deduct gas and increment sequence
                        let sender_change = changeset.get_or_create_change(from_addr);
                        sender_change.increment_sequence(); // Prevent replay
                        sender_change.debit(gas_cost); // User still pays for attempt
                        
                        let dao_addr = AccountAddress::from_hex_literal(KanariAddress::DAO_ADDRESS)?;
                        changeset.collect_gas(dao_addr, gas_cost);
                        changeset.set_gas_used(gas_meter.gas_used);
                        return Ok(changeset);
                    }
                }

                // Build ChangeSet: transfer
                changeset.transfer(from_addr, to_addr, *amount);
                
                // CRITICAL: Increment sequence and deduct gas for successful transfer
                let sender_change = changeset.get_or_create_change(from_addr);
                sender_change.increment_sequence(); // Prevent replay attacks
                sender_change.debit(gas_cost);

                // Credit gas to DAO
                let dao_addr = AccountAddress::from_hex_literal(KanariAddress::DAO_ADDRESS)?;
                changeset.collect_gas(dao_addr, gas_cost);
                
                changeset.set_gas_used(gas_meter.gas_used);
            }
        }

        Ok(changeset)
    }

    /// Mine/produce a new block with pending transactions
    /// Now uses ChangeSet pattern: execute -> collect ChangeSets -> apply atomically
    /// 
    /// CRITICAL: ALL ChangeSets (both successful and failed) are applied to state.
    /// Failed transactions still deduct gas and increment sequence to prevent spam and replay attacks.
    pub fn produce_block(&self) -> Result<BlockInfo> {
        let mut pending = self.pending_txs.write().unwrap();
        
        if pending.is_empty() {
            anyhow::bail!("No pending transactions");
        }

        let transactions = pending.drain(..).collect::<Vec<_>>();
        let tx_count = transactions.len();

        // Execute all transactions and collect ALL ChangeSets (success + failed)
        let mut all_changesets = Vec::new();
        let mut executed = 0;
        let mut failed = 0;
        let mut _total_gas_used = 0u64;
        
        for tx in &transactions {
            match self.execute_transaction(tx) {
                Ok(changeset) => {
                    if changeset.success {
                        executed += 1;
                    } else {
                        eprintln!("Transaction failed: {:?}", changeset.error_message);
                        failed += 1;
                    }
                    // CRITICAL: Collect ALL ChangeSets regardless of success status
                    // Failed transactions contain gas deduction and sequence increment
                    _total_gas_used += changeset.gas_used;
                    all_changesets.push(changeset);
                }
                Err(e) => {
                    eprintln!("Transaction execution error: {:?}", e);
                    failed += 1;
                    // No ChangeSet to apply if execute_transaction failed before creating one
                }
            }
        }

        // Apply ALL ChangeSets atomically (both successful and failed)
        {
            let mut state = self.state.write().unwrap();
            for changeset in &all_changesets {
                state.apply_changeset(changeset)
                    .context("Failed to apply changeset to state")?;
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
        state.get_account_by_hex(address).map(|acc| AccountInfo {
            address: format!("{:#x}", acc.address),
            balance: acc.balance,
            sequence_number: acc.sequence_number,
            modules: acc.modules.iter().cloned().collect(),
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
        use kanari_crypto::keys::{generate_keypair, CurveType};
        
        let engine = BlockchainEngine::new().unwrap();
        
        // Generate keypair and use its address as sender
        let keypair = generate_keypair(CurveType::Ed25519).unwrap();
        
        let tx = Transaction::new_transfer(
            keypair.address.clone(),
            "0x2".to_string(),
            1000,
        );

        // Sign transaction with matching keypair
        let mut signed_tx = SignedTransaction::new(tx);
        signed_tx.sign(&keypair.private_key, CurveType::Ed25519).unwrap();

        engine.submit_transaction(signed_tx).unwrap();
        let stats = engine.get_stats();
        assert_eq!(stats.pending_transactions, 1);
    }
}
