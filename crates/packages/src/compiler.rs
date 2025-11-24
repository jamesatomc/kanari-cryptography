use anyhow::{Result, Context};
use move_compiler::{Compiler, Flags};
use move_command_line_common::address::NumericalAddress;
use move_symbol_pool::Symbol;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::fs;

/// Kanari Package Data - compiled Move modules
#[derive(Serialize, Deserialize)]
pub struct KanariPackage {
    pub package_name: String,
    pub modules: Vec<CompiledModuleData>,
    pub compiled_at: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CompiledModuleData {
    pub name: String,
    pub address: String,
    pub bytecode: Vec<u8>,
}

/// Compile Move package and create .rpd file
pub fn compile_package(package_dir: &Path, output_dir: &Path, version: &str, address: &str) -> Result<PathBuf> {
    println!("📦 Compiling package: {:?}", package_dir);
    
    let sources_dir = package_dir.join("sources");
    if !sources_dir.exists() {
        anyhow::bail!("Sources directory not found: {:?}", sources_dir);
    }

    // Read Move.toml to get package name
    let move_toml = package_dir.join("Move.toml");
    let package_name = if move_toml.exists() {
        let content = fs::read_to_string(&move_toml)?;
        parse_package_name(&content).unwrap_or_else(|| "unknown".to_string())
    } else {
        package_dir.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    };

    println!("  Package name: {}", package_name);

    // Collect all .move files
    let mut source_files = Vec::new();
    for entry in fs::read_dir(&sources_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("move") {
            source_files.push(path);
        }
    }

    if source_files.is_empty() {
        anyhow::bail!("No Move source files found in {:?}", sources_dir);
    }

    println!("  Found {} source files", source_files.len());

    // Setup dependencies - skip for stdlib (0x1), use stdlib for others
    let mut dependencies = Vec::new();
    // Check if address is NOT 0x1 (with any format: 0x1, 0x01, 0x0000...01)
    let is_stdlib = address.trim_start_matches("0x").trim_start_matches('0') == "1" 
                    || address == "0x1";
    
    if !is_stdlib {
        let local_stdlib = package_dir.join("../move-stdlib/sources");
        if local_stdlib.exists() {
            for entry in fs::read_dir(&local_stdlib)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("move") {
                    dependencies.push(path);
                }
            }
        }
    }

    println!("  Found {} dependency files", dependencies.len());

    // Setup named addresses
    let mut named_addresses = BTreeMap::new();
    named_addresses.insert(
        Symbol::from("std"), 
        NumericalAddress::parse_str("0x1")
            .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?
    );
    named_addresses.insert(
        Symbol::from("system"), 
        NumericalAddress::parse_str("0x2")
            .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?
    );

    // Compile Move sources
    let compiled_modules = compile_move_source(
        source_files,
        dependencies,
        named_addresses,
    )?;

    println!("  ✓ Compiled {} modules", compiled_modules.len());

    // Create Kanari package
    let package = KanariPackage {
        package_name: package_name.clone(),

        modules: compiled_modules,
        compiled_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    // Create output directory structure: output_dir/version/address/
    let version_dir = output_dir.join(version);
    let address_dir = version_dir.join(address);
    fs::create_dir_all(&address_dir)?;

    // Write .rpd file as package.rpd
    let output_file = address_dir.join("package.rpd");
    let json_data = serde_json::to_string_pretty(&package)?;
    fs::write(&output_file, json_data)?;

    println!("  ✓ Created: {:?}", output_file);

    Ok(output_file)
}

/// Compile Move source files to bytecode
fn compile_move_source(
    source_files: Vec<PathBuf>,
    dependencies: Vec<PathBuf>,
    named_addresses: BTreeMap<Symbol, NumericalAddress>,
) -> Result<Vec<CompiledModuleData>> {
    
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
        None,
        sources,
        deps,
        named_addresses,
    )
    .set_flags(flags)
    .build_and_report()
    .context("Move compilation failed")?;

    // Extract compiled modules
    let mut modules = Vec::new();
    
    for unit in compiled_units {
        let named_module = unit.into_compiled_unit();
        let module = &named_module.module;
        
        // Get module info
        let module_id = module.self_id();
        let name = module_id.name().to_string();
        let address = format!("{}", module_id.address());
        
        // Serialize bytecode
        let mut bytecode = Vec::new();
        module.serialize(&mut bytecode)
            .context("Failed to serialize module")?;
        
        modules.push(CompiledModuleData {
            name,
            address,
            bytecode,
        });
    }

    Ok(modules)
}

/// Parse package name from Move.toml
fn parse_package_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") {
            if let Some(name_part) = line.split('=').nth(1) {
                let name = name_part.trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                return Some(name);
            }
        }
    }
    None
}