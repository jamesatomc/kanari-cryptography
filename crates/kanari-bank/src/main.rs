use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use log::info;
use clap::{Parser, Subcommand};

// Move VM imports
use move_core_types::account_address::AccountAddress;

mod move_runtime;
mod move_compiler_wrapper;
mod move_vm_state;

use move_runtime::MoveRuntime;
use move_compiler_wrapper::compile_simple_package;
use move_vm_state::MoveVMState;

/// Kanari Bank - A Move-based money transfer system
#[derive(Parser)]
#[command(name = "kanari-bank")]
#[command(about = "Money transfer system using Move language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new account
    CreateAccount {
        /// Address of the new account
        #[arg(short, long)]
        address: String,
    },
    /// Mint new coins
    Mint {
        /// Amount to mint
        #[arg(short, long)]
        amount: u64,
        /// Recipient address
        #[arg(short, long)]
        recipient: String,
    },
    /// Transfer coins
    Transfer {
        /// Sender address
        #[arg(short, long)]
        from: String,
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount to transfer
        #[arg(short, long)]
        amount: u64,
    },
    /// Check balance
    Balance {
        /// Account address
        #[arg(short, long)]
        address: String,
    },
    /// Create escrow
    Escrow {
        /// Sender address
        #[arg(short, long)]
        from: String,
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount to escrow
        #[arg(short, long)]
        amount: u64,
    },
    /// Batch transfer
    BatchTransfer {
        /// Sender address
        #[arg(short, long)]
        from: String,
        /// Recipients (comma separated addresses)
        #[arg(short, long)]
        recipients: String,
        /// Amounts (comma separated)
        #[arg(short, long)]
        amounts: String,
    },
    /// List all accounts
    List,
    /// Reset all data (careful!)
    Reset {
        /// Confirm reset
        #[arg(short, long)]
        confirm: bool,
    },
    /// Compile Move package
    CompileMove {
        /// Path to Move package
        #[arg(short, long, default_value = "crates/packages/system")]
        path: String,
    },
    /// Initialize Move VM with compiled modules
    InitMove,
}

/// Parse address from hex string
fn parse_address(addr_str: &str) -> Result<AccountAddress> {
    let addr_str = addr_str.trim_start_matches("0x");
    let bytes = hex::decode(addr_str)
        .context("Failed to decode address")?;
    
    // Ensure the address is 32 bytes
    let mut addr_bytes = [0u8; AccountAddress::LENGTH];
    if bytes.len() <= AccountAddress::LENGTH {
        addr_bytes[AccountAddress::LENGTH - bytes.len()..].copy_from_slice(&bytes);
        Ok(AccountAddress::new(addr_bytes))
    } else {
        anyhow::bail!("Address too long")
    }
}

fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    info!("Kanari Bank - Move-based Transfer System");
    info!("==========================================");

    // Always use Move VM
    println!("🚀 Using Move VM");
    
    let mut state = MoveVMState::load()?;
    let mut runtime = MoveRuntime::new()?;
    
    // Load compiled modules
    let package_path = PathBuf::from("crates/packages/system");
    let build_dir = package_path.join("build/kanari/bytecode_modules");
    
    if build_dir.exists() {
        for entry in std::fs::read_dir(&build_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("mv") {
                let module_bytes = std::fs::read(&path)?;
                runtime.load_module(module_bytes)?;
            }
        }
        println!("✓ Move VM initialized with modules");
    } else {
        println!("⚠️  No compiled modules found. Run: compile-move first");
    }

    match cli.command {
        Commands::CreateAccount { address } => {
            let addr = parse_address(&address)?;
            state.create_account(addr)?;
            state.save()?;
            println!("✓ Account created: {}", address);
        }
        
        Commands::Mint { amount, recipient } => {
            let addr = parse_address(&recipient)?;
            state.create_account(addr).ok();
            let current_balance = state.get_balance(&addr);
            state.set_balance(addr, current_balance + amount);
            state.save()?;
            println!("✓ Minted {} coins to {}", amount, recipient);
            println!("  New balance: {}", state.get_balance(&addr));
        }
        
        Commands::Transfer { from, to, amount } => {
            let from_addr = parse_address(&from)?;
            let to_addr = parse_address(&to)?;
            
            // Use Move VM for transfer
            state.transfer(&mut runtime, from_addr, to_addr, amount)?;
            state.save()?;
            
            println!("✓ Transfer completed (via Move VM)");
            println!("  From: {} (balance: {})", from, state.get_balance(&from_addr));
            println!("  To: {} (balance: {})", to, state.get_balance(&to_addr));
            println!("  Amount: {}", amount);
        }
        
        Commands::Balance { address } => {
            let addr = parse_address(&address)?;
            let balance = state.get_balance(&addr);
            println!("Balance of {}: {}", address, balance);
        }
        
        Commands::List => {
            if state.accounts().is_empty() {
                println!("No accounts found.");
            } else {
                println!("All Accounts:");
                println!("{:<66} {:>15}", "Address", "Balance");
                println!("{}", "=".repeat(82));
                for (addr, balance) in state.accounts() {
                    println!("{:<66} {:>15}", addr, balance);
                }
                let total: u64 = state.accounts().values().sum();
                println!("{}", "=".repeat(82));
                println!("{:<66} {:>15}", "Total Supply", total);
                
                if !state.transfers().is_empty() {
                    println!("\nTransfer History:");
                    for (i, transfer) in state.transfers().iter().enumerate() {
                        println!("  {}. {} → {} : {} coins", 
                            i + 1, transfer.from, transfer.to, transfer.amount);
                    }
                }
            }
        }
        
        Commands::Reset { confirm } => {
            if !confirm {
                println!("⚠️  Warning: This will delete all data!");
                println!("Use --confirm flag to proceed");
            } else {
                let path = MoveVMState::data_file();
                if path.exists() {
                    fs::remove_file(&path)?;
                    println!("✓ Data has been reset");
                } else {
                    println!("No data file found");
                }
            }
        }
        
        Commands::Escrow { from, to, amount } => {
            let from_addr = parse_address(&from)?;
            let _to_addr = parse_address(&to)?;
            
            let from_balance = state.get_balance(&from_addr);
            if from_balance < amount {
                anyhow::bail!("Insufficient balance for escrow");
            }
            
            println!("✓ Escrow created");
            println!("  From: {}", from);
            println!("  To: {}", to);
            println!("  Amount: {}", amount);
            println!("  Note: Escrow functionality requires additional Move module development");
        }
        
        Commands::BatchTransfer { from, recipients, amounts } => {
            let from_addr = parse_address(&from)?;
            let recipient_addrs: Vec<String> = recipients.split(',').map(|s| s.trim().to_string()).collect();
            let amount_vals: Vec<u64> = amounts.split(',')
                .map(|s| s.trim().parse::<u64>().unwrap_or(0))
                .collect();
            
            if recipient_addrs.len() != amount_vals.len() {
                anyhow::bail!("Recipients and amounts count mismatch");
            }
            
            let total: u64 = amount_vals.iter().sum();
            let from_balance = state.get_balance(&from_addr);
            
            if from_balance < total {
                anyhow::bail!("Insufficient balance for batch transfer");
            }
            
            println!("✓ Batch transfer initiated");
            println!("  From: {}", from);
            println!("  Recipients: {}", recipient_addrs.len());
            println!("  Total amount: {}", total);
            
            for (i, recipient) in recipient_addrs.iter().enumerate() {
                let to_addr = parse_address(recipient)?;
                let amount = amount_vals[i];
                state.transfer(&mut runtime, from_addr, to_addr, amount)?;
                println!("  → {} : {} coins", recipient, amount);
            }
            
            state.save()?;
            println!("  Remaining balance: {}", state.get_balance(&from_addr));
        }
        
        Commands::CompileMove { path } => {
            println!("🔨 Compiling Move package at: {}", path);
            let package_path = PathBuf::from(&path);
            
            if !package_path.exists() {
                anyhow::bail!("Package path does not exist: {}", path);
            }

            println!("📦 Using built-in Move compiler...");
            
            match compile_simple_package(&package_path) {
                Ok(modules) => {
                    println!("✓ Successfully compiled {} module(s)", modules.len());
                    
                    // Save compiled modules to build directory
                    let build_dir = package_path.join("build/kanari/bytecode_modules");
                    std::fs::create_dir_all(&build_dir)?;
                    
                    for (i, module_bytes) in modules.iter().enumerate() {
                        let module_file = build_dir.join(format!("module_{}.mv", i));
                        std::fs::write(&module_file, module_bytes)?;
                        println!("  Module {}: {} bytes → {:?}", i + 1, module_bytes.len(), module_file);
                    }
                    
                    println!("✓ Modules saved to {:?}", build_dir);
                }
                Err(e) => {
                    println!("❌ Compilation failed: {}", e);
                    return Err(e);
                }
            }
        }
        
        Commands::InitMove => {
            println!("🚀 Initializing Move VM...");
            
            match MoveRuntime::new() {
                Ok(mut runtime) => {
                    println!("✓ Move VM initialized");
                    
                    // Try to load compiled modules
                    let package_path = PathBuf::from("crates/packages/system");
                    let build_dir = package_path.join("build");
                    
                    if build_dir.exists() {
                        println!("📦 Loading compiled modules...");
                        
                        // Look for bytecode_modules in build directory
                        for entry in std::fs::read_dir(&build_dir)? {
                            let entry = entry?;
                            let bytecode_dir = entry.path().join("bytecode_modules");
                            
                            if bytecode_dir.exists() {
                                for module_entry in std::fs::read_dir(&bytecode_dir)? {
                                    let module_entry = module_entry?;
                                    let module_path = module_entry.path();
                                    
                                    if module_path.extension().and_then(|s| s.to_str()) == Some("mv") {
                                        let module_bytes = std::fs::read(&module_path)?;
                                        match runtime.load_module(module_bytes) {
                                            Ok(module_id) => {
                                                println!("  ✓ Loaded: {}::{}", 
                                                    module_id.address(), 
                                                    module_id.name());
                                            }
                                            Err(e) => {
                                                println!("  ⚠️  Failed to load {}: {}", 
                                                    module_path.display(), e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        println!("✓ Move VM ready");
                    } else {
                        println!("⚠️  No compiled modules found.");
                        println!("   Run: cargo run --bin kanari-bank -- compile-move");
                    }
                }
                Err(e) => {
                    println!("❌ Failed to initialize Move VM: {}", e);
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let addr_str = "0x1234567890abcdef";
        assert!(parse_address(addr_str).is_ok());
        
        let addr_str2 = "1234567890abcdef";
        assert!(parse_address(addr_str2).is_ok());
    }
}
