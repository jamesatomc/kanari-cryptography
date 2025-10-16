// cargo run -p kanari-crypto --example hd_wallet_example

use kanari_crypto::keys::{generate_mnemonic, CurveType};
use kanari_crypto::wallet::{save_mnemonic, create_hd_wallet, save_hd_wallet, load_wallet, Wallet};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Kanari Crypto - HD Wallet Example");
    println!("================================");

    // 1) Generate a mnemonic (for demo only)
    let mnemonic = generate_mnemonic(12)?;
    println!("Generated mnemonic: {}", mnemonic);

    // Example addresses vector (empty initially)
    let addresses: Vec<String> = Vec::new();

    // 2) Save mnemonic into keystore with password
    let password = "demo-password-123";
    save_mnemonic(&mnemonic, password, addresses)?;
    println!("Saved mnemonic into keystore (encrypted)");

    // 3) Derive an HD child wallet (example: Ethereum-like path)
    let path = "m/44'/60'/0'/0/0";
    let curve = CurveType::K256;

    let child_wallet: Wallet = create_hd_wallet(password, path, curve)?;
    println!("Derived child wallet for path {} -> address {}", path, child_wallet.address);

    // 4) Persist the child wallet into keystore
    save_hd_wallet(&child_wallet, password)?;
    println!("Saved derived child wallet to keystore");

    // 5) Load the wallet back to verify
    let loaded = load_wallet(&child_wallet.address.to_string(), password)?;
    println!("Loaded wallet from keystore: {}", loaded.address);

    Ok(())
}
