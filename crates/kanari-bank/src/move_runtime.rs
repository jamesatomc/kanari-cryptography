use anyhow::{Result, Context};
use move_binary_format::CompiledModule;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    resolver::{ModuleResolver, ResourceResolver, LinkageResolver},
};
use move_vm_runtime::move_vm::MoveVM;
use move_vm_types::gas::UnmeteredGasMeter;
use std::collections::HashMap;
use kanari_types::transfer::TransferModule;
use bcs;

/// Simple storage implementation for Move VM
pub struct SimpleStorage {
    modules: HashMap<ModuleId, Vec<u8>>,
}

impl SimpleStorage {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, module_id: ModuleId, module_bytes: Vec<u8>) {
        self.modules.insert(module_id, module_bytes);
    }
}

impl ModuleResolver for SimpleStorage {
    type Error = anyhow::Error;

    fn get_module(&self, module_id: &ModuleId) -> std::result::Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.modules.get(module_id).cloned())
    }
}

impl ResourceResolver for SimpleStorage {
    type Error = anyhow::Error;

    fn get_resource(
        &self,
        _address: &AccountAddress,
        _struct_tag: &move_core_types::language_storage::StructTag,
    ) -> std::result::Result<Option<Vec<u8>>, Self::Error> {
        // For now, return None (no resources stored)
        Ok(None)
    }
}

impl LinkageResolver for SimpleStorage {
    type Error = anyhow::Error;
}

/// Move VM wrapper for executing Move modules
pub struct MoveRuntime {
    vm: MoveVM,
    storage: SimpleStorage,
}

impl MoveRuntime {
    pub fn new() -> Result<Self> {
        let vm = MoveVM::new(vec![]).context("Failed to create Move VM")?;
        
        Ok(Self {
            vm,
            storage: SimpleStorage::new(),
        })
    }

    /// Load compiled Move module
    pub fn load_module(&mut self, module_bytes: Vec<u8>) -> Result<ModuleId> {
        let compiled_module = CompiledModule::deserialize_with_defaults(&module_bytes)
            .context("Failed to deserialize module")?;
        
        let module_id = compiled_module.self_id();
        self.storage.add_module(module_id.clone(), module_bytes);
        
        Ok(module_id)
    }

    /// Execute a Move function
    pub fn execute_function(
        &mut self,
        _sender: AccountAddress,
        module_id: &ModuleId,
        function_name: &str,
        _ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>> {
        // Create a new session with our storage
        let mut session = self.vm.new_session(&self.storage);
        
        let function_name = Identifier::new(function_name)
            .context("Invalid function name")?;

        // Execute the function
        let return_values = session
            .execute_function_bypass_visibility(
                module_id,
                &function_name,
                vec![], // Type args conversion is complex, use empty for now
                args,
                &mut UnmeteredGasMeter,
            )
            .context("Failed to execute function")?;

        // Extract just the byte vectors from the return values
        let results = return_values.return_values
            .into_iter()
            .map(|(bytes, _layout)| bytes)
            .collect();

        Ok(results)
    }

    /// Validate transfer using Move VM by calling Move function
    pub fn validate_transfer(&mut self, from: &AccountAddress, to: &AccountAddress, amount: u64) -> Result<bool> {
        // Try to call Move function if module is loaded
        let module_id = TransferModule::get_module_id()?;
        
        // Check if module is loaded
        if self.storage.modules.contains_key(&module_id) {
            // Serialize arguments
            let args = vec![
                bcs::to_bytes(&amount)?,
            ];

            // Call is_valid_amount function
            match self.execute_function(
                AccountAddress::ZERO,
                &module_id,
                "is_valid_amount",
                vec![],
                args,
            ) {
                Ok(results) => {
                    if !results.is_empty() {
                        // Deserialize bool result
                        let is_valid: bool = bcs::from_bytes(&results[0])?;
                        return Ok(is_valid && from != to);
                    }
                }
                Err(_) => {
                    // Fallback to simple validation if Move call fails
                }
            }
        }
        
        // Fallback: Simple validation if module not loaded
        Ok(amount > 0 && from != to)
    }

    /// Create a transfer record using Move VM
    pub fn create_transfer_record(&mut self, from: &AccountAddress, to: &AccountAddress, amount: u64) -> Result<Vec<u8>> {
        let module_id = TransferModule::get_module_id()?;
        
        // Serialize arguments - AccountAddress is serialized directly via BCS
        let args = vec![
            bcs::to_bytes(from)?,
            bcs::to_bytes(to)?,
            bcs::to_bytes(&amount)?,
        ];

        // Call create_transfer function
        let results = self.execute_function(
            AccountAddress::ZERO,
            &module_id,
            "create_transfer",
            vec![],
            args,
        )?;

        if results.is_empty() {
            anyhow::bail!("No return value from create_transfer");
        }

        Ok(results[0].clone())
    }

    /// Get transfer amount from transfer record
    pub fn get_transfer_amount(&mut self, transfer_bytes: Vec<u8>) -> Result<u64> {
        let module_id = TransferModule::get_module_id()?;
        
        // Call get_amount function
        let results = self.execute_function(
            AccountAddress::ZERO,
            &module_id,
            "get_amount",
            vec![],
            vec![transfer_bytes],
        )?;

        if results.is_empty() {
            anyhow::bail!("No return value from get_amount");
        }

        let amount: u64 = bcs::from_bytes(&results[0])?;
        Ok(amount)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_runtime_creation() {
        let runtime = MoveRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_validate_transfer_without_module() {
        let mut runtime = MoveRuntime::new().unwrap();
        
        let addr1 = AccountAddress::from_hex_literal("0x100").unwrap();
        let addr2 = AccountAddress::from_hex_literal("0x200").unwrap();
        
        // Test with fallback validation (no module loaded)
        let result = runtime.validate_transfer(&addr1, &addr2, 500);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Test with zero amount
        let result = runtime.validate_transfer(&addr1, &addr2, 0);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        
        // Test with same address
        let result = runtime.validate_transfer(&addr1, &addr1, 500);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
