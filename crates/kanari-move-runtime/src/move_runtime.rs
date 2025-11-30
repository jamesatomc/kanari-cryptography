// This file contains the MoveRuntime wrapper implementation.
// It utilizes MoveVM and InMemoryStorage for executing functions and publishing modules.

use anyhow::Result;
use move_binary_format::file_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op as MoveOp;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::{ModuleId, TypeTag};
use move_vm_runtime::move_vm::MoveVM;
use move_vm_test_utils::InMemoryStorage;
use move_vm_types::gas::UnmeteredGasMeter;

use crate::gas::{GasMeter, GasOperation};
use kanari_types::address::Address as KanariAddress;

use crate::changeset::ChangeSet;
use crate::move_vm_state::MoveVMState;

/// Simple runtime wrapper around `move-vm` for executing functions and publishing modules.
pub struct MoveRuntime {
    vm: MoveVM,
    storage: InMemoryStorage,
    state: MoveVMState,
}

impl MoveRuntime {
    /// Open the runtime using the default persistent DB path (see README).
    pub fn new() -> Result<Self> {
        let state = MoveVMState::open_default()?;
        let mut storage = InMemoryStorage::new();
        state.load_into_storage(&mut storage)?;
        // For simplicity we initialise the VM with no custom natives.
        let vm =
            MoveVM::new(vec![]).map_err(|e| anyhow::anyhow!(format!("VM init error: {:?}", e)))?;
        Ok(MoveRuntime { vm, storage, state })
    }

    /// Publish a module (bytes) with the given sender address.
    /// Returns ChangeSet containing the module addition and any resource changes from Move VM.
    pub fn publish_module(
        &mut self,
        module_bytes: Vec<u8>,
        sender: AccountAddress,
        // Optional gas tuple: (gas_limit, gas_price). If `Some`, runtime will
        // include gas accounting (debit sender, credit DAO) in the returned ChangeSet.
        gas_info: Option<(u64, u64)>,
    ) -> Result<ChangeSet> {
        let storage_clone = self.storage.clone();
        let mut session = self.vm.new_session(storage_clone);
        let mut gas = UnmeteredGasMeter;

        session
            .publish_module(module_bytes.clone(), sender, &mut gas)
            .map_err(|e| anyhow::anyhow!(format!("publish error: {:?}", e)))?;

        let (res, new_storage) = session.finish();
        let (move_changeset, events) =
            res.map_err(|e| anyhow::anyhow!(format!("finish error: {:?}", e)))?;

        let mut storage = new_storage;
        storage
            .apply(move_changeset.clone())
            .map_err(|e| anyhow::anyhow!(format!("apply error: {:?}", e)))?;

        // update our runtime storage
        self.storage = storage.clone();

        // persist module bytes to DB so they are available on next startup
        let compiled = CompiledModule::deserialize_with_defaults(&module_bytes)
            .map_err(|e| anyhow::anyhow!(format!("deserialize error: {:?}", e)))?;
        let module_id = compiled.self_id();
        self.state.save_module(&module_id, &module_bytes)?;

        // Create ChangeSet from Move VM changeset
        let mut cs = ChangeSet::new();
        cs.publish_module(sender, module_id.name().to_string());

        // If caller provided gas info, include gas accounting in the ChangeSet.
        if let Some((gas_limit, gas_price)) = gas_info {
            let mut meter = GasMeter::new(gas_limit, gas_price);
            let gas_op = GasOperation::PublishModule {
                module_size: module_bytes.len(),
            };
            meter.consume(gas_op.gas_units())?;
            let gas_cost = meter.total_cost();

            let sender_change = cs.get_or_create_change(sender);
            sender_change.increment_sequence();
            sender_change.debit(gas_cost);

            let dao_addr = AccountAddress::from_hex_literal(KanariAddress::DAO_ADDRESS)?;
            cs.collect_gas(dao_addr, gas_cost);
            cs.set_gas_used(meter.gas_used);
        }

        // Parse Move VM changeset and events
        self.parse_move_changeset(&move_changeset, &mut cs);
        self.parse_move_events(&events, &mut cs);

        Ok(cs)
    }

