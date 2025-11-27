use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::str::FromStr;
use kanari_types::address::Address;
use kanari_types::module_registry::ModuleRegistry;
use kanari_move_runtime::BlockchainEngine;
use kanari_crypto::{
    keys::{generate_keypair, generate_mnemonic, keypair_from_mnemonic, CurveType},
    wallet::{list_wallet_files, load_wallet, save_wallet, Wallet},
};

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
    /// Transfer Kanari tokens to another address
    Transfer {
        /// Sender wallet address
        #[arg(short, long)]
        from: String,
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount in Kanari (will be converted to Mist)
        #[arg(short, long)]
        amount: f64,
        /// Wallet password
        #[arg(short, long)]
        password: String,
    },
    /// Check wallet balance
    Balance {
        /// Wallet address
        #[arg(short, long)]
        address: String,
    },
    /// Show blockchain statistics
    Stats,
    /// Show available Move modules
    Modules,
}

fn main() -> Result<()> {
	let cli = Cli::parse();

	match cli.command {
        Commands::CreateWallet { password, curve, words } => {
            let curve_type = match curve.to_lowercase().as_str() {
                "ed25519" => CurveType::Ed25519,
                "k256" | "secp256k1" => CurveType::K256,
                "p256" | "secp256r1" => CurveType::P256,
                "dilithium2" => CurveType::Dilithium2,
                "dilithium3" => CurveType::Dilithium3,
                "dilithium5" => CurveType::Dilithium5,
                "sphincs+" | "sphincsplus" => CurveType::SphincsPlusSha256Robust,
                "ed25519+dilithium3" | "ed25519_dilithium3" => CurveType::Ed25519Dilithium3,
                "k256+dilithium3" | "k256_dilithium3" => CurveType::K256Dilithium3,
                other => {
                    println!("Unknown curve '{}', falling back to Ed25519", other);
                    CurveType::Ed25519
                }
            };

            // For classical curves we can derive from a mnemonic; for PQC/hybrid generate directly
            let (private_key, address_str, seed_phrase) = if curve_type.is_post_quantum() || curve_type.is_hybrid() {
                let kp = generate_keypair(curve_type)
                    .context("Failed to generate keypair")?;
                (kp.private_key, kp.address, String::new())
            } else {
                let mnemonic = generate_mnemonic(words)
                    .context("Failed to generate mnemonic")?;
                let kp = keypair_from_mnemonic(&mnemonic, curve_type, "")
                    .context("Failed to derive keypair from mnemonic")?;
                (kp.private_key, kp.address, mnemonic)
            };

            let address = Address::from_str(&address_str)
                .context("Generated invalid address")?;

            // Save wallet
            save_wallet(&address, &private_key, &seed_phrase, &password, curve_type)
                .context("Failed to save wallet")?;

            println!("Created wallet: {}", address_str);
            if !seed_phrase.is_empty() {
                println!("Seed phrase: {}", seed_phrase);
            }

            Ok(())
        }

		Commands::LoadWallet { address, password } => {
            let wallet: Wallet = load_wallet(&address, &password)
                .context("Failed to load wallet")?;
            println!("Wallet loaded: {} (curve: {})", address, wallet.curve_type);
			Ok(())
		}

		Commands::ListWallets => {
			let wallets = list_wallet_files()
				.context("Failed to list wallets")?;
			println!("Found {} wallets", wallets.len());
			Ok(())
		}

		Commands::WalletInfo { address, password, show_secrets } => {
            let wallet = load_wallet(&address, &password)
                .context("Failed to load wallet")?;
            println!("Wallet info for {}", address);
            if show_secrets {
                println!("Private key: {}", wallet.private_key);
                println!("Seed phrase: {}", wallet.seed_phrase);
            } else {
                println!("Address: {}", wallet.address.to_string());
            }
            Ok(())
		}

		Commands::Transfer { from, to, amount, password } => {
			// Load sender wallet to verify ownership
			let _wallet = load_wallet(&from, &password)
				.context("Failed to load sender wallet")?;

			println!("💸 Transferring Kanari tokens...");
			println!("  From: {}", from);
			println!("  To: {}", to);
			println!("  Amount: {} KANARI", amount);

			// Convert Kanari to Mist (1 KANARI = 10^9 Mist)
			const MIST_PER_KANARI: f64 = 1_000_000_000.0;
			let amount_mist = (amount * MIST_PER_KANARI) as u64;
			println!("  Amount (Mist): {}", amount_mist);

			// Initialize blockchain engine
			let engine = BlockchainEngine::new()
				.context("Failed to initialize blockchain engine")?;

			// Submit transfer transaction with gas
			let tx = kanari_move_runtime::Transaction::new_transfer(
				from.clone(),
				to.clone(),
				amount_mist,
			);

			println!("  Gas Limit: {}", tx.gas_limit());
			println!("  Gas Price: {} Mist/gas", tx.gas_price());

			let tx_hash = engine.submit_transaction(tx)
				.context("Failed to submit transaction")?;

			println!("  ✅ Transaction submitted: {}", hex::encode(&tx_hash[..16]));

			// Try to produce a block
			match engine.produce_block() {
				Ok(block_info) => {
					println!("  ⛏️  Block #{} produced", block_info.height);
					println!("     Executed: {} txs", block_info.executed);
					if block_info.failed > 0 {
						println!("     Failed: {} txs", block_info.failed);
					}
				}
				Err(e) => {
					eprintln!("  ⚠️  Block production failed: {}", e);
					println!("  Transaction is pending...");
				}
			}

			Ok(())
		}

		Commands::Balance { address } => {
			let engine = BlockchainEngine::new()
				.context("Failed to initialize blockchain engine")?;

			match engine.get_account_info(&address) {
				Some(account) => {
					const MIST_PER_KANARI: f64 = 1_000_000_000.0;
					let balance_kanari = account.balance as f64 / MIST_PER_KANARI;

					println!("💰 Balance for {}", address);
					println!("  Kanari: {:.9} KANARI", balance_kanari);
					println!("  Mist: {} Mist", account.balance);
					println!("  Sequence: {}", account.sequence_number);
					if !account.modules.is_empty() {
						println!("  Modules deployed: {}", account.modules.len());
					}
				}
				None => {
					println!("❌ Account not found: {}", address);
					println!("   This address has no transactions yet.");
				}
			}

			Ok(())
		}

		Commands::Stats => {
			let engine = BlockchainEngine::new()
				.context("Failed to initialize blockchain engine")?;

			let stats = engine.get_stats();
			const MIST_PER_KANARI: f64 = 1_000_000_000.0;
			let total_supply_kanari = stats.total_supply as f64 / MIST_PER_KANARI;

			println!("📊 Kanari Blockchain Statistics");
			println!("─────────────────────────────────");
			println!("  Block Height: {}", stats.height);
			println!("  Total Blocks: {}", stats.total_blocks);
			println!("  Total Transactions: {}", stats.total_transactions);
			println!("  Pending Transactions: {}", stats.pending_transactions);
			println!("  Total Accounts: {}", stats.total_accounts);
			println!("  Total Supply: {:.0} KANARI", total_supply_kanari);
			println!("─────────────────────────────────");

			Ok(())
		}

		Commands::Modules => {
			println!("📦 Available Move Modules");
			println!("─────────────────────────────────");

			for info in ModuleRegistry::all_modules_info() {
				println!("\n{}", info.display());
			}

			println!("\n─────────────────────────────────");
			println!("Total modules: {}", ModuleRegistry::all_modules().len());

			Ok(())
		}

	}
}
