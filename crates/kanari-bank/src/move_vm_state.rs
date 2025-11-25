use crate::move_runtime::MoveRuntime;
use anyhow::{Context, Result};
use kanari_types::balance::BalanceRecord;
use kanari_types::kanari::KanariModule;
use kanari_types::transfer::TransferRecord;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// State manager that uses Move VM for execution
#[derive(Serialize, Deserialize)]
pub struct MoveVMState {
    /// Account balances in MIST - managed by Move VM Balance module
    accounts: HashMap<String, BalanceRecord>,
    /// Transfer history
    transfers: Vec<TransferRecord>,
    /// Total supply tracker
    total_supply: u64,
}

impl MoveVMState {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transfers: Vec::new(),
            total_supply: 0,
        }
    }

    pub fn data_file() -> PathBuf {
        // Use .kari/kanari-db in user home directory
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".kari")
            .join("kanari-db")
            .join("move_vm_data.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::data_file();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

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

    /// Convert KANARI to MIST
    pub fn kanari_to_mist(kanari: u64) -> u64 {
        KanariModule::kanari_to_mist(kanari)
    }

    /// Convert MIST to KANARI
    pub fn mist_to_kanari(mist: u64) -> u64 {
        KanariModule::mist_to_kanari(mist)
    }

    /// Format MIST amount as KANARI string
    pub fn format_balance(mist: u64) -> String {
        KanariModule::format_kanari(mist)
    }

    /// Create account with Move VM Balance
    pub fn create_account(&mut self, address: AccountAddress) -> Result<()> {
        let addr_hex = format!("{}", address);
        if self.accounts.contains_key(&addr_hex) {
            anyhow::bail!("Account already exists");
        }
        // Create zero balance using Move Balance module
        self.accounts.insert(addr_hex, BalanceRecord::zero());
        Ok(())
    }

    /// Get balance in MIST
    pub fn get_balance(&self, address: &AccountAddress) -> u64 {
        let addr_hex = format!("{}", address);
        self.accounts
            .get(&addr_hex)
            .map(|b| b.value)
            .unwrap_or(0)
    }

    /// Get balance formatted as KANARI
    pub fn get_balance_formatted(&self, address: &AccountAddress) -> String {
        let mist = self.get_balance(address);
        Self::format_balance(mist)
    }

    /// Get balance record (for Move operations)
    pub fn get_balance_record(&self, address: &AccountAddress) -> Option<&BalanceRecord> {
        let addr_hex = format!("{}", address);
        self.accounts.get(&addr_hex)
    }

    /// Set balance using Move Balance module
    pub fn set_balance(&mut self, address: AccountAddress, balance_mist: u64) -> Result<()> {
        let addr_hex = format!("{}", address);
        let balance = BalanceRecord::new(balance_mist);
        self.accounts.insert(addr_hex, balance);
        Ok(())
    }

    /// Mint coins to an account using Move Balance operations
    pub fn mint(&mut self, address: AccountAddress, amount: u64) -> Result<()> {
        let addr_hex = format!("{}", address);
        
        if let Some(balance) = self.accounts.get_mut(&addr_hex) {
            // Use Move Balance increase operation
            balance.increase(amount)?;
        } else {
            // Create new account with initial balance
            let mut balance = BalanceRecord::zero();
            balance.increase(amount)?;
            self.accounts.insert(addr_hex, balance);
        }
        
        self.total_supply += amount;
        Ok(())
    }

    /// Transfer using Move VM with full Balance module operations
    pub fn transfer(
        &mut self,
        runtime: &mut MoveRuntime,
        from: AccountAddress,
        to: AccountAddress,
        amount: u64,
    ) -> Result<()> {
        let from_hex = format!("{}", from);
        let to_hex = format!("{}", to);

        // Get sender balance
        let from_balance = self.accounts.get_mut(&from_hex)
            .ok_or_else(|| anyhow::anyhow!("Sender account not found"))?;

        // Verify sufficient balance using Move Balance module
        if !from_balance.is_sufficient(amount) {
            anyhow::bail!("Insufficient balance: has {}, need {}", from_balance.value, amount);
        }

        // Call Move function to validate transfer
        let is_valid = runtime.validate_transfer(&from, &to, amount)?;

        if !is_valid {
            anyhow::bail!("Transfer validation failed: invalid amount or addresses");
        }

        // Create transfer record using Move VM (REQUIRED)
        let transfer_bytes = runtime.create_transfer_record(&from, &to, amount).context(
            "Failed to create transfer record via Move VM - this is required for production",
        )?;

        // Verify the transfer amount from Move VM
        let move_amount = runtime
            .get_transfer_amount(transfer_bytes)
            .context("Failed to extract amount from Move transfer record")?;

        if move_amount != amount {
            anyhow::bail!(
                "Amount mismatch: expected {}, got {} from Move VM",
                amount,
                move_amount
            );
        }

        println!(
            "✓ Move VM validated transfer: {} → {} amount: {} MIST ({})",
            from, to, move_amount, Self::format_balance(move_amount)
        );

        // Perform balance operations using Move Balance module
        // Decrease sender balance
        let from_balance = self.accounts.get_mut(&from_hex).unwrap();
        from_balance.decrease(amount)
            .context("Failed to decrease sender balance")?;

        // Increase recipient balance (create account if needed)
        if !self.accounts.contains_key(&to_hex) {
            self.accounts.insert(to_hex.clone(), BalanceRecord::zero());
        }
        
        let to_balance = self.accounts.get_mut(&to_hex).unwrap();
        to_balance.increase(amount)
            .context("Failed to increase recipient balance")?;

        // Record transfer
        self.transfers
            .push(TransferRecord::from_addresses(from, to, amount));

        Ok(())
    }

    /// Get total supply
    pub fn get_total_supply(&self) -> u64 {
        self.total_supply
    }

    /// Get total supply formatted as KANARI
    pub fn get_total_supply_formatted(&self) -> String {
        Self::format_balance(self.total_supply)
    }
}
