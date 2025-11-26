use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::info;
use std::fs;

// Move VM imports
use move_core_types::account_address::AccountAddress;

// Kanari Crypto imports
use kanari_crypto::{
    keys::{generate_keypair, generate_mnemonic, CurveType},
    wallet::{list_wallet_files, load_wallet, save_wallet},
};

use kanari_move_runtime::{MoveRuntime, MoveVMState};
use kanari_types::genesis;
use kanari_types::kanari::KanariModule;

/// Kanari - A Move-based money transfer system
#[derive(Parser)]
#[command(name = "kanari")]
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
    
    /// Signed transfer with wallet authentication
    SignedTransfer {
        /// Sender wallet address
        #[arg(short, long)]
        from: String,
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount to transfer in KANARI (e.g., 0.5 for 0.5 KANARI)
        #[arg(short, long)]
        amount: f64,
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
}

/// Convert KANARI amount to MIST
fn kanari_to_mist(kanari: f64) -> Result<u64> {
    if kanari < 0.0 {
        anyhow::bail!("Amount cannot be negative");
    }
    let mist = (kanari * 1_000_000_000.0).round() as u64;
    Ok(mist)
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

    info!("Kanari - Move-based Transfer System");
    info!("==========================================");

    let mut state = MoveVMState::load()?;
    let mut runtime = MoveRuntime::new()?;

    // First-run genesis bootstrap: if total supply is zero, seed developer account
    if state.get_total_supply() == 0 {
        match genesis::dev_account_address() {
            Ok(dev_addr) => {
                let total = KanariModule::TOTAL_SUPPLY_MIST;
                println!("Genesis: calling Move initializer and minting {} MIST to {}", total, dev_addr);

                // Call Move runtime genesis initializer (for on-chain/module auditability)
                if let Err(e) = runtime.initialize_genesis(&dev_addr) {
                    println!("Warning: Move genesis initializer failed: {}", e);
                } else {
                    println!("✓ Move genesis initializer executed")
                }

                // Update persistent MoveVMState so balances/total_supply reflect genesis
                state.mint(dev_addr, total)?;
                state.save()?;
                println!("✓ Genesis completed: total supply set (local state updated)")
            }
            Err(e) => {
                println!("Failed to parse dev address for genesis: {}", e);
            }
        }
    }

    // Auto-load Move module for CLI mode
    let default_module_path =
        "crates//packages/kanari-system/build/KanariSystem/bytecode_modules/transfer.mv";
    if std::path::Path::new(default_module_path).exists() {
        if let Ok(module_bytes) = fs::read(default_module_path) {
            let _ = runtime.load_module(module_bytes);
        }
    }

    match cli.command {
        Commands::CreateWallet {
            password,
            curve,
            words,
        } => {
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
            save_wallet(&address, &private_key_hex, &mnemonic, &password, curve_type)?;

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
                    let balance = state.get_balance_formatted(&addr);
                    println!("  Balance: {}", balance);
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
                        println!("{:<66} {:>20}", "Address", "Balance");
                        println!("{}", "=".repeat(87));
                        for (wallet_addr, _is_selected) in wallets {
                            // Try to get balance
                            if let Ok(addr) = parse_address(&wallet_addr) {
                                let balance = state.get_balance_formatted(&addr);
                                println!("{:<66} {:>20}", wallet_addr, balance);
                            } else {
                                println!("{:<66} {:>20}", wallet_addr, "N/A");
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to list wallets: {}", e);
                }
            }
        }

        Commands::WalletInfo {
            address,
            password,
            show_secrets,
        } => {
            match load_wallet(&address, &password) {
                Ok(wallet) => {
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("📱 Wallet Information");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("📍 Address: {}", wallet.address);
                    println!("🔐 Curve: {:?}", wallet.curve_type);

                    // Check balance
                    let addr = parse_address(&wallet.address.to_hex())?;
                    let balance = state.get_balance_formatted(&addr);
                    println!("💰 Balance: {}", balance);

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

            // Convert KANARI to MIST
            let amount_mist = kanari_to_mist(amount)?;

            // Create transaction message
            let tx_message = format!("transfer:{}:{}:{}", from, to, amount);

            // Sign the transaction
            let signature = wallet
                .sign(tx_message.as_bytes(), &password)
                .map_err(|e| anyhow::anyhow!("Failed to sign transaction: {}", e))?;

            println!("✓ Transaction signed");
            println!("  Signature: {}", hex::encode(&signature));

            // Perform transfer via Move VM
            state.transfer(&mut runtime, from_addr, to_addr, amount_mist)?;
            state.save()?;

            println!("✓ Signed transfer completed");
            println!(
                "  From: {} (balance: {})",
                from,
                state.get_balance_formatted(&from_addr)
            );
            println!(
                "  To: {} (balance: {})",
                to,
                state.get_balance_formatted(&to_addr)
            );
            println!("  Amount: {} KANARI ({} MIST)", amount, amount_mist);
        }

        

        Commands::Reset { confirm } => {
            if !confirm {
                println!("⚠️  Warning: This will delete all data!");
                println!("Use --confirm flag to proceed");
            } else {
                // MoveVMState now uses RocksDB directory for persistence. Remove the DB directory.
                let db_path = MoveVMState::db_path();
                if db_path.exists() {
                    std::fs::remove_dir_all(&db_path)?;
                    println!("✓ Data has been reset (RocksDB directory removed)");
                } else {
                    println!("No data directory found");
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
    }

    Ok(())
}
