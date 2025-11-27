// This file contains the MoveRuntime wrapper implementation.
// It utilizes MoveVM and InMemoryStorage for executing functions and publishing modules.

use anyhow::Result;
use move_binary_format::file_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::{ModuleId, TypeTag};
use move_vm_runtime::move_vm::MoveVM;
use move_vm_test_utils::InMemoryStorage;
use move_vm_types::gas::UnmeteredGasMeter;
use move_core_types::runtime_value::{MoveValue, MoveStruct};
use move_core_types::account_address::AccountAddress as MoveAccountAddress;
use move_core_types::identifier::Identifier as MoveIdentifier;
use move_core_types::language_storage::ModuleId as MoveModuleId;

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
		let vm = MoveVM::new(vec![]).map_err(|e| anyhow::anyhow!(format!("VM init error: {:?}", e)))?;
		Ok(MoveRuntime { vm, storage, state })
	}

	/// Run genesis by calling `kanari_system::genesis::init` with a fabricated TxContext
	/// where sender = @0x0, tx_hash = zeros (32 bytes), epoch = 0, epoch_timestamp_ms = 0, ids_created = 0
	pub fn run_genesis(&mut self) -> Result<()> {
		// build a MoveValue::Struct representing TxContext { sender, tx_hash, epoch, epoch_timestamp_ms, ids_created }
		let sender = MoveAccountAddress::ZERO;
		let tx_hash_bytes = vec![0u8; 32];
		let tx_hash_val = MoveValue::vector_u8(tx_hash_bytes);
		let fields = vec![
			MoveValue::Address(sender),
			tx_hash_val,
			MoveValue::U64(0),
			MoveValue::U64(0),
			MoveValue::U64(0),
		];
		let txctx = MoveValue::Struct(MoveStruct::new(fields));
		let arg = txctx.simple_serialize().ok_or_else(|| anyhow::anyhow!("failed to serialize TxContext"))?;

		// prepare session and execute
		let storage_clone = self.storage.clone();
		let mut session = self.vm.new_session(storage_clone);
		let mut gas = UnmeteredGasMeter;

		// build module id for kanari_system::genesis
		let addr = MoveAccountAddress::from_hex_literal("0x2").map_err(|e| anyhow::anyhow!(e.to_string()))?;
		let name = MoveIdentifier::new("genesis").map_err(|e| anyhow::anyhow!(e.to_string()))?;
		let module_id = MoveModuleId::new(addr, name);

		let ident = move_core_types::identifier::IdentStr::new("init").map_err(|e| anyhow::anyhow!(e.to_string()))?;

		session.execute_entry_function(&module_id, ident, vec![], vec![arg], &mut gas)
			.map_err(|e| anyhow::anyhow!(format!("genesis exec error: {:?}", e)))?;

		let (res, new_storage) = session.finish();
		let (changeset, _events) = res.map_err(|e| anyhow::anyhow!(format!("finish error: {:?}", e)))?;

		let mut storage = new_storage;
		storage.apply(changeset).map_err(|e| anyhow::anyhow!(format!("apply error: {:?}", e)))?;
		self.storage = storage;
		Ok(())
	}

	/// Publish a module (bytes) with the given sender address.
	/// The module is applied to the in-memory storage and persisted to RocksDB.
	pub fn publish_module(&mut self, module_bytes: Vec<u8>, sender: AccountAddress) -> Result<()> {
		let storage_clone = self.storage.clone();
		let mut session = self.vm.new_session(storage_clone);
		let mut gas = UnmeteredGasMeter;

		session
			.publish_module(module_bytes.clone(), sender, &mut gas)
			.map_err(|e| anyhow::anyhow!(format!("publish error: {:?}", e)))?;

		let (res, new_storage) = session.finish();
		let (changeset, _events) = res.map_err(|e| anyhow::anyhow!(format!("finish error: {:?}", e)))?;

		let mut storage = new_storage;
		storage
			.apply(changeset)
			.map_err(|e| anyhow::anyhow!(format!("apply error: {:?}", e)))?;

		// update our runtime storage
		self.storage = storage.clone();

		// persist module bytes to DB so they are available on next startup
		let compiled = CompiledModule::deserialize_with_defaults(&module_bytes)
			.map_err(|e| anyhow::anyhow!(format!("deserialize error: {:?}", e)))?;
		let module_id = compiled.self_id();
		self.state.save_module(&module_id, &module_bytes)?;

		Ok(())
	}

	/// Execute an entry function. `type_args` are Move `TypeTag`s and `args` are serialized
	/// arguments as Vec<u8> (Move simple-serialized values).
	pub fn execute_entry_function(
		&mut self,
		module_id: &ModuleId,
		function_name: &str,
		type_args: Vec<TypeTag>,
		args: Vec<Vec<u8>>,
	) -> Result<()> {
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
		let (changeset, _events) = res.map_err(|e| anyhow::anyhow!(format!("finish error: {:?}", e)))?;

		let mut storage = new_storage;
		storage
			.apply(changeset)
			.map_err(|e| anyhow::anyhow!(format!("apply error: {:?}", e)))?;

		self.storage = storage;
		Ok(())
	}
}

