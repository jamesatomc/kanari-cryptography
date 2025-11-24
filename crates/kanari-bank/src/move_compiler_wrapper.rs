use anyhow::{Result, Context};
use move_compiler::{Compiler, Flags};
use move_command_line_common::address::NumericalAddress;
use move_symbol_pool::Symbol;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

/// Compile Move source files to bytecode
pub fn compile_move_source(
    source_files: Vec<PathBuf>,
    dependencies: Vec<PathBuf>,
    named_addresses: BTreeMap<Symbol, NumericalAddress>,
) -> Result<Vec<Vec<u8>>> {
    
    let flags = Flags::empty();
    
    // Convert paths to symbols
    let sources: Vec<Symbol> = source_files
        .iter()
        .map(|p| Symbol::from(p.to_string_lossy().as_ref()))
        .collect();
    
    let deps: Vec<Symbol> = dependencies
        .iter()
        .map(|p| Symbol::from(p.to_string_lossy().as_ref()))
        .collect();

    // Compile
    let (_files, compiled_units) = Compiler::from_files(
        None, // no pre-compiled lib
        sources,
        deps,
        named_addresses,
    )
    .set_flags(flags)
    .build_and_report()
    .context("Move compilation failed")?;

    // Extract compiled modules
    let mut compiled_modules = Vec::new();
    
    for unit in compiled_units {
        // Get the compiled module
        let named_module = unit.into_compiled_unit();
        let mut bytecode = Vec::new();
        named_module.module.serialize(&mut bytecode)
            .context("Failed to serialize module")?;
        compiled_modules.push(bytecode);
    }

    Ok(compiled_modules)
}

/// Compile a simple Move package
pub fn compile_simple_package(package_dir: &Path) -> Result<Vec<Vec<u8>>> {
    let sources_dir = package_dir.join("sources");
    
    if !sources_dir.exists() {
        anyhow::bail!("Sources directory not found: {:?}", sources_dir);
    }

    // Collect all .move files
    let mut source_files = Vec::new();
    for entry in std::fs::read_dir(&sources_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("move") {
            source_files.push(path);
        }
    }

    if source_files.is_empty() {
        anyhow::bail!("No Move source files found in {:?}", sources_dir);
    }

    // Setup dependencies (move-stdlib)
    let stdlib_path = package_dir
        .join("../../../third_party/move/crates/move-stdlib/sources");
    
    let mut dependencies = Vec::new();
    if stdlib_path.exists() {
        // Collect stdlib sources
        for entry in std::fs::read_dir(&stdlib_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("move") {
                dependencies.push(path);
            }
        }
    }

    // Setup named addresses
    let mut named_addresses = BTreeMap::new();
    named_addresses.insert(
        Symbol::from("std"), 
        NumericalAddress::parse_str("0x1").map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?
    );
    named_addresses.insert(
        Symbol::from("system"), 
        NumericalAddress::parse_str("0x1").map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?
    );

    println!("  Found {} source files", source_files.len());
    println!("  Found {} dependency files", dependencies.len());

    compile_move_source(source_files, dependencies, named_addresses)
}
