use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::move_runtime::MoveRuntime;
use kanari_types::transfer::TransferRecord;

/// State manager that uses Move VM for execution
#[derive(Serialize, Deserialize)]
pub struct MoveVMState {
    /// Account balances (synced with Move VM)
    accounts: HashMap<String, u64>,
    /// Transfer history
    transfers: Vec<TransferRecord>,
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

        // Call Move function to validate transfer
        let is_valid = runtime.validate_transfer(&from, &to, amount)?;
        
        if !is_valid {
            anyhow::bail!("Transfer validation failed");
        }

        // Try to create transfer record using Move VM
        match runtime.create_transfer_record(&from, &to, amount) {
            Ok(transfer_bytes) => {
                // Verify the transfer amount from Move VM
                if let Ok(move_amount) = runtime.get_transfer_amount(transfer_bytes) {
                    if move_amount != amount {
                        anyhow::bail!("Amount mismatch: expected {}, got {} from Move VM", amount, move_amount);
                    }
                    println!("✓ Move VM validated transfer: {} → {} amount: {}", from, to, move_amount);
                }
            }
            Err(e) => {
                println!("⚠ Move VM not available, using fallback validation: {}", e);
            }
        }

        // Update local state
        let to_balance = self.get_balance(&to);
        self.set_balance(from, from_balance - amount);
        self.set_balance(to, to_balance + amount);

        // Record transfer
        self.transfers.push(TransferRecord::from_addresses(from, to, amount));

        Ok(())
    }
}
