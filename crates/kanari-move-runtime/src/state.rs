use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use kanari_types::address::Address as KanariAddress;

/// Account state in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub balance: u64,
    pub sequence_number: u64,
    pub modules: Vec<String>,
}

impl Account {
    pub fn new(address: String, balance: u64) -> Self {
        Self {
            address,
            balance,
            sequence_number: 0,
            modules: vec![],
        }
    }

    pub fn add_module(&mut self, module_name: String) {
        if !self.modules.contains(&module_name) {
            self.modules.push(module_name);
        }
    }

    pub fn increment_sequence(&mut self) {
        self.sequence_number += 1;
    }
}

/// Global state manager for accounts and balances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateManager {
    pub accounts: HashMap<String, Account>,
    pub total_supply: u64,
}

impl StateManager {
    /// Create new state with genesis allocation
    /// Total supply: 10 billion KANARI = 10,000,000,000,000,000,000 Mist
    /// Dev address gets entire supply according to kanari.move
    pub fn new() -> Self {
        let mut accounts = HashMap::new();
        
        // Total supply in Mist (10 billion KANARI * 10^9)
        const TOTAL_SUPPLY_MIST: u64 = 10_000_000_000_000_000_000;
        
        // Initialize system accounts
        accounts.insert(
            KanariAddress::GENESIS_ADDRESS.to_string(),
            Account::new(KanariAddress::GENESIS_ADDRESS.to_string(), 0),
        );
        accounts.insert(
            KanariAddress::STD_ADDRESS.to_string(),
            Account::new(KanariAddress::STD_ADDRESS.to_string(), 0),
        );
        accounts.insert(
            KanariAddress::KANARI_SYSTEM_ADDRESS.to_string(),
            Account::new(KanariAddress::KANARI_SYSTEM_ADDRESS.to_string(), 0),
        );

        // DAO address receives all gas fees
        accounts.insert(
            KanariAddress::DAO_ADDRESS.to_string(),
            Account::new(KanariAddress::DAO_ADDRESS.to_string(), 0),
        );

        // Dev address receives entire initial supply
        accounts.insert(
            KanariAddress::DEV_ADDRESS.to_string(),
            Account::new(KanariAddress::DEV_ADDRESS.to_string(), TOTAL_SUPPLY_MIST),
        );

        Self {
            accounts,
            total_supply: TOTAL_SUPPLY_MIST,
        }
    }

    pub fn get_or_create_account(&mut self, address: String) -> &mut Account {
        self.accounts
            .entry(address.clone())
            .or_insert_with(|| Account::new(address, 0))
    }

    pub fn get_account(&self, address: &str) -> Option<&Account> {
        self.accounts.get(address)
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<()> {
        // Check sender exists and has sufficient balance
        let sender_balance = self
            .accounts
            .get(from)
            .map(|acc| acc.balance)
            .ok_or_else(|| anyhow::anyhow!("Sender account not found"))?;

        if sender_balance < amount {
            anyhow::bail!("Insufficient balance");
        }

        // Deduct from sender
        if let Some(sender) = self.accounts.get_mut(from) {
            sender.balance -= amount;
            sender.increment_sequence();
        }

        // Add to receiver
        let receiver = self.get_or_create_account(to.to_string());
        receiver.balance += amount;

        Ok(())
    }

    pub fn mint(&mut self, to: &str, amount: u64) -> Result<()> {
        let account = self.get_or_create_account(to.to_string());
        account.balance += amount;
        Ok(())
    }

    pub fn burn(&mut self, from: &str, amount: u64) -> Result<()> {
        let account = self
            .accounts
            .get_mut(from)
            .ok_or_else(|| anyhow::anyhow!("Account not found"))?;

        if account.balance < amount {
            anyhow::bail!("Insufficient balance");
        }

        account.balance -= amount;
        Ok(())
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        self.accounts
            .get(address)
            .map(|acc| acc.balance)
            .unwrap_or(0)
    }

    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    pub fn compute_state_root(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        let serialized = serde_json::to_vec(&self.accounts).unwrap();
        Sha256::digest(&serialized).to_vec()
    }

    /// Transfer gas fees to DAO address
    pub fn collect_gas(&mut self, gas_amount: u64) -> Result<()> {
        use kanari_types::address::Address as KanariAddress;
        let dao = self.get_or_create_account(KanariAddress::DAO_ADDRESS.to_string());
        dao.balance += gas_amount;
        Ok(())
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_creation() {
        let state = StateManager::new();
        assert_eq!(state.accounts.len(), 2); // 0x1 and 0x2
        assert!(state.accounts.contains_key("0x1"));
        assert!(state.accounts.contains_key("0x2"));
    }

    #[test]
    fn test_get_or_create_account() {
        let mut state = StateManager::new();
        let account = state.get_or_create_account("0x123".to_string());
        assert_eq!(account.address, "0x123");
        assert_eq!(account.balance, 0);
    }

    #[test]
    fn test_transfer() {
        let mut state = StateManager::new();
        state.mint("0x1", 1000).unwrap();
        state.transfer("0x1", "0x2", 500).unwrap();

        assert_eq!(state.get_balance("0x1"), 500);
        assert_eq!(state.get_balance("0x2"), 500);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut state = StateManager::new();
        state.mint("0x1", 100).unwrap();
        
        let result = state.transfer("0x1", "0x2", 200);
        assert!(result.is_err());
    }

    #[test]
    fn test_mint_and_burn() {
        let mut state = StateManager::new();
        state.mint("0x1", 1000).unwrap();
        assert_eq!(state.get_balance("0x1"), 1000);

        state.burn("0x1", 300).unwrap();
        assert_eq!(state.get_balance("0x1"), 700);
    }

    #[test]
    fn test_sequence_number() {
        let mut state = StateManager::new();
        state.mint("0x1", 1000).unwrap();
        state.transfer("0x1", "0x2", 100).unwrap();

        let account = state.get_account("0x1").unwrap();
        assert_eq!(account.sequence_number, 1);
    }
}
