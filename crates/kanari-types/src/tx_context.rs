use anyhow::{Result, Context};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::ModuleId,
};
use serde::{Serialize, Deserialize};

/// Transaction context structure
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxContextRecord {
    pub sender: String,
    pub epoch: u64,
    pub digest: Vec<u8>,
    pub gas_budget: u64,
    pub gas_price: u64,
}

impl TxContextRecord {
    /// Create a new transaction context
    pub fn new(
        sender: String,
        epoch: u64,
        digest: Vec<u8>,
        gas_budget: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            sender,
            epoch,
            digest,
            gas_budget,
            gas_price,
        }
    }

    /// Create from AccountAddress
    pub fn from_address(
        sender: AccountAddress,
        epoch: u64,
        digest: Vec<u8>,
        gas_budget: u64,
        gas_price: u64,
    ) -> Self {
        Self::new(
            format!("{}", sender),
            epoch,
            digest,
            gas_budget,
            gas_price,
        )
    }

    /// Get sender address
    pub fn sender(&self) -> &str {
        &self.sender
    }

    /// Get current epoch
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Calculate total gas cost
    pub fn total_gas_cost(&self) -> u64 {
        self.gas_budget.saturating_mul(self.gas_price)
    }
}

/// TxContext module constants and utilities
pub struct TxContextModule;

impl TxContextModule {
    pub const KANARI_SYSTEM_ADDRESS: &'static str = "0x2";
    pub const TX_CONTEXT_MODULE: &'static str = "tx_context";

    /// Get the module ID for kanari_system::tx_context
    pub fn get_module_id() -> Result<ModuleId> {
        let address = AccountAddress::from_hex_literal(Self::KANARI_SYSTEM_ADDRESS)
            .context("Invalid system address")?;
        
        let module_name = Identifier::new(Self::TX_CONTEXT_MODULE)
            .context("Invalid tx_context module name")?;
        
        Ok(ModuleId::new(address, module_name))
    }

    /// Get function names used in tx_context module
    pub fn function_names() -> TxContextFunctions {
        TxContextFunctions {
            sender: "sender",
            epoch: "epoch",
            digest: "digest",
        }
    }
}

/// TxContext module function names
pub struct TxContextFunctions {
    pub sender: &'static str,
    pub epoch: &'static str,
    pub digest: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_context_creation() {
        let ctx = TxContextRecord::new(
            "0x1".to_string(),
            5,
            vec![1, 2, 3, 4],
            1000,
            100,
        );
        assert_eq!(ctx.sender(), "0x1");
        assert_eq!(ctx.epoch(), 5);
        assert_eq!(ctx.gas_budget, 1000);
    }

    #[test]
    fn test_total_gas_cost() {
        let ctx = TxContextRecord::new(
            "0x1".to_string(),
            0,
            vec![],
            1000,
            50,
        );
        assert_eq!(ctx.total_gas_cost(), 50_000);
    }

    #[test]
    fn test_module_id() {
        let module_id = TxContextModule::get_module_id();
        assert!(module_id.is_ok());
    }
}