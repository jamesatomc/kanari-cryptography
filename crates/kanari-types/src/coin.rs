use crate::address::Address;
use anyhow::{Context, Result};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
};
use serde::{Deserialize, Serialize};

/// Coin record structure
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoinRecord {
    pub value: u64,
}

impl CoinRecord {
    /// Create a new coin
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    /// Get coin value
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Burn coin and return value
    pub fn burn(self) -> u64 {
        self.value
    }
}

/// Currency metadata structure
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurrencyMetadata {
    pub symbol: Vec<u8>,
    pub name: Vec<u8>,
    pub description: Vec<u8>,
}

impl CurrencyMetadata {
    /// Create new metadata
    pub fn new(symbol: Vec<u8>, name: Vec<u8>, description: Vec<u8>) -> Self {
        Self {
            symbol,
            name,
            description,
        }
    }

    /// Get symbol as string
    pub fn symbol_str(&self) -> Result<String> {
        String::from_utf8(self.symbol.clone()).context("Invalid UTF-8 in symbol")
    }

    /// Get name as string
    pub fn name_str(&self) -> Result<String> {
        String::from_utf8(self.name.clone()).context("Invalid UTF-8 in name")
    }
}

/// Supply record structure
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SupplyRecord {
    pub total: u64,
}

impl SupplyRecord {
    /// Create new supply
    pub fn new(total: u64) -> Self {
        Self { total }
    }

    /// Get total supply
    pub fn total(&self) -> u64 {
        self.total
    }
}

/// Coin module constants and utilities
pub struct CoinModule;

impl CoinModule {
    pub const COIN_MODULE: &'static str = "coin";

    /// Get the module ID for kanari_system::coin
    pub fn get_module_id() -> Result<ModuleId> {
        let address = AccountAddress::from_hex_literal(Address::KANARI_SYSTEM_ADDRESS)
            .context("Invalid system address")?;

        let module_name = Identifier::new(Self::COIN_MODULE).context("Invalid coin module name")?;

        Ok(ModuleId::new(address, module_name))
    }

    /// Get function names used in coin module
    pub fn function_names() -> CoinFunctions {
        CoinFunctions {
            create_currency: "create_currency",
            treasury_into_supply: "treasury_into_supply",
            increase_supply: "increase_supply",
            destroy_supply: "destroy_supply",
            create_coin: "create_coin",
            value: "value",
            burn: "burn",
        }
    }
}

/// Coin module function names
pub struct CoinFunctions {
    pub create_currency: &'static str,
    pub treasury_into_supply: &'static str,
    pub increase_supply: &'static str,
    pub destroy_supply: &'static str,
    pub create_coin: &'static str,
    pub value: &'static str,
    pub burn: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coin_creation() {
        let coin = CoinRecord::new(1000);
        assert_eq!(coin.value(), 1000);
    }

    #[test]
    fn test_coin_burn() {
        let coin = CoinRecord::new(500);
        let value = coin.burn();
        assert_eq!(value, 500);
    }

    #[test]
    fn test_metadata() {
        let metadata = CurrencyMetadata::new(
            b"KANARI".to_vec(),
            b"Kanari Coin".to_vec(),
            b"Native coin of Kanari".to_vec(),
        );
        assert_eq!(metadata.symbol_str().unwrap(), "KANARI");
        assert_eq!(metadata.name_str().unwrap(), "Kanari Coin");
    }
}
