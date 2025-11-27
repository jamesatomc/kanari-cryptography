# Kanari Move CLI - Publish & Call Commands

คำสั่งสำหรับอัพโหลด Move modules และเรียกใช้ฟังก์ชัน

## 📦 คำสั่ง `publish`

อัพโหลด Move module ไปยัง blockchain

### การใช้งาน

```bash
kanari move publish [OPTIONS] --sender <SENDER>
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--sender <ADDRESS>` | ที่อยู่ผู้เผยแพร่ module (เช่น 0x1) | **Required** |
| `--package-path <PATH>` | Path ไปยัง Move package | Current directory |
| `--gas-limit <AMOUNT>` | Gas limit สำหรับ transaction | 1,000,000 |
| `--gas-price <PRICE>` | Gas price ในหน่วย Mist | 1,000 |
| `--private-key <KEY>` | Private key สำหรับ sign (hex string) | - |
| `--skip-signature` | ข้าม signature (สำหรับทดสอบ) | false |
| `--rpc <URL>` | RPC endpoint | <http://localhost:9944> |

### ตัวอย่าง

```bash
# Publish package ปัจจุบัน
kanari move publish --sender 0x1 --skip-signature

# Publish package จาก path อื่น
kanari move publish --sender 0x1 --package-path ./my-token --skip-signature

# Publish พร้อม signature
kanari move publish \
  --sender 0x1 \
  --private-key "abc123..." \
  --gas-limit 2000000 \
  --gas-price 1500

# Publish ไปยัง custom RPC
kanari move publish \
  --sender 0x1 \
  --skip-signature \
  --rpc http://testnet.kanari.network:9944
```

### Output

```
📦 Building Move package...
✅ Package compiled successfully!
   Modules: 3

📤 Publishing modules to blockchain...

  📝 Module: my_token
     Size: 1266 bytes
     Address: 0x1
     Functions: 10
     Estimated Gas: 72660 units
     Creating publish transaction...
     🔑 Signing transaction...
     ✅ Transaction created
     RPC: http://localhost:9944

✅ All modules published successfully!

💡 Next steps:
   • Use 'kanari move call' to execute functions
   • Check transaction status on blockchain explorer
```

## 📞 คำสั่ง `call`

เรียกใช้ฟังก์ชันใน Move module

### การใช้งาน

```bash
kanari move call [OPTIONS] --function <FUNCTION> --sender <SENDER>
```

### Options

| Option | Description | Example |
|--------|-------------|---------|
| `--function <ID>` | Function identifier: `<address>::<module>::<function>` | `0x1::coin::transfer` |
| `--sender <ADDRESS>` | ที่อยู่ผู้เรียก | `0x1` |
| `--type-args <TYPES>` | Type arguments (comma-separated) | `0x1::coin::KANARI,u64` |
| `--args <ARGS>` | Function arguments (comma-separated) | `0x123,1000` |
| `--gas-limit <AMOUNT>` | Gas limit | 200,000 |
| `--gas-price <PRICE>` | Gas price (Mist) | 1,000 |
| `--private-key <KEY>` | Private key สำหรับ sign | - |
| `--skip-signature` | ข้าม signature | false |
| `--rpc <URL>` | RPC endpoint | <http://localhost:9944> |
| `--dry-run` | Dry run (ประเมิน gas อย่างเดียว) | false |

### รูปแบบ Arguments

#### ที่อยู่ (Address)

```bash
--args "0x1"          # Short form
--args "0x0000...01"  # Full form (32 bytes)
```

#### ตัวเลข

```bash
--args "1000"         # u64
--args "1000000000000000" # u128
```

#### Boolean

```bash
--args "true"
--args "false"
```

#### Multiple Arguments

```bash
--args "0x2,1000,true"  # address, u64, bool
```

### ตัวอย่าง

#### 1. Transfer Tokens

```bash
kanari move call \
  --function "0x2::kanari::transfer" \
  --sender 0x1 \
  --args "0x2,1000" \
  --skip-signature
```

#### 2. Mint Tokens (with Type Args)

```bash
kanari move call \
  --function "0x1::coin::mint" \
  --type-args "0x1::my_token::MyCoin" \
  --sender 0x1 \
  --args "0x123,5000" \
  --skip-signature
```

#### 3. Dry Run (Estimate Gas)

```bash
kanari move call \
  --function "0x2::kanari::burn" \
  --sender 0x1 \
  --args "500" \
  --dry-run
```

#### 4. With Signature

```bash
kanari move call \
  --function "0x2::kanari::transfer" \
  --sender 0x1 \
  --args "0xabcd...,1000" \
  --private-key "your_private_key_hex" \
  --gas-limit 300000 \
  --gas-price 1500
```

#### 5. Complex Function Call

