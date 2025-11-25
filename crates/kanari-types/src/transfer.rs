use anyhow::{Result, Context};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::ModuleId,
};
use serde::{Serialize, Deserialize};
use crate::address::Address;

/// Transfer record structure
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferRecord {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
}

impl TransferRecord {
    /// Create a new transfer record
    pub fn new(from: String, to: String, amount: u64) -> Self {
        Self {
            from,
            to,
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Create transfer record from AccountAddress
    pub fn from_addresses(from: AccountAddress, to: AccountAddress, amount: u64) -> Self {
        Self::new(format!("{}", from), format!("{}", to), amount)
    }
}

/// Transfer validation utilities
pub struct TransferValidator;

impl TransferValidator {
    /// Validate transfer with addresses directly
    pub fn validate_addresses(from: &AccountAddress, to: &AccountAddress, amount: u64) -> Result<bool> {
        Ok(amount > 0 && from != to)
    }
}

/// Transfer module constants and utilities
pub struct TransferModule;

impl TransferModule {
    pub const TRANSFER_MODULE: &'static str = "transfer";

    /// Get the module ID for system::transfer
    pub fn get_module_id() -> Result<ModuleId> {
        let address = AccountAddress::from_hex_literal(Address::KANARI_SYSTEM_ADDRESS)
            .context("Invalid system address")?;
        
        let module_name = Identifier::new(Self::TRANSFER_MODULE)
            .context("Invalid transfer module name")?;
        
        Ok(ModuleId::new(address, module_name))
    }

    /// Get function names used in transfer module
    pub fn function_names() -> TransferFunctions {
        TransferFunctions {
            is_valid_amount: "is_valid_amount",
            create_transfer: "create_transfer",
            get_amount: "get_amount",
            get_from: "get_from",
            get_to: "get_to",
        }
    }
}

/// Transfer module function names
pub struct TransferFunctions {
    pub is_valid_amount: &'static str,
    pub create_transfer: &'static str,
    pub get_amount: &'static str,
    pub get_from: &'static str,
    pub get_to: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_record_creation() {
        let record = TransferRecord::new(
            "0x1".to_string(),
            "0x2".to_string(),
            1000,
        );
        assert_eq!(record.from, "0x1");
        assert_eq!(record.to, "0x2");
        assert_eq!(record.amount, 1000);
        assert!(record.timestamp > 0);
    }

    #[test]
    fn test_transfer_validator() {
        let addr1 = AccountAddress::from_hex_literal("0x1").unwrap();
        let addr2 = AccountAddress::from_hex_literal("0x2").unwrap();
        
        // Valid transfer
        assert!(TransferValidator::validate_addresses(&addr1, &addr2, 500).unwrap());
        
        // Invalid: zero amount
        assert!(!TransferValidator::validate_addresses(&addr1, &addr2, 0).unwrap());
        
        // Invalid: same address
        assert!(!TransferValidator::validate_addresses(&addr1, &addr1, 500).unwrap());
    }

    #[test]
    fn test_get_transfer_module_id() {
        let module_id = TransferModule::get_module_id();
        assert!(module_id.is_ok());
        
        let module_id = module_id.unwrap();
        assert_eq!(module_id.name().as_str(), "transfer");
    }
}
