use anyhow::{Result, Context};
use move_core_types::account_address::AccountAddress;
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::move_runtime::MoveRuntime;

/// State manager that uses Move VM for execution
#[derive(Serialize, Deserialize)]
pub struct MoveVMState {
    /// Account balances (synced with Move VM)
    accounts: HashMap<String, u64>,
    /// Transfer history
    transfers: Vec<TransferRecord>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransferRecord {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
}

impl MoveVMState {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transfers: Vec::new(),
        }
    }

    pub fn data_file() -> PathBuf {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".kanari_bank_move_vm_data.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::data_file();
        if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            let state: MoveVMState = serde_json::from_str(&data)?;
            Ok(state)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::data_file();
        let data = serde_json::to_string_pretty(&self)?;
        std::fs::write(&path, data)?;
        Ok(())
    }

    /// Create account
    pub fn create_account(&mut self, address: AccountAddress) -> Result<()> {
        let addr_hex = format!("{}", address);
        if self.accounts.contains_key(&addr_hex) {
            anyhow::bail!("Account already exists");
        }
        self.accounts.insert(addr_hex, 0);
        Ok(())
    }

    /// Get balance
    pub fn get_balance(&self, address: &AccountAddress) -> u64 {
        let addr_hex = format!("{}", address);
        *self.accounts.get(&addr_hex).unwrap_or(&0)
    }

    /// Set balance
    pub fn set_balance(&mut self, address: AccountAddress, balance: u64) {
        let addr_hex = format!("{}", address);
        self.accounts.insert(addr_hex, balance);
    }

    /// Transfer using Move VM
    pub fn transfer(
        &mut self,
        runtime: &mut MoveRuntime,
        from: AccountAddress,
        to: AccountAddress,
        amount: u64,
    ) -> Result<()> {
        // Verify balances
        let from_balance = self.get_balance(&from);
        if from_balance < amount {
            anyhow::bail!("Insufficient balance");
        }

        // Create transfer record using Move VM
        let from_u64 = address_to_u64(&from);
        let to_u64 = address_to_u64(&to);
        
        // Call Move function to validate transfer
        let is_valid = runtime.validate_transfer(from_u64, to_u64, amount)?;
        
        if !is_valid {
            anyhow::bail!("Transfer validation failed");
        }

        // Update local state
        let to_balance = self.get_balance(&to);
        self.set_balance(from, from_balance - amount);
        self.set_balance(to, to_balance + amount);

        // Record transfer
        self.transfers.push(TransferRecord {
            from: format!("{}", from),
            to: format!("{}", to),
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(())
    }

    /// Get all accounts
    pub fn accounts(&self) -> &HashMap<String, u64> {
        &self.accounts
    }

    /// Get transfer history
    pub fn transfers(&self) -> &Vec<TransferRecord> {
        &self.transfers
    }
}

/// Convert AccountAddress to u64 for Move VM (simplified)
fn address_to_u64(addr: &AccountAddress) -> u64 {
    let bytes = addr.to_vec();
    let mut result = 0u64;
    for (i, &byte) in bytes.iter().rev().take(8).enumerate() {
        result |= (byte as u64) << (i * 8);
    }
    result
}
