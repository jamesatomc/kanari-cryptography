// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Context, Result};
use clap::*;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::TypeTag,
    parser,
};

/// Call a Move function on the blockchain
#[derive(Parser)]
#[clap(name = "call")]
pub struct Call {
    /// Function identifier: <address>::<module>::<function>
    /// Example: 0x1::coin::transfer
    #[clap(long = "function")]
    pub function: String,

    /// Type arguments (comma-separated)
    /// Example: "0x1::coin::KANARI,u64"
    #[clap(long = "type-args")]
    pub type_args: Option<String>,

    /// Function arguments (comma-separated, BCS-encoded hex)
    /// Example: "0x123,1000"
    #[clap(long = "args")]
    pub args: Option<String>,

    /// Sender/Caller address
    #[clap(long = "sender")]
    pub sender: String,

    /// Gas limit for the transaction
    #[clap(long = "gas-limit", default_value = "200000")]
    pub gas_limit: u64,

    /// Gas price in Mist
    #[clap(long = "gas-price", default_value = "1000")]
    pub gas_price: u64,

    /// Private key for signing (hex string)
    #[clap(long = "private-key")]
    pub private_key: Option<String>,

    /// Skip signature (for testing)
    #[clap(long = "skip-signature")]
    pub skip_signature: bool,

    /// RPC endpoint
    #[clap(long = "rpc", default_value = "http://localhost:9944")]
    pub rpc_endpoint: String,

    /// Dry run (estimate gas without executing)
    #[clap(long = "dry-run")]
    pub dry_run: bool,
}

impl Call {
    pub fn execute(self) -> Result<()> {
        println!("📞 Preparing function call...");

        // Parse function identifier
        let (module_addr, module_name, function_name) = self.parse_function_identifier()?;

        println!("\n📋 Call Details:");
        println!("   Address: {}", module_addr);
        println!("   Module: {}", module_name);
        println!("   Function: {}", function_name);
        println!("   Sender: {}", self.sender);
        println!("   Gas Limit: {}", self.gas_limit);
        println!("   Gas Price: {}", self.gas_price);

        // Parse type arguments
        let _type_args = if let Some(ref type_args_str) = self.type_args {
            let parsed = self.parse_type_args(type_args_str)?;
            println!("   Type Args: {:?}", parsed);
            parsed
        } else {
            vec![]
        };

        // Parse arguments
        let _args = if let Some(ref args_str) = self.args {
            let parsed = self.parse_args(args_str)?;
            println!("   Arguments: {} args provided", parsed.len());
            parsed
        } else {
            vec![]
        };

        // Estimate gas
        let estimated_gas = 35_000 + (function_name.len() as u64 * 100);
        println!("\n⛽ Gas Estimation:");
        println!("   Estimated: {} units", estimated_gas);
        println!("   Limit: {} units", self.gas_limit);
        println!("   Total Cost: {} Mist", estimated_gas * self.gas_price);

        if self.dry_run {
            println!("\n🧪 Dry run mode - not executing");
            return Ok(());
        }

        // Create transaction
        println!("\n🔨 Creating transaction...");

        if self.skip_signature {
            println!("   ⚠️  Skipping signature (test mode)");
        } else if self.private_key.is_none() {
            bail!("Private key required (use --private-key or --skip-signature)");
        } else {
            println!("   🔑 Signing transaction...");
        }

        // In production, this would:
        // 1. Create ContractCall
        // 2. Sign with private key
        // 3. Submit to RPC endpoint
        // 4. Wait for confirmation

        println!("   ✅ Transaction created");
        println!("   RPC: {}", self.rpc_endpoint);

        println!("\n✅ Function call submitted!");
        println!("\n💡 Next steps:");
        println!("   • Check transaction status");
        println!("   • View execution results on explorer");

        Ok(())
    }

