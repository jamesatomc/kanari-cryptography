use anyhow::Result;
use kanari_crypto::keys::{generate_keypair, CurveType};
use kanari_move_runtime::{
    BlockchainEngine, ContractCall, ContractDeployment, ContractMetadata, SignedTransaction,
    Transaction,
};
use move_core_types::account_address::AccountAddress;

fn main() -> Result<()> {
    println!("=== Kanari Contract Upload & Interaction Demo (with Signing) ===\n");

    // Initialize blockchain engine
    let engine = BlockchainEngine::new()?;

    // Generate keypairs for testing
    println!("🔑 Generating Keypairs...");
    let publisher_keypair = generate_keypair(CurveType::Ed25519)?;
    let caller_keypair = generate_keypair(CurveType::Ed25519)?;
    
    println!("  Publisher: {}", publisher_keypair.address);
    println!("  Caller: {}", caller_keypair.address);
    println!();

    println!("📦 Step 1: Deploy Smart Contract");
    println!("─────────────────────────────────────────────");

    // Prepare contract metadata
    let metadata = ContractMetadata::new(
        "MyToken".to_string(),
        "1.0.0".to_string(),
        publisher_keypair.address.clone(),
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

    println!("  📋 Contract Info:");
    println!("     Name: MyToken");
    println!("     Version: 1.0.0");
    println!("     Publisher: {}", publisher_keypair.address);
    println!("     Module: my_token");
    println!();

    // Create and sign deployment transaction
    let tx = Transaction::PublishModule {
        sender: publisher_keypair.address.clone(),
        module_bytes: module_bytecode.clone(),
        module_name: "my_token".to_string(),
        gas_limit: 1_000_000,
        gas_price: 1500,
    };

    let mut signed_tx = SignedTransaction::new(tx);
    signed_tx.sign(&publisher_keypair.private_key, CurveType::Ed25519)?;

    match engine.submit_transaction(signed_tx.clone()) {
        Ok(tx_hash) => {
            println!("  ✅ Contract transaction submitted!");
            println!("     TX Hash: {}", hex::encode(&tx_hash[..8]));
            
            // Manually register contract (since we're not producing blocks)
            let contract_info = kanari_move_runtime::ContractInfo {
                address: publisher_keypair.address.clone(),
                module_name: "my_token".to_string(),
                bytecode: module_bytecode.clone(),
                deployment_tx: tx_hash.clone(),
                deployed_at: 0,
                abi: kanari_move_runtime::ContractABI::new(),
                metadata: metadata.clone(),
            };
            
            engine.contract_registry.write().unwrap().register(contract_info);
            println!("     Registered in contract registry");
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
    
    if let Some(contract) = engine.get_contract(&publisher_keypair.address, "my_token") {
        println!("  ✅ Found Contract:");
        println!("     Address: {}", contract.address);
        println!("     Module: {}", contract.module_name);
        println!("     Name: {}", contract.metadata.name);
        println!("     Version: {}", contract.metadata.version);
        println!("     License: {:?}", contract.metadata.license);
        println!("     Tags: {:?}", contract.metadata.tags);
        println!("     Bytecode Size: {} bytes", contract.bytecode.len());
    } else {
        println!("  ⚠️  Contract not found in registry");
    }
    println!();

    println!("📞 Step 3: Call Contract Function");
    println!("─────────────────────────────────────────────");

    // Prepare function call arguments
    let recipient = bcs::to_bytes(&AccountAddress::from_hex_literal(&caller_keypair.address)?)?;
    let amount = bcs::to_bytes(&1000u64)?;

    let tx = Transaction::ExecuteFunction {
        sender: publisher_keypair.address.clone(),
        module: publisher_keypair.address.clone(),
        function: "mint".to_string(),
        type_args: vec![],
        args: vec![recipient, amount],
        gas_limit: 200_000,
        gas_price: 1500,
    };

    println!("  📋 Call Info:");
    println!("     Contract: {}::my_token", publisher_keypair.address);
    println!("     Function: mint");
    println!("     Caller: {}", publisher_keypair.address);
    println!("     Gas Limit: 200000");
    println!();

    // Sign and submit
    let mut signed_tx = SignedTransaction::new(tx);
    signed_tx.sign(&publisher_keypair.private_key, CurveType::Ed25519)?;

    match engine.submit_transaction(signed_tx) {
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
    let contracts = engine.list_contracts_by_address(&publisher_keypair.address);
    println!("  Contracts by address {}: {}", publisher_keypair.address, contracts.len());
    for contract in &contracts {
        println!("    • {} (v{})", contract.metadata.name, contract.metadata.version);
        println!("      Description: {}", contract.metadata.description);
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
        println!("    📦 {}::{}", contract.address, contract.module_name);
        println!("       Name: {}", contract.metadata.name);
        println!("       Author: {}", contract.metadata.author);
        println!("       License: {:?}", contract.metadata.license);
        println!("       Tags: {:?}", contract.metadata.tags);
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

    println!("⛽ Step 6: Gas Estimation");
    println!("─────────────────────────────────────────────");
    
    use kanari_move_runtime::GasOperation;
    
    let deploy_gas = GasOperation::ContractDeployment {
        module_size: module_bytecode.len(),
        metadata_size: 200,
    };
    println!("  Contract Deployment Gas: {} units", deploy_gas.gas_units());
    println!("  Cost: {} Mist", deploy_gas.gas_units() * 1500);
    
    let call_gas = GasOperation::ContractCall {
        function_name_len: 4, // "mint"
    };
    println!("  Contract Call Gas: {} units", call_gas.gas_units());
    println!("  Cost: {} Mist", call_gas.gas_units() * 1500);
    println!();

    println!("✅ Demo Complete!");
    println!("\n💡 Tips:");
    println!("  • ใช้ move-cli compile เพื่อสร้าง bytecode จริง");
    println!("  • เพิ่ม ABI เพื่อ type-safe function calls");
    println!("  • ใช้ produce_block() เพื่อ execute transactions");
    println!("  • ตรวจสอบ gas costs ก่อน deploy");

    Ok(())
}
