// Copyright (c) Kanari Network
// SPDX-License-Identifier: Apache-2.0


// Package configurations for Kanari packages MoveVM
use move_core_types::account_address::AccountAddress;
use std::str::FromStr;

/// Package configuration
#[derive(Debug, Clone)]
pub struct PackageConfig {
    pub name: &'static str,
    pub directory: &'static str,
    pub address: AccountAddress,
}

/// All framework packages configuration
pub static FRAMEWORK_PACKAGES: &[(&str, &str, &str)] = &[
    // (name, directory, address)
    ("MoveStdlib", "move-stdlib", "0x1"),
    ("KanariSystem", "system", "0x2"),
    // Add more packages here as needed:
    // ("KanariFramework", "kanari-framework", "0x3"),
    // ("KanariNursery", "kanari-nursery", "0x4"),
];

/// Get package configurations
pub fn get_package_configs() -> Vec<PackageConfig> {
    FRAMEWORK_PACKAGES
        .iter()
        .filter_map(|(name, directory, address_str)| {
            AccountAddress::from_str(address_str)
                .ok()
                .map(|addr| PackageConfig {
                    name,
                    directory,
                    address: addr,
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_configs() {
        let configs = get_package_configs();
        assert!(configs.len() >= 2);
        
        let stdlib = configs.iter().find(|p| p.name == "MoveStdlib").unwrap();
        assert_eq!(stdlib.directory, "move-stdlib");
        assert_eq!(stdlib.address, AccountAddress::ONE);
        
        let system = configs.iter().find(|p| p.name == "KanariSystem").unwrap();
        assert_eq!(system.directory, "system");
        assert_eq!(format!("{}", system.address), "0x0000000000000000000000000000000000000000000000000000000000000002");
    }
}
