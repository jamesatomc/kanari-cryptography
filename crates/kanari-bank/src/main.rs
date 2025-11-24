use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::info;
use std::fs;
use std::path::PathBuf;

// Move VM imports
use move_core_types::account_address::AccountAddress;

// Kanari Crypto imports
use kanari_crypto::{
    keys::{generate_keypair, generate_mnemonic, CurveType},
    wallet::{list_wallet_files, load_wallet, save_wallet},
};

mod move_compiler_wrapper;
mod move_runtime;
mod move_vm_state;

use move_compiler_wrapper::compile_simple_package;
use move_runtime::MoveRuntime;
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
    /// Create a new wallet with kanari-crypto
    CreateWallet {
        /// Password for wallet encryption
        #[arg(short, long)]
        password: String,
        /// Curve type (ed25519, k256, p256, dilithium2, dilithium3, dilithium5)
        #[arg(short, long, default_value = "ed25519")]
        curve: String,
        /// Number of seed words (12 or 24)
        #[arg(short, long, default_value = "12")]
        words: usize,
    },
    /// Load an existing wallet
    LoadWallet {
        /// Wallet address to load
        #[arg(short, long)]
        address: String,
        /// Password to decrypt wallet
        #[arg(short, long)]
        password: String,
    },
    /// List all wallets with balances
    ListWallets,
    /// Show detailed wallet information
    WalletInfo {
        /// Wallet address
        #[arg(short, long)]
        address: String,
        /// Password to decrypt wallet
        #[arg(short, long)]
        password: String,
        /// Show private key and seed phrase (dangerous!)
        #[arg(long, default_value = "false")]
        show_secrets: bool,
    },
    /// Mint new coins to a wallet
    Mint {
        /// Amount to mint
        #[arg(short, long)]
        amount: u64,
        /// Recipient wallet address
        #[arg(short, long)]
        recipient: String,
    },
    /// Signed transfer with wallet authentication
    SignedTransfer {
        /// Sender wallet address
        #[arg(short, long)]
        from: String,
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount to transfer
        #[arg(short, long)]
        amount: u64,
        /// Wallet password
        #[arg(short, long)]
        password: String,
    },
    /// Batch transfer to multiple recipients with wallet authentication
    BatchTransfer {
        /// Sender wallet address
        #[arg(short, long)]
        from: String,
        /// Recipients (comma separated addresses)
        #[arg(short, long)]
        recipients: String,
        /// Amounts (comma separated)
        #[arg(short, long)]
        amounts: String,
        /// Wallet password
        #[arg(short, long)]
        password: String,
    },
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
    let bytes = hex::decode(addr_str).context("Failed to decode address")?;

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
        Commands::CreateWallet { password, curve, words } => {
            // Validate word count
            if words != 12 && words != 24 {
                println!("❌ Invalid word count. Use 12 or 24.");
                return Ok(());
            }

            // Parse curve type
            let curve_type = match curve.to_lowercase().as_str() {
                "ed25519" => CurveType::Ed25519,
                "k256" => CurveType::K256,
                "p256" => CurveType::P256,
                "dilithium2" => CurveType::Dilithium2,
                "dilithium3" => CurveType::Dilithium3,
                "dilithium5" => CurveType::Dilithium5,
                _ => {
                    println!("❌ Invalid curve type. Use: ed25519, k256, p256, dilithium2, dilithium3, or dilithium5");
                    return Ok(());
                }
            };

            // Generate mnemonic with selected word count
            let mnemonic = generate_mnemonic(words)?;
            println!("🔑 Generated Wallet ({} words)", words);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("\n📝 SEED PHRASE (SAVE THIS SECURELY!):");
            println!("   {}", mnemonic);
            println!();

            // Generate keypair from curve
            let keypair = generate_keypair(curve_type)?;

            // Create address from public key hash
            let pub_key_bytes =
                hex::decode(&keypair.public_key).context("Failed to decode public key")?;
            let address_bytes = kanari_crypto::hash_data(&pub_key_bytes);
            let mut addr_array = [0u8; 32];
            addr_array.copy_from_slice(&address_bytes[..32]);
            let address = kanari_types::address::Address::new(addr_array);

            // Save wallet with proper parameters
            let private_key_hex = keypair.private_key.clone();
            save_wallet(
                &address,
                &private_key_hex,
                &mnemonic,
                &password,
                curve_type,
            )?;

            // Create account in Move VM state
            let addr = parse_address(&address.to_hex())?;
            state.create_account(addr).ok();
            state.save()?;

            println!("🔐 PRIVATE KEY (NEVER SHARE THIS!):");
            println!("   {}", private_key_hex);
            println!();
            println!("✓ Wallet created successfully!");
            println!("  📍 Address: {}", address);
            println!("  🔒 Curve: {:?}", curve_type);
            println!("  ✅ Account registered in Move VM");
            println!("\n⚠️  WARNING: Save your seed phrase and private key in a secure location!");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }

        Commands::LoadWallet { address, password } => {
            match load_wallet(&address, &password) {
                Ok(wallet) => {
                    println!("✓ Wallet loaded successfully!");
                    println!("  Address: {}", wallet.address);
                    println!("  Curve: {:?}", wallet.curve_type);

                    // Check balance
                    let addr = parse_address(&wallet.address.to_hex())?;
                    let balance = state.get_balance(&addr);
                    println!("  Balance: {} coins", balance);
                }
                Err(e) => {
                    println!("❌ Failed to load wallet: {}", e);
                }
            }
        }

        Commands::ListWallets => {
            match list_wallet_files() {
                Ok(wallets) => {
                    if wallets.is_empty() {
                        println!("No wallets found.");
                    } else {
                        println!("Available Wallets:");
                        println!("{:<66} {:>15}", "Address", "Balance");
                        println!("{}", "=".repeat(82));
                        for (wallet_addr, _is_selected) in wallets {
                            // Try to get balance
                            if let Ok(addr) = parse_address(&wallet_addr) {
                                let balance = state.get_balance(&addr);
                                println!("{:<66} {:>15}", wallet_addr, balance);
                            } else {
                                println!("{:<66} {:>15}", wallet_addr, "N/A");
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to list wallets: {}", e);
                }
            }
        }

        Commands::WalletInfo { address, password, show_secrets } => {
            match load_wallet(&address, &password) {
                Ok(wallet) => {
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("📱 Wallet Information");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("📍 Address: {}", wallet.address);
                    println!("🔐 Curve: {:?}", wallet.curve_type);
                    
                    // Check balance
                    let addr = parse_address(&wallet.address.to_hex())?;
                    let balance = state.get_balance(&addr);
                    println!("💰 Balance: {} coins", balance);
                    
                    if show_secrets {
                        println!("\n⚠️  SENSITIVE INFORMATION (NEVER SHARE!)");
                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                        println!("📝 Seed Phrase:");
                        println!("   {}", wallet.seed_phrase);
                        println!("\n🔑 Private Key:");
                        println!("   {}", wallet.private_key);
                    } else {
                        println!("\n💡 Use --show-secrets to display seed phrase and private key");
                    }
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                }
                Err(e) => {
                    println!("❌ Failed to load wallet: {}", e);
                }
            }
        }

        Commands::SignedTransfer {
            from,
            to,
            amount,
            password,
        } => {
            // Load wallet
            let wallet = load_wallet(&from, &password)
                .map_err(|e| anyhow::anyhow!("Failed to load wallet: {}", e))?;

            // Parse addresses
            let from_addr = parse_address(&wallet.address.to_hex())?;
            let to_addr = parse_address(&to)?;

            // Create transaction message
            let tx_message = format!("transfer:{}:{}:{}", from, to, amount);

            // Sign the transaction
            let signature = wallet
                .sign(tx_message.as_bytes(), &password)
                .map_err(|e| anyhow::anyhow!("Failed to sign transaction: {}", e))?;

            println!("✓ Transaction signed");
            println!("  Signature: {}", hex::encode(&signature));

            // Perform transfer via Move VM
            state.transfer(&mut runtime, from_addr, to_addr, amount)?;
            state.save()?;

            println!("✓ Signed transfer completed");
            println!(
                "  From: {} (balance: {})",
                from,
                state.get_balance(&from_addr)
            );
            println!("  To: {} (balance: {})", to, state.get_balance(&to_addr));
            println!("  Amount: {}", amount);
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

        Commands::BatchTransfer {
            from,
            recipients,
            amounts,
            password,
        } => {
            // Load and verify wallet
            let wallet = load_wallet(&from, &password)
                .map_err(|e| anyhow::anyhow!("Failed to load wallet: {}", e))?;

            let from_addr = parse_address(&wallet.address.to_hex())?;
            let recipient_addrs: Vec<String> = recipients
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            let amount_vals: Vec<u64> = amounts
                .split(',')
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

            // Sign the batch transaction
            let batch_message = format!("batch:{}:{}:{}", from, recipients, amounts);
            let signature = wallet
                .sign(batch_message.as_bytes(), &password)
                .map_err(|e| anyhow::anyhow!("Failed to sign batch transfer: {}", e))?;

            println!("✓ Batch transfer initiated (signed)");
            println!("  From: {}", from);
            println!("  Recipients: {}", recipient_addrs.len());
            println!("  Total amount: {}", total);
            println!(
                "  Signature: {}",
                hex::encode(&signature[..32.min(signature.len())])
            );

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
                        println!(
                            "  Module {}: {} bytes → {:?}",
                            i + 1,
                            module_bytes.len(),
                            module_file
                        );
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

                                    if module_path.extension().and_then(|s| s.to_str())
                                        == Some("mv")
                                    {
                                        let module_bytes = std::fs::read(&module_path)?;
                                        match runtime.load_module(module_bytes) {
                                            Ok(module_id) => {
                                                println!(
                                                    "  ✓ Loaded: {}::{}",
                                                    module_id.address(),
                                                    module_id.name()
                                                );
                                            }
                                            Err(e) => {
                                                println!(
                                                    "  ⚠️  Failed to load {}: {}",
                                                    module_path.display(),
                                                    e
                                                );
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