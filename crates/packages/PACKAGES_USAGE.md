# Kanari Packages System

A configuration-based Move package compilation system for the Kanari blockchain framework.

## Overview

The Kanari Packages system provides an automated way to compile Move smart contract packages into a standardized `.rpd` (Kanari Package Data) format. The system is designed to be extensible and maintainable through a simple configuration approach.

## Architecture

```
crates/packages/
├── src/
│   ├── compiler.rs          # Core compilation logic
│   ├── packages_config.rs   # Package configuration
│   └── main.rs              # CLI entry point
├── move-stdlib/             # Move standard library (0x1)
├── system/                  # Kanari system package (0x2)
└── released/                # Compiled output directory
    └── {version}/
        ├── 0x1/
        │   └── package.rpd
        └── 0x2/
            └── package.rpd
```

## Features

- **Configuration-Based**: Add new packages by simply editing the configuration file
- **Versioned Output**: Each compilation creates version-specific outputs
- **Address Management**: Automatic address assignment and validation
- **Dependency Resolution**: Handles stdlib dependencies automatically
- **JSON Format**: Outputs packages in JSON format for easy integration

## Quick Start

### Basic Usage

Compile packages with default version (1):

```bash
cargo run --package kanari-packages --bin packages
```

Compile packages with specific version:

```bash
cargo run --package kanari-packages --bin packages -- --version 10
```

### Output Structure

Compiled packages are saved in:

```
crates/packages/released/{version}/{address}/package.rpd
```

Example:

```
released/
└── 10/
    ├── 0x1/
    │   └── package.rpd  (MoveStdlib)
    └── 0x2/
        └── package.rpd  (KanariSystem)
```

## Adding New Packages

### Step 1: Create Package Directory

Create a new directory in `crates/packages/` with the following structure:

```
your-package/
├── Move.toml
└── sources/
    └── your_module.move
```

### Step 2: Configure Move.toml

```toml
[package]
name = "YourPackage"
version = "0.0.1"

[dependencies]
MoveStdlib = { local = "../move-stdlib" }

[addresses]
your_package = "0x3"
std = "0x1"
```

### Step 3: Add to Configuration

Edit `src/packages_config.rs`:

```rust
pub static FRAMEWORK_PACKAGES: &[(&str, &str, &str)] = &[
    ("MoveStdlib", "move-stdlib", "0x1"),
    ("KanariSystem", "system", "0x2"),
    ("YourPackage", "your-package", "0x3"),  // Add this line
];
```

### Step 4: Compile

Run the compiler:

```bash
cargo run --package kanari-packages --bin packages -- --version 1
```

Your package will be compiled to:

```
released/1/0x3/package.rpd
```

## Package Configuration Reference

### PackageConfig Structure

```rust
pub struct PackageConfig {
    pub name: &'static str,      // Display name
    pub directory: &'static str, // Directory name
    pub address: AccountAddress, // Package address
}
```

### Configuration Format

```rust
FRAMEWORK_PACKAGES: &[(&str, &str, &str)] = &[
    // (Display Name, Directory Name, Address)
    ("MoveStdlib", "move-stdlib", "0x1"),
];
```

### Address Guidelines

- **0x1**: Reserved for Move standard library
- **0x2**: Reserved for Kanari system package
- **0x3+**: Available for custom packages
- Use consecutive addresses for better organization

## Package Data Format (.rpd)

The compiled `.rpd` files contain:

```json
{
  "package_name": "YourPackage",
  "version": "0.0.10",
  "modules": [
    {
      "name": "module_name",
      "address": "0x0000...0003",
      "bytecode": [/* compiled bytecode */]
    }
  ],
  "compiled_at": 1700000000
}
```

### Fields Description

- `package_name`: Name from Move.toml
- `version`: Format `0.0.{version_arg}`
- `modules`: Array of compiled modules
  - `name`: Module identifier
  - `address`: Full hex address (padded to 64 hex digits)
  - `bytecode`: Compiled Move bytecode
- `compiled_at`: Unix timestamp

## Dependencies

### Automatic Dependency Resolution

- **Stdlib (0x1)**: No dependencies (self-contained)
- **Other packages**: Automatically include stdlib as dependency

The compiler handles this automatically based on the package address.

### Dependency Detection

```rust
// In compiler.rs
let is_stdlib = address.trim_start_matches("0x")
    .trim_start_matches('0') == "1" || address == "0x1";

if !is_stdlib {
    // Include stdlib dependencies
}
```

## CLI Reference

### Command

```bash
cargo run --package kanari-packages --bin packages [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--version <VERSION>` | 1 | Version number for compiled packages |

### Examples

```bash
# Compile version 1
cargo run --package kanari-packages --bin packages

# Compile version 10
cargo run --package kanari-packages --bin packages -- --version 10

# Compile version 100
cargo run --package kanari-packages --bin packages -- --version 100
```

## Compilation Process

### 1. Discovery Phase

- Load package configurations
- Validate package directories
- Check for Move.toml files

### 2. Compilation Phase

For each package:

1. Read source files from `sources/` directory
2. Load dependencies (if not stdlib)
3. Set up named addresses
4. Invoke Move compiler
5. Serialize modules to bytecode

### 3. Output Phase

1. Create version directory
2. Create address subdirectory
3. Write `package.rpd` file

### 4. Verification

- Report compilation status
- Display summary statistics

## Error Handling

### Common Errors

#### "Package directory not found"

**Cause**: Directory specified in config doesn't exist
**Solution**: Create the directory or fix the config path

#### "unbound module: 'std::vector'"

**Cause**: Missing stdlib dependencies
**Solution**: Ensure package has correct dependency configuration

#### "Sources directory not found"

**Cause**: Missing `sources/` folder in package
**Solution**: Create `sources/` directory with Move files

#### "Invalid address"

**Cause**: Malformed address in configuration
**Solution**: Use valid hex format (e.g., "0x3")

## Best Practices

### Package Organization

1. **One package per directory**
2. **Clear naming conventions**
   - Use kebab-case for directories
   - Use PascalCase for package names
3. **Consistent versioning**
4. **Document dependencies**

### Address Management

1. **Reserve ranges**
   - 0x1: Standard library
   - 0x2: Core system
   - 0x3-0xF: Framework packages
   - 0x10+: Application packages

2. **Document allocations**
   - Keep a registry of assigned addresses
   - Comment in packages_config.rs

### Testing

Always test package compilation:

```bash
# Clean previous output
rm -rf crates/packages/released

# Compile
cargo run --package kanari-packages --bin packages -- --version 1

# Verify output
ls -la crates/packages/released/1/
```

## Integration

### Loading Packages

```rust
use crate::compiler::load_package;

// Load a compiled package
let package_path = Path::new("released/10/0x2/package.rpd");
let package = load_package(package_path)?;

println!("Package: {}", package.package_name);
println!("Version: {}", package.version);
println!("Modules: {}", package.modules.len());
```

### Extracting Bytecode

```rust
for module in &package.modules {
    println!("Module: {}", module.name);
    println!("Address: {}", module.address);
    println!("Bytecode size: {} bytes", module.bytecode.len());
}
```

## Advanced Usage

### Custom Compilation Scripts

Create a custom build script:

```bash
#!/bin/bash
# build-packages.sh

VERSIONS=(1 10 20 30)

for version in "${VERSIONS[@]}"; do
    echo "Compiling version $version..."
    cargo run --package kanari-packages --bin packages -- --version $version
done

echo "All versions compiled!"
```

### CI/CD Integration

```yaml
# .github/workflows/compile-packages.yml
name: Compile Packages

on:
  push:
    branches: [main]
    paths:
      - 'crates/packages/**'

jobs:
  compile:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - name: Compile packages
        run: |
          cargo run --package kanari-packages --bin packages -- --version ${{ github.run_number }}
      - name: Upload artifacts
        uses: actions/upload-artifact@v2
        with:
          name: packages-${{ github.run_number }}
          path: crates/packages/released/
```

## Troubleshooting

### Debug Mode

For detailed compilation logs:

```bash
RUST_LOG=debug cargo run --package kanari-packages --bin packages -- --version 1
```

### Verify Configuration

Test package discovery:

```rust
// In packages_config.rs tests
cargo test --package kanari-packages test_package_configs
```

### Clean Build

Remove all compiled packages:

```bash
rm -rf crates/packages/released
```

## Contributing

### Adding Features

1. Fork the repository
2. Create a feature branch
3. Make changes
4. Test thoroughly
5. Submit pull request

### Code Style

- Follow Rust conventions
- Use rustfmt
- Add documentation
- Include tests

## License

Apache-2.0

## Support

For issues and questions:

- GitHub Issues: [kanari-cp/issues](https://github.com/jamesatomc/kanari-cp/issues)
- Documentation: [docs/](../../../docs/)

## Changelog

### Version 0.1.0

- Initial release
- Configuration-based package system
- Support for stdlib and system packages
- Versioned compilation output
- JSON package format

---

**Copyright © 2025 Kanari Network**
