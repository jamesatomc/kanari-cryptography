use anyhow::Result;
use hex::encode as hex_encode;
use kanari_move_runtime::{MoveRuntime};
use kanari_crypto::wallet::list_wallet_files;

use move_core_types::account_address::AccountAddress;

use std::path::PathBuf;
use std::{env, thread, time::{Duration, SystemTime, UNIX_EPOCH}};

fn main() -> Result<()> {
	// Simple CLI: subcommands: run | publish-all | list-wallets | publish-file <path>
	let args: Vec<String> = env::args().collect();
	let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("run");

	match cmd {
		"list-wallets" => {
			let wallets = list_wallet_files()?;
			for (addr, selected) in wallets {
				println!("{}{}", addr, if selected { " (selected)" } else { "" });
			}
			return Ok(());
		}

		"publish-file" => {
			let path = match args.get(2) {
				Some(p) => PathBuf::from(p),
				None => {
					eprintln!("Usage: publish-file <path-to-bytecode.mv>");
					std::process::exit(2);
				}
			};

			let mut rt = MoveRuntime::new()?;
			let bytes = std::fs::read(&path)?;
			// use system address as sender
			let sender = AccountAddress::from_hex_literal("0x2")?;
			println!("Publishing {}...", path.display());
			rt.publish_module(bytes, sender)?;
			println!("Published.");
			return Ok(());
		}

		"publish-all" => {
			// Path relative to workspace root where build output is placed
			let modules_dir = PathBuf::from("crates/kanari-frameworks/packages/kanari-system/build/KanariSystem/bytecode_modules");
			if !modules_dir.exists() {
				eprintln!("Bytecode modules directory not found: {}", modules_dir.display());
				eprintln!("Build the Move package first (see README).");
				std::process::exit(1);
			}

			let mut rt = MoveRuntime::new()?;
			let sender = AccountAddress::from_hex_literal("0x2")?;

			for entry in std::fs::read_dir(&modules_dir)? {
				let entry = entry?;
				let path = entry.path();
				if path.extension().and_then(|s| s.to_str()) == Some("mv") {
					let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("<file>");
					println!("Publishing {}...", name);
					let bytes = std::fs::read(&path)?;
					if let Err(e) = rt.publish_module(bytes, sender) {
						eprintln!("Failed to publish {}: {:?}", name, e);
					} else {
						println!("Published {}", name);
					}
				}
			}
			println!("Publish-all complete.");
			return Ok(());
		}

		"run-genesis" => {
			let mut rt = MoveRuntime::new()?;
			println!("Running genesis...");
			if let Err(e) = rt.run_genesis() {
				eprintln!("Genesis failed: {:?}", e);
				std::process::exit(1);
			} else {
				println!("Genesis executed successfully.");
			}
			return Ok(());
		}

		"run" | _ => {
			// fallthrough to node run
		}
	}

	// Default: run node loop
	let mut runtime = MoveRuntime::new()?;

	println!("Kanari node starting...");
	let mut tick: u64 = 0;
	loop {
		tick += 1;
		let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
		let wallets = list_wallet_files().unwrap_or_default();
		println!("[{}] tick={} wallets={} uptime_seconds={}", hex_encode(&now.to_be_bytes()), tick, wallets.len(), now);
		thread::sleep(Duration::from_secs(5));
	}
}
