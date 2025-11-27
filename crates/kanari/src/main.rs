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


fn main() {

}