    /// Publish a bundle of modules atomically. This helps resolving inter-module dependencies.
    pub fn publish_module_bundle(
        &mut self,
        modules: Vec<Vec<u8>>,
        sender: AccountAddress,
    ) -> Result<()> {
        let storage_clone = self.storage.clone();
        let mut session = self.vm.new_session(storage_clone);
        let mut gas = UnmeteredGasMeter;

        session
            .publish_module_bundle(modules.clone(), sender, &mut gas)
            .map_err(|e| anyhow::anyhow!(format!("publish bundle error: {:?}", e)))?;

        let (res, new_storage) = session.finish();
        let (changeset, _events) =
            res.map_err(|e| anyhow::anyhow!(format!("finish error: {:?}", e)))?;

        let mut storage = new_storage;
        storage
            .apply(changeset)
            .map_err(|e| anyhow::anyhow!(format!("apply error: {:?}", e)))?;

        // update runtime storage
        self.storage = storage.clone();

        // persist each compiled module to DB
        for module_bytes in modules.into_iter() {
            let compiled = CompiledModule::deserialize_with_defaults(&module_bytes)
                .map_err(|e| anyhow::anyhow!(format!("deserialize error: {:?}", e)))?;
            let module_id = compiled.self_id();
            self.state.save_module(&module_id, &module_bytes)?;
        }

        Ok(())
    }

    /// Attempt to publish modules in an order that satisfies dependencies by retrying
    /// individual publishes. Each module is published with its declared `self_id().address()` as sender.
    pub fn publish_modules_ordered(&mut self, modules: Vec<Vec<u8>>) -> Result<()> {
        use std::collections::VecDeque;
        let mut queue: VecDeque<Vec<u8>> = VecDeque::from(modules);
        let mut made_progress = true;
        let mut last_err: Option<anyhow::Error> = None;

        while !queue.is_empty() && made_progress {
            made_progress = false;
            let len = queue.len();
            for _ in 0..len {
                let bytes = queue.pop_front().unwrap();
                // try to deserialize to get module address
                match CompiledModule::deserialize_with_defaults(&bytes) {
                    Ok(compiled) => {
                        let mod_id = compiled.self_id();
                        let sender = AccountAddress::from_hex_literal(&format!(
                            "0x{}",
                            mod_id.address().short_str_lossless()
                        ))
                        .unwrap_or(mod_id.address().clone());
                        let res = self.publish_module(bytes.clone(), sender, None);
                        match res {
                            Ok(_changeset) => made_progress = true,
                            Err(e) => {
                                last_err = Some(e);
                                // push back for another attempt later
                                queue.push_back(bytes);
                            }
                        }
                    }
                    Err(e) => {
                        last_err = Some(anyhow::anyhow!(format!("deserialize error: {:?}", e)));
                        // cannot determine sender, give up on this module
                    }
                }
            }
        }

        if !queue.is_empty() {
            return Err(last_err.unwrap_or_else(|| {
                anyhow::anyhow!("failed to publish modules due to unresolved dependencies")
            }));
        }
        Ok(())
    }

    /// Execute an entry function. `type_args` are Move `TypeTag`s and `args` are serialized
    /// arguments as Vec<u8> (Move simple-serialized values).
    /// Returns ChangeSet containing all state changes from Move VM execution.
    pub fn execute_entry_function(
        &mut self,
        module_id: &ModuleId,
        function_name: &str,
        type_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        // Optional sender address. If provided along with `gas_info`, runtime will debit this sender.
        sender: Option<AccountAddress>,
        // Optional gas tuple: (gas_limit, gas_price). If provided, runtime will
        // include gas accounting (debit sender if available, credit DAO) in the returned ChangeSet.
        gas_info: Option<(u64, u64)>,
    ) -> Result<ChangeSet> {
        let storage_clone = self.storage.clone();
        let mut session = self.vm.new_session(storage_clone);
        let mut gas = UnmeteredGasMeter;

        // convert type tags to VM runtime types
        let mut ty_args_loaded = vec![];
        for tag in type_args.iter() {
            let ty = session
                .load_type(tag)
                .map_err(|e| anyhow::anyhow!(format!("load type error: {:?}", e)))?;
            ty_args_loaded.push(ty);
        }

        let ident = IdentStr::new(function_name).map_err(|e| anyhow::anyhow!(e.to_string()))?;

        session
            .execute_entry_function(module_id, ident, ty_args_loaded, args, &mut gas)
            .map_err(|e| anyhow::anyhow!(format!("exec error: {:?}", e)))?;

        let (res, new_storage) = session.finish();
        let (move_changeset, events) =
            res.map_err(|e| anyhow::anyhow!(format!("finish error: {:?}", e)))?;

        let mut storage = new_storage;
        storage
            .apply(move_changeset.clone())
            .map_err(|e| anyhow::anyhow!(format!("apply error: {:?}", e)))?;

        self.storage = storage;

        // Create ChangeSet from Move VM execution
        let mut cs = ChangeSet::new();

        // Parse Move VM changeset and events
        self.parse_move_changeset(&move_changeset, &mut cs);
        self.parse_move_events(&events, &mut cs);

        // If gas accounting requested, include gas debit/credit in ChangeSet.
        if let Some((gas_limit, gas_price)) = gas_info {
            let mut meter = GasMeter::new(gas_limit, gas_price);
            // We use a default complexity of 1 when called directly via runtime.
            let gas_op = GasOperation::ExecuteFunction { complexity: 1 };
            meter.consume(gas_op.gas_units())?;
            let gas_cost = meter.total_cost();

            // If sender provided, debit them and increment sequence to prevent replay.
            if let Some(saddr) = sender {
                let sender_change = cs.get_or_create_change(saddr);
                sender_change.increment_sequence();
                sender_change.debit(gas_cost);
            }

            let dao_addr = AccountAddress::from_hex_literal(KanariAddress::DAO_ADDRESS)?;
            cs.collect_gas(dao_addr, gas_cost);
            cs.set_gas_used(meter.gas_used);
        }

        Ok(cs)
    }

