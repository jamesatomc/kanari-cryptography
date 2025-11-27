use anyhow::Result;
use kanari_move_runtime::MoveRuntime;
use move_core_types::account_address::AccountAddress;
use move_binary_format::file_format::CompiledModule;
use std::path::PathBuf;

fn main() -> Result<()> {
    // This example publishes all compiled .mv bytecode modules from the build directory.
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
            // print module id
            if let Ok(compiled) = CompiledModule::deserialize_with_defaults(&bytes) {
                println!("ModuleId: {}::{}", compiled.self_id().address(), compiled.self_id().name());
            }
            rt.publish_module(bytes, sender)?;
            println!("Published {}", name);
        }
    }

    println!("Done.");
    Ok(())
}
