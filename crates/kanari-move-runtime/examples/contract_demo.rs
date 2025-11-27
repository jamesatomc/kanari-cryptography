use anyhow::Result;
use kanari_move_runtime::{
    BlockchainEngine, ContractCall, ContractDeployment, ContractMetadata,
};

fn main() -> Result<()> {
    println!("=== Kanari Contract Upload & Interaction Demo ===\n");

    // Initialize blockchain engine
    let engine = BlockchainEngine::new()?;

    println!("📦 Step 1: Deploy Smart Contract");
    println!("─────────────────────────────────────────────");

    // Prepare contract metadata
    let metadata = ContractMetadata::new(
        "MyToken".to_string(),
        "1.0.0".to_string(),
        "0x1".to_string(),
    )
    .with_description("A simple token contract for testing".to_string())
    .with_license("MIT".to_string())
    .with_tags(vec!["token".to_string(), "defi".to_string()]);

    // Example bytecode (in production, this would be compiled Move code)
    let module_bytecode = vec![
        0xa1, 0x1c, 0xeb, 0x0b, // Move magic number
        0x07, // Version 7
        // ... rest of compiled Move bytecode
    ];

    // Create deployment
    let deployment = ContractDeployment::new(
        module_bytecode,
        "my_token".to_string(),
        "0x1",
        metadata,
    )?
    .with_gas_limit(1_000_000)
    .with_gas_price(1500);

    println!("  📋 Contract Info:");
    println!("     Name: {}", deployment.metadata.name);
    println!("     Version: {}", deployment.metadata.version);
    println!("     Publisher: {}", deployment.publisher_address());
    println!("     Module: {}", deployment.module_name);
    println!("     Gas Limit: {}", deployment.gas_limit);
    println!("     Gas Price: {} Mist", deployment.gas_price);
    println!();

    // Deploy contract
    match engine.deploy_contract(deployment) {
        Ok(tx_hash) => {
            println!("  ✅ Contract deployed successfully!");
            println!("     TX Hash: {}", hex::encode(&tx_hash[..8]));
            println!();
        }
        Err(e) => {
            println!("  ❌ Deployment failed: {}", e);
            println!();
        }
    }

    println!("📊 Step 2: Check Contract Registry");
    println!("─────────────────────────────────────────────");
    
    let contract_count = engine.get_contract_count();
    println!("  Total Contracts Deployed: {}", contract_count);
    
    if let Some(contract) = engine.get_contract("0x1", "my_token") {
        println!("  ✅ Found Contract:");
        println!("     Address: {}", contract.address);
        println!("     Module: {}", contract.module_name);
        println!("     Deployed at Block: {}", contract.deployed_at);
        println!("     Bytecode Size: {} bytes", contract.bytecode.len());
    }
    println!();

    println!("📞 Step 3: Call Contract Function");
    println!("─────────────────────────────────────────────");

    // Create a contract call
    let call = ContractCall::new(
        "0x1",           // Contract address
        "my_token",      // Module name
        "mint",          // Function name
        "0x2",           // Caller address
    )?
    .with_gas_limit(200_000)
    .with_gas_price(1500);

    println!("  📋 Call Info:");
    println!("     Contract: {}::{}", call.module_address(), call.module_name());
    println!("     Function: {}", call.function);
    println!("     Caller: 0x{}", hex::encode(call.sender.to_vec()));
    println!("     Gas Limit: {}", call.gas_limit);
    println!();

    // Execute call
    match engine.call_contract(call) {
        Ok(tx_hash) => {
            println!("  ✅ Function call submitted!");
            println!("     TX Hash: {}", hex::encode(&tx_hash[..8]));
            println!();
        }
        Err(e) => {
            println!("  ❌ Call failed: {}", e);
            println!();
        }
    }

    println!("🔍 Step 4: Query Contracts");
    println!("─────────────────────────────────────────────");

    // List all contracts by address
    let contracts = engine.list_contracts_by_address("0x1");
    println!("  Contracts by address 0x1: {}", contracts.len());
    for contract in &contracts {
        println!("    • {} (v{})", contract.metadata.name, contract.metadata.version);
    }
    println!();

    // Search by tag
    let token_contracts = engine.search_contracts_by_tag("token");
    println!("  Contracts with tag 'token': {}", token_contracts.len());
    for contract in &token_contracts {
        println!("    • {}: {}", contract.module_name, contract.metadata.description);
    }
    println!();

    // List all contracts
    let all_contracts = engine.list_all_contracts();
    println!("  All Contracts: {}", all_contracts.len());
    for contract in &all_contracts {
        println!("    • {}::{}", contract.address, contract.module_name);
        println!("      Author: {}", contract.metadata.author);
        println!("      License: {:?}", contract.metadata.license);
        println!("      Tags: {:?}", contract.metadata.tags);
    }
    println!();

    println!("📊 Step 5: Blockchain Stats");
    println!("─────────────────────────────────────────────");
    
    let stats = engine.get_stats();
    println!("  Height: {}", stats.height);
    println!("  Total Blocks: {}", stats.total_blocks);
    println!("  Pending Transactions: {}", stats.pending_transactions);
    println!("  Total Contracts: {}", contract_count);
    println!();

    println!("✅ Demo Complete!");

    Ok(())
}
