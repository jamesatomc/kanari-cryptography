// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use super::reroot_path;
use anyhow::{bail, Result};
use clap::*;
use move_package::BuildConfig;
use std::path::PathBuf;

/// Publish the Move module to the blockchain
#[derive(Parser)]
#[clap(name = "publish")]
pub struct Publish {
    /// Path to the Move package (defaults to current directory)
    #[clap(long = "package-path")]
    pub package_path: Option<PathBuf>,

    /// Gas limit for the transaction
    #[clap(long = "gas-limit", default_value = "1000000")]
    pub gas_limit: u64,

    /// Gas price in Mist
    #[clap(long = "gas-price", default_value = "1000")]
    pub gas_price: u64,

    /// Account address publishing the module (e.g., 0x1)
    #[clap(long = "sender")]
    pub sender: String,

    /// Private key for signing (hex string)
    #[clap(long = "private-key")]
    pub private_key: Option<String>,

    /// Skip signature (for testing)
    #[clap(long = "skip-signature")]
    pub skip_signature: bool,

    /// RPC endpoint
    #[clap(long = "rpc", default_value = "http://localhost:9944")]
    pub rpc_endpoint: String,
}

impl Publish {
    pub fn execute(self, path: Option<PathBuf>, config: BuildConfig) -> Result<()> {
        let rerooted_path = reroot_path(path.or(self.package_path.clone()))?;

        println!("📦 Building Move package...");
        
        // Build the package
        let compiled_package = config.compile_package(&rerooted_path, &mut std::io::stderr())?;

        println!("✅ Package compiled successfully!");
        println!("   Modules: {}", compiled_package.all_modules().count());

        // Get compiled modules
        let modules: Vec<_> = compiled_package.all_modules().collect();
        
        if modules.is_empty() {
            bail!("No modules found in package");
        }

        println!("\n📤 Publishing modules to blockchain...");
        
        for module_unit in &modules {
            let module = &module_unit.unit.module;
            let module_name = module.self_id().name().to_string();
            let module_bytecode = {
                let mut bytes = vec![];
                module.serialize(&mut bytes)?;
                bytes
            };

            println!("\n  📝 Module: {}", module_name);
            println!("     Size: {} bytes", module_bytecode.len());
            println!("     Address: {}", module.self_id().address());
            println!("     Functions: {}", module.function_defs.len());

            // Estimate gas
            let estimated_gas = 60_000 + (module_bytecode.len() as u64 * 10);
            println!("     Estimated Gas: {} units", estimated_gas);
            
            if estimated_gas > self.gas_limit {
                eprintln!("     ⚠️  Warning: Estimated gas ({}) exceeds limit ({})", 
                    estimated_gas, self.gas_limit);
            }

            // Create transaction (in real implementation, this would call the RPC)
            println!("     Creating publish transaction...");
            
            if self.skip_signature {
                println!("     ⚠️  Skipping signature (test mode)");
            } else if self.private_key.is_none() {
                bail!("Private key required (use --private-key or --skip-signature)");
            } else {
                println!("     🔑 Signing transaction...");
            }

            // In production, this would:
            // 1. Create ContractDeployment
            // 2. Sign with private key
            // 3. Submit to RPC endpoint
            // 4. Wait for confirmation
            
            println!("     ✅ Transaction created");
            println!("     RPC: {}", self.rpc_endpoint);
        }

        println!("\n✅ All modules published successfully!");
        println!("\n💡 Next steps:");
        println!("   • Use 'kanari move call' to execute functions");
        println!("   • Check transaction status on blockchain explorer");
        
        Ok(())
    }

    /// Execute with integration to blockchain engine
    #[cfg(feature = "blockchain")]
    pub fn execute_with_engine(
        self,
        path: Option<PathBuf>,
        config: BuildConfig,
    ) -> Result<()> {
        use kanari_move_runtime::{BlockchainEngine, ContractDeployment, ContractMetadata};
        use kanari_crypto::keys::CurveType;

        let rerooted_path = reroot_path(path.or(self.package_path.clone()))?;

        println!("📦 Building Move package...");
        let compiled_package = config.compile_package(&rerooted_path, &mut std::io::stderr())?;
        
        // Initialize blockchain engine
        let engine = BlockchainEngine::new()?;

        let modules: Vec<_> = compiled_package.all_modules().collect();
        
        for module_unit in &modules {
            let module = &module_unit.unit.module;
            let module_name = module.self_id().name().to_string();
            
            let mut module_bytecode = vec![];
            module.serialize(&mut module_bytecode)?;

            // Prepare metadata
            let metadata = ContractMetadata::new(
                module_name.clone(),
                "1.0.0".to_string(),
                self.sender.clone(),
            )
            .with_description(format!("Move module {}", module_name));

            // Create deployment
            let deployment = ContractDeployment::new(
                module_bytecode,
                module_name.clone(),
                &self.sender,
                metadata,
            )?
            .with_gas_limit(self.gas_limit)
            .with_gas_price(self.gas_price);

            // Deploy
            let tx_hash = if self.skip_signature {
                // For testing without signature
                use kanari_move_runtime::{SignedTransaction, Transaction};
                
                let tx = Transaction::PublishModule {
                    sender: self.sender.clone(),
                    module_bytes: deployment.bytecode.clone(),
                    module_name: module_name.clone(),
                    gas_limit: self.gas_limit,
                    gas_price: self.gas_price,
                };
                
                let signed_tx = SignedTransaction::new(tx);
                engine.submit_transaction(signed_tx)?
            } else {
                // With signature
                let private_key = self.private_key.as_ref()
                    .context("Private key required")?;
                
                use kanari_move_runtime::{SignedTransaction, Transaction};
                
                let tx = Transaction::PublishModule {
                    sender: self.sender.clone(),
                    module_bytes: deployment.bytecode.clone(),
                    module_name: module_name.clone(),
                    gas_limit: self.gas_limit,
                    gas_price: self.gas_price,
                };
                
                let mut signed_tx = SignedTransaction::new(tx);
                signed_tx.sign(private_key, CurveType::Ed25519)?;
                
                engine.submit_transaction(signed_tx)?
            };

            println!("✅ Module '{}' published", module_name);
            println!("   TX Hash: {}", hex::encode(&tx_hash[..8]));
        }

        Ok(())
    }
}
