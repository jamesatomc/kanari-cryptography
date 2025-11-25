mod compiler;
mod doc_generator;
mod packages_config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use doc_generator::{generate_documentation, PackageDocConfig};
use kanari_types::address::Address;
use packages_config::get_package_configs;
use std::{
    env,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(name = "packages")]
#[command(about = "Kanari Package Manager - Compiler & Documentation Generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile Move packages
    Build {
        /// Package version to compile (default: latest)
        #[arg(long, default_value = "latest")]
        version: String,
    },
    /// Generate documentation for Move packages
    Docs {
        /// Specific package to generate docs for (optional, generates for all if not specified)
        #[arg(long)]
        package: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let packages_dir = get_packages_dir()?;

    match cli.command {
        Commands::Build { version } => build_packages(&packages_dir, version),
        Commands::Docs { package } => generate_docs(&packages_dir, package),
    }
}

/// Get the packages directory from current working directory
fn get_packages_dir() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    Ok(if current_dir.ends_with("packages") {
        current_dir
    } else {
        current_dir.join("crates/packages")
    })
}

/// Print summary of operations
fn print_summary(operation: &str, success: usize, failed: usize) {
    println!("\n✨ {} Summary:", operation);
    println!("   ✅ Successful: {}", success);
    if failed > 0 {
        println!("   ❌ Failed: {}", failed);
    }
}

fn build_packages(packages_dir: &Path, version: String) -> Result<()> {
    println!("🚀 Kanari Package Compiler");
    println!("==========================\n");
    println!("📌 Version: {}\n", version);

    let output_dir = packages_dir.join("released");
    println!("📁 Packages: {:?}", packages_dir);
    println!("📁 Output: {:?}\n", output_dir);

    let (success, failed) = process_packages(|config| {
        let package_dir = packages_dir.join(config.directory);
        if !package_dir.exists() {
            eprintln!("⚠️  Not found: {:?}\n", package_dir);
            return Err(anyhow::anyhow!("Directory not found"));
        }

        println!("Compiling {} ({})...", config.name, config.address);
        compiler::compile_package(&package_dir, &output_dir, &version, config.address).map(|file| {
            println!("✅ {}", config.name);
            println!("   {:?}\n", file);
        })
    });

    print_summary("Compilation", success, failed);

    Ok(())
}

fn generate_docs(packages_dir: &Path, specific_package: Option<String>) -> Result<()> {
    println!("📚 Kanari Documentation Generator");
    println!("==================================\n");

    let mut doc_configs = get_doc_configs(packages_dir)?;

    if let Some(pkg_name) = &specific_package {
        doc_configs.retain(|cfg| cfg.name == *pkg_name);
        if doc_configs.is_empty() {
            eprintln!("❌ Package not found: {}", pkg_name);
            return Ok(());
        }
    }

    if doc_configs.is_empty() {
        eprintln!("❌ No packages configured");
        return Ok(());
    }

    println!("📦 Generating docs for {} package(s)\n", doc_configs.len());

    let (success, failed) = process_doc_configs(doc_configs);
    print_summary("Documentation", success, failed);

    Ok(())
}

/// Process packages with a given function
fn process_packages<F>(mut process_fn: F) -> (usize, usize)
where
    F: FnMut(&packages_config::PackageConfig) -> Result<()>,
{
    let mut success = 0;
    let mut failed = 0;

    for config in get_package_configs() {
        match process_fn(&config) {
            Ok(_) => success += 1,
            Err(e) => {
                eprintln!("❌ {}: {}\n", config.name, e);
                failed += 1;
            }
        }
    }

    (success, failed)
}

/// Process documentation configurations
fn process_doc_configs(configs: Vec<PackageDocConfig>) -> (usize, usize) {
    let mut success = 0;
    let mut failed = 0;

    for config in configs {
        match generate_documentation(&config) {
            Ok(_) => success += 1,
            Err(e) => {
                eprintln!("❌ {}: {}", config.name, e);
                failed += 1;
            }
        }
    }

    (success, failed)
}

fn get_doc_configs(packages_dir: &Path) -> Result<Vec<PackageDocConfig>> {
    let package_configs = get_package_configs();
    let mut doc_configs = Vec::new();

    for config in package_configs {
        let package_path = packages_dir.join(config.directory);
        if !package_path.exists() {
            continue;
        }

        let mut doc_config =
            PackageDocConfig::new(config.directory, package_path.to_str().unwrap());

        // Add address mapping using config method
        doc_config = doc_config.with_address(config.address_name, config.address)?;

        // Add stdlib dependency for non-stdlib packages
        if !config.is_stdlib() {
            doc_config = doc_config.with_address("std", Address::STD_ADDRESS)?;

            // Add dependency paths
            for dep in config.get_dependencies() {
                let dep_path = packages_dir.join(format!("{}/sources", dep));
                if dep_path.exists() {
                    doc_config = doc_config.with_dependency(dep_path.to_string_lossy().to_string());
                }
            }
        }

        doc_configs.push(doc_config);
    }

    Ok(doc_configs)
}