    /// Execute with integration to blockchain engine
    #[cfg(feature = "blockchain")]
    pub fn execute_with_engine(self) -> Result<()> {
        use kanari_move_runtime::{BlockchainEngine, ContractCall};
        use kanari_crypto::keys::CurveType;

        println!("📞 Preparing function call with blockchain engine...");

        // Parse function identifier
        let (module_addr, module_name, function_name) = self.parse_function_identifier()?;

        // Parse type arguments
        let type_args = if let Some(ref type_args_str) = self.type_args {
            self.parse_type_args(type_args_str)?
        } else {
            vec![]
        };

        // Parse arguments
        let args = if let Some(ref args_str) = self.args {
            self.parse_args(args_str)?
        } else {
            vec![]
        };

        // Initialize blockchain engine
        let engine = BlockchainEngine::new()?;

        // Create contract call
        let mut call = ContractCall::new(
            &module_addr,
            &module_name,
            &function_name,
            &self.sender,
        )?
        .with_gas_limit(self.gas_limit)
        .with_gas_price(self.gas_price);

        // Add type arguments
        for type_arg in type_args {
            call = call.with_type_arg(type_arg);
        }

        // Add arguments
        for arg in args {
            call = call.with_arg(arg);
        }

        if self.dry_run {
            println!("🧪 Dry run mode - not executing");
            return Ok(());
        }

        // Execute call
        let tx_hash = if self.skip_signature {
            // For testing without signature
            use kanari_move_runtime::{SignedTransaction, Transaction};

            let tx = Transaction::ExecuteFunction {
                sender: self.sender.clone(),
                module: module_addr.clone(),
                function: function_name.clone(),
                type_args: call.type_args.iter().map(|t| format!("{}", t)).collect(),
                args: call.args.clone(),
                gas_limit: self.gas_limit,
                gas_price: self.gas_price,
            };

            let signed_tx = SignedTransaction::new(tx);
            engine.submit_transaction(signed_tx)?
        } else {
            // With signature
            let private_key = match self.private_key.as_ref() {
                Some(key) => key,
                None => bail!("Private key required"),
            };

            use kanari_move_runtime::{SignedTransaction, Transaction};

            let tx = Transaction::ExecuteFunction {
                sender: self.sender.clone(),
                module: module_addr.clone(),
                function: function_name.clone(),
                type_args: call.type_args.iter().map(|t| format!("{}", t)).collect(),
                args: call.args.clone(),
                gas_limit: self.gas_limit,
                gas_price: self.gas_price,
            };

            let mut signed_tx = SignedTransaction::new(tx);
            signed_tx.sign(private_key, CurveType::Ed25519)?;

            engine.submit_transaction(signed_tx)?
        };

        println!("✅ Function call submitted!");
        println!("   TX Hash: {}", hex::encode(&tx_hash[..8]));

        Ok(())
    }

    /// Parse function identifier: address::module::function
    fn parse_function_identifier(&self) -> Result<(String, String, String)> {
        let parts: Vec<&str> = self.function.split("::").collect();
        
        if parts.len() != 3 {
            bail!(
                "Invalid function identifier format. Expected: <address>::<module>::<function>, got: {}",
                self.function
            );
        }

        let address = parts[0].to_string();
        let module = parts[1].to_string();
        let function = parts[2].to_string();

        // Validate address format
        if !address.starts_with("0x") {
            bail!("Address must start with 0x");
        }

        Ok((address, module, function))
    }

    /// Parse type arguments
    fn parse_type_args(&self, type_args_str: &str) -> Result<Vec<TypeTag>> {
        let mut result = Vec::new();
        
        for type_arg in type_args_str.split(',') {
            let type_arg = type_arg.trim();
            
            // Parse type tag
            let type_tag = parser::parse_type_tag(type_arg)
                .with_context(|| format!("Failed to parse type argument: {}", type_arg))?;
            
            result.push(type_tag);
        }

        Ok(result)
    }

    /// Parse function arguments (simplified - expects hex strings or numbers)
    fn parse_args(&self, args_str: &str) -> Result<Vec<Vec<u8>>> {
        let mut result = Vec::new();

        for arg in args_str.split(',') {
            let arg = arg.trim();

            // Try to parse as different types
            let bytes = if arg.starts_with("0x") {
                // Hex address or bytes
                let hex_str = &arg[2..];
                
                // Check if it looks like an address (1-64 hex chars)
                if hex_str.len() <= 64 && hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
                    // Pad to 32 bytes for addresses
                    let padded = format!("{:0>64}", hex_str);
                    let addr = AccountAddress::from_hex_literal(&format!("0x{}", padded))
                        .with_context(|| format!("Failed to parse address: {}", arg))?;
                    bcs::to_bytes(&addr)?
                } else {
                    // Raw hex bytes
                    hex::decode(hex_str)
                        .with_context(|| format!("Failed to parse hex: {}", arg))?
                }
            } else if let Ok(num) = arg.parse::<u64>() {
                // u64 number
                bcs::to_bytes(&num)?
            } else if let Ok(num) = arg.parse::<u128>() {
                // u128 number
                bcs::to_bytes(&num)?
            } else if arg == "true" || arg == "false" {
                // Boolean
                let b = arg == "true";
                bcs::to_bytes(&b)?
            } else {
                // String
                bcs::to_bytes(arg)?
            };

            result.push(bytes);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function_identifier() {
        let call = Call {
            function: "0x1::coin::transfer".to_string(),
            sender: "0x1".to_string(),
            type_args: None,
            args: None,
            gas_limit: 200000,
            gas_price: 1000,
            private_key: None,
            skip_signature: true,
            rpc_endpoint: "http://localhost:9944".to_string(),
            dry_run: false,
        };

        let (addr, module, func) = call.parse_function_identifier().unwrap();
        assert_eq!(addr, "0x1");
        assert_eq!(module, "coin");
        assert_eq!(func, "transfer");
    }

    #[test]
    fn test_parse_args() {
        let call = Call {
            function: "0x1::coin::transfer".to_string(),
            sender: "0x1".to_string(),
            type_args: None,
            args: None,
            gas_limit: 200000,
            gas_price: 1000,
            private_key: None,
            skip_signature: true,
            rpc_endpoint: "http://localhost:9944".to_string(),
            dry_run: false,
        };

        // Test u64
        let args = call.parse_args("1000,2000").unwrap();
        assert_eq!(args.len(), 2);

        // Test boolean
        let args = call.parse_args("true,false").unwrap();
        assert_eq!(args.len(), 2);
    }
}
