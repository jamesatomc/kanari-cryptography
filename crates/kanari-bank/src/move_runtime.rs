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
use std::path::PathBuf;

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

    /// Load all compiled modules from a directory
    pub fn load_modules_from_dir(&mut self, dir: PathBuf) -> Result<Vec<ModuleId>> {
        let mut module_ids = Vec::new();
        
        if !dir.exists() {
            anyhow::bail!("Directory does not exist: {:?}", dir);
        }

        // Look for .mv files (compiled Move modules)
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("mv") {
                let module_bytes = std::fs::read(&path)
                    .context(format!("Failed to read module: {:?}", path))?;
                
                let module_id = self.load_module(module_bytes)?;
                module_ids.push(module_id);
            }
        }

        Ok(module_ids)
    }

    /// Execute a Move function
    pub fn execute_function(
        &mut self,
        _sender: AccountAddress,
        module_id: &ModuleId,
        function_name: &str,
        _ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Result<()> {
        // Create a new session with our storage
        let mut session = self.vm.new_session(&self.storage);
        
        let function_name = Identifier::new(function_name)
            .context("Invalid function name")?;

        // Execute the function (simplified for now without type args)
        let _return_values = session
            .execute_function_bypass_visibility(
                module_id,
                &function_name,
                vec![], // Empty type args for now
                args,
                &mut UnmeteredGasMeter,
            )
            .context("Failed to execute function")?;

        Ok(())
    }

    /// Validate transfer using Move VM
    pub fn validate_transfer(&mut self, from: u64, to: u64, amount: u64) -> Result<bool> {
        // For now, just check if amount is valid using Move logic
        // In real implementation, this would call Move function
        
        // Simple validation: amount must be > 0
        Ok(amount > 0 && from != to)
    }

    /// Get module ID for system::simple_transfer
    pub fn get_transfer_module_id() -> Result<ModuleId> {
        let addr = AccountAddress::from_hex_literal("0x1")?;
        let name = Identifier::new("simple_transfer")?;
        Ok(ModuleId::new(addr, name))
    }
}

/// Helper to compile Move source files
pub fn compile_move_package(package_path: PathBuf) -> Result<Vec<Vec<u8>>> {
    use std::process::Command;
    
    // Check if iota move is available
    let output = Command::new("iota")
        .args(&["move", "build", "--path", package_path.to_str().unwrap()])
        .output()
        .context("Failed to execute iota move build")?;

    if !output.status.success() {
        anyhow::bail!(
            "Move compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Read compiled modules from build directory
    let build_dir = package_path.join("build");
    let mut compiled_modules = Vec::new();

    // Look for .mv files in build directory
    if build_dir.exists() {
        for entry in std::fs::read_dir(build_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Look in bytecode_modules subdirectory
                let bytecode_dir = path.join("bytecode_modules");
                if bytecode_dir.exists() {
                    for module_entry in std::fs::read_dir(bytecode_dir)? {
                        let module_entry = module_entry?;
                        let module_path = module_entry.path();
                        
                        if module_path.extension().and_then(|s| s.to_str()) == Some("mv") {
                            let module_bytes = std::fs::read(&module_path)?;
                            compiled_modules.push(module_bytes);
                        }
                    }
                }
            }
        }
    }

    if compiled_modules.is_empty() {
        anyhow::bail!("No compiled modules found");
    }

    Ok(compiled_modules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_runtime_creation() {
        let runtime = MoveRuntime::new();
        assert!(runtime.is_ok());
    }
}
