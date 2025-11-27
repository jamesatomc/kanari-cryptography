use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use move_vm_test_utils::InMemoryStorage;
use rocksdb::{IteratorMode, Options, DB};
use std::path::PathBuf;

/// Simple persistent store for published modules and small runtime state.
pub struct MoveVMState {
	db: DB,
}

impl MoveVMState {
	/// Open default DB at `~/.kari/kanari-db/move_vm_db`.
	pub fn open_default() -> Result<Self> {
		let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
		path.push(".kari");
		path.push("kanari-db");
		std::fs::create_dir_all(&path)?;
		path.push("move_vm_db");

		let mut opts = Options::default();
		opts.create_if_missing(true);
		let db = DB::open(&opts, path)?;
		Ok(MoveVMState { db })
	}

	/// Save a module blob keyed by module id.
	pub fn save_module(&self, module_id: &ModuleId, blob: &[u8]) -> Result<()> {
		let key = format!("module:{}:{}", module_id.address().to_hex_literal(), module_id.name().as_str());
		self.db.put(key.as_bytes(), blob)?;
		Ok(())
	}

	/// Load persisted modules into an `InMemoryStorage` instance.
	pub fn load_into_storage(&self, storage: &mut InMemoryStorage) -> Result<()> {
		let iter = self.db.iterator(IteratorMode::Start);
		for item in iter {
			let (key, value) = item?;
			if let Ok(s) = String::from_utf8(key.to_vec()) {
				if s.starts_with("module:") {
					// format: module:{address}:{name}
					let parts: Vec<&str> = s.splitn(3, ':').collect();
					if parts.len() == 3 {
						let addr_str = parts[1];
						let name = parts[2];
						if let Ok(addr) = AccountAddress::from_hex_literal(addr_str) {
							if let Ok(ident) = Identifier::from_utf8(name.as_bytes().to_vec()) {
								let module_id = ModuleId::new(addr, ident);
								storage.publish_or_overwrite_module(module_id, value.to_vec());
							}
						}
					}
				}
			}
		}
		Ok(())
	}
}


