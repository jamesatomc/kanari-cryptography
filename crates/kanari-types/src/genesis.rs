use crate::address::Address;
use anyhow::{Context, Result};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
};

/// Genesis module helpers
pub struct GenesisModule;

impl GenesisModule {
    pub const GENESIS_MODULE: &'static str = "genesis";

    /// Get the module ID for `kanari_system::genesis`
    pub fn get_module_id() -> Result<ModuleId> {
        let address = AccountAddress::from_hex_literal(Address::KANARI_SYSTEM_ADDRESS)
            .context("Invalid system address")?;

        let module_name = Identifier::new(Self::GENESIS_MODULE).context("Invalid genesis module name")?;

        Ok(ModuleId::new(address, module_name))
    }

    /// Get function names used in genesis module
    pub fn function_names() -> GenesisFunctions {
        GenesisFunctions { init_genesis: "init_genesis" }
    }
}

/// Genesis module function names
pub struct GenesisFunctions {
    pub init_genesis: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_id() {
        let module_id = GenesisModule::get_module_id();
        assert!(module_id.is_ok());
    }

    #[test]
    fn test_function_names_are_accessible() {
        let fns = GenesisModule::function_names();
        assert_eq!(fns.init_genesis, "init_genesis");
    }
}