```bash
kanari move call \
  --function "0x1::dex::swap" \
  --type-args "0x1::usdc::USDC,0x2::kanari::KANARI" \
  --sender 0x1 \
  --args "1000000,900000,true" \
  --skip-signature
```

### Output

```
📞 Preparing function call...

📋 Call Details:
   Address: 0x2
   Module: kanari
   Function: transfer
   Sender: 0x1
   Gas Limit: 200000
   Gas Price: 1000
   Arguments: 2 args provided

⛽ Gas Estimation:
   Estimated: 35800 units
   Limit: 200000 units
   Total Cost: 35800000 Mist

✅ Function call submitted!

💡 Next steps:
   • Check transaction status
   • View execution results on explorer
```

## 🔐 Signature Management

### วิธีใช้ Private Key

1. **Generate Keypair** (ใช้ kanari-crypto):

```rust
use kanari_crypto::keys::{generate_keypair, CurveType};

let keypair = generate_keypair(CurveType::Ed25519)?;
println!("Address: {}", keypair.address);
println!("Private Key: {}", keypair.private_key);
```

2. **Use with CLI**:

```bash
kanari move publish \
  --sender 0x1234... \
  --private-key "abc123def456..."
```

### Test Mode (No Signature)

```bash
# สำหรับทดสอบ - ไม่ต้อง sign
kanari move publish --sender 0x1 --skip-signature
kanari move call --function "0x1::test::fn" --sender 0x1 --skip-signature
```

## ⛽ Gas Costs

### Publish Costs

- **Base**: 60,000 gas units
- **Per Byte**: 10 gas units
- **Metadata**: 5 gas units per byte

**ตัวอย่าง**:

```
Module size: 1000 bytes
Metadata size: 200 bytes
Total: 60,000 + (1000 × 10) + (200 × 5) = 71,000 gas units
Cost: 71,000 × 1000 = 71,000,000 Mist
```

### Call Costs

- **Base**: 35,000 gas units
- **Function Name**: 100 gas units per character

**ตัวอย่าง**:

```
Function: "transfer" (8 chars)
Total: 35,000 + (8 × 100) = 35,800 gas units
Cost: 35,800 × 1000 = 35,800,000 Mist
```

## 🚀 Workflow

### 1. พัฒนา Move Module

```move
// sources/my_token.move
module 0x1::my_token {
    use std::signer;
    use kanari_system::coin;
    
    struct MyToken has drop {}
    
    public entry fun initialize(admin: &signer) {
        // ...
    }
    
    public entry fun mint(admin: &signer, to: address, amount: u64) {
        // ...
    }
}
```

### 2. Build & Test

```bash
# Build
kanari move build

# Test
kanari move test
```

### 3. Publish

```bash
kanari move publish \
  --sender 0x1 \
  --skip-signature
```

### 4. Call Functions

```bash
# Initialize
kanari move call \
  --function "0x1::my_token::initialize" \
  --sender 0x1 \
  --skip-signature

# Mint
kanari move call \
  --function "0x1::my_token::mint" \
  --sender 0x1 \
  --args "0x2,1000" \
  --skip-signature
```

## 🧪 Testing & Debugging

### Dry Run

```bash
# ตรวจสอบ gas ก่อนส่ง transaction จริง
kanari move call \
  --function "0x1::expensive::compute" \
  --sender 0x1 \
  --dry-run
```

### Estimate Gas

```bash
# ดู gas estimate สำหรับ publish
kanari move publish \
  --sender 0x1 \
  --skip-signature
# Output จะแสดง "Estimated Gas" สำหรับแต่ละ module
```

## 📚 เอกสารเพิ่มเติม

- [Move Language Book](https://move-language.github.io/move/)
- [Kanari System Modules](../../kanari-frameworks/packages/kanari-system/)
- [Contract System Guide](../kanari-move-runtime/CONTRACT_GUIDE.md)

## ❓ Troubleshooting

### Error: "Invalid function identifier"

```bash
# ❌ Wrong
--function "coin::transfer"

# ✅ Correct
--function "0x1::coin::transfer"
```

### Error: "Failed to parse address"

```bash
# ใช้ 0x prefix เสมอ
--args "0x1,1000"

# Address จะถูก pad เป็น 32 bytes อัตโนมัติ
```

### Error: "Gas limit exceeded"

```bash
# เพิ่ม gas limit
--gas-limit 2000000
```

### Error: "Private key required"

```bash
# ใช้ --skip-signature สำหรับทดสอบ
--skip-signature

# หรือระบุ private key
--private-key "your_key"
```

## 🎯 Best Practices

1. **ทดสอบด้วย --dry-run ก่อน**
2. **ใช้ --skip-signature บน testnet**
3. **ตรวจสอบ gas estimates**
4. **Verify bytecode ก่อน publish**
5. **เก็บ private keys ให้ปลอดภัย**

---

**Version**: 1.0.0  
**Last Updated**: November 28, 2025