    /// Parse Move VM ChangeSet and extract state changes into Kanari ChangeSet
    /// This converts Move VM's canonical state changes into our domain model
    fn parse_move_changeset(
        &self,
        move_cs: &move_core_types::effects::ChangeSet,
        kanari_cs: &mut ChangeSet,
    ) {
        for (addr, account_changes) in move_cs.accounts() {
            // Process module changes
            for (module_name, op) in account_changes.modules() {
                match op {
                    MoveOp::New(_bytes) | MoveOp::Modify(_bytes) => {
                        // Module published or updated
                        kanari_cs.publish_module(*addr, module_name.to_string());
                    }
                    MoveOp::Delete => {
                        // Module deletion (rare, but possible)
                        eprintln!(
                            "Warning: Module deletion detected for {}::{}",
                            addr, module_name
                        );
                    }
                }
            }

            // Process resource changes
            for (struct_tag, op) in account_changes.resources() {
                match op {
                    MoveOp::New(bytes) | MoveOp::Modify(bytes) => {
                        // Try to parse balance changes from Coin/Balance resources
                        // Format: 0xADDR::coin::Coin<0xADDR::kanari::KANARI>
                        if self.is_balance_resource(struct_tag) {
                            if let Some(balance) = self.extract_balance_from_bytes(bytes) {
                                // Note: This is a simplified approach
                                // In production, you'd track the delta by comparing with previous value
                                eprintln!(
                                    "Balance resource changed for {}: {} (type: {})",
                                    addr, balance, struct_tag
                                );
                            }
                        }
                    }
                    MoveOp::Delete => {
                        // Resource deletion
                        eprintln!("Resource deleted for {}: {}", addr, struct_tag);
                    }
                }
            }
        }
    }

    /// Check if struct tag represents a balance/coin resource
    fn is_balance_resource(
        &self,
        struct_tag: &move_core_types::language_storage::StructTag,
    ) -> bool {
        // Common patterns: Coin<T>, Balance<T>, Account<T>
        let name = struct_tag.name.as_str();
        name == "Coin" || name == "Balance" || name == "Account"
    }

    /// Extract u64 balance from Move BCS-encoded bytes
    /// This is a simplified parser - production code would use proper BCS deserialization
    fn extract_balance_from_bytes(&self, bytes: &[u8]) -> Option<u64> {
        // Simple u64 BCS encoding: little-endian 8 bytes
        // In real implementation, parse full struct with bcs::from_bytes
        if bytes.len() >= 8 {
            let balance_bytes: [u8; 8] = bytes[0..8].try_into().ok()?;
            Some(u64::from_le_bytes(balance_bytes))
        } else {
            None
        }
    }

    /// Parse Move VM events and add to Kanari ChangeSet
    /// Events provide an audit trail of all state changes
    fn parse_move_events(
        &self,
        events: &[move_core_types::effects::Event],
        kanari_cs: &mut ChangeSet,
    ) {
        use crate::changeset::Event;

        for (key, sequence_number, type_tag, event_data) in events {
            let kanari_event = Event {
                key: key.clone(),
                sequence_number: *sequence_number,
                type_tag: format!("{}", type_tag),
                event_data: event_data.clone(),
            };
            kanari_cs.add_event(kanari_event);
        }
    }
}
