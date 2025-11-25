# Kanari Bank - Full Move VM Integration

## ภาพรวม

Kanari Bank ได้รับการอัปเกรดให้ใช้ Move VM เต็มรูปแบบ โดยจัดการ Balance, Coin และ Transfer ทั้งหมดผ่าน Move modules

## โครงสร้างระบบ

### 1. Move Modules Integration

```
kanari-system/
├── balance.move      - จัดการยอดคงเหลือด้วย Move Balance module
├── coin.move         - จัดการ Coin และ Supply
├── kanari.move       - เหรียญหลัก KANARI
└── transfer.move     - ระบบโอนเงิน
```

### 2. Rust Integration Layer

```
kanari-bank/
├── move_vm_state.rs  - State management ด้วย Move Balance
├── move_runtime.rs   - Move VM execution
└── main.rs           - CLI interface
```

## คุณสมบัติหลัก

### ✅ Move Balance Operations

- **Create Balance**: สร้าง balance ใหม่ด้วย `Balance::zero()` และ `Balance::create()`
- **Increase**: เพิ่มยอดด้วย `balance.increase(amount)` พร้อม overflow protection
- **Decrease**: ลดยอดด้วย `balance.decrease(amount)` พร้อม insufficient balance check
- **Transfer**: โอนระหว่าง balance ด้วย Move VM validation

### ✅ KANARI Token Support

- **หน่วย MIST**: 1 KANARI = 1,000,000,000 MIST
- **Total Supply**: ติดตามจำนวนเหรียญทั้งหมดที่ mint
- **Formatted Display**: แสดงผลเป็น KANARI ที่อ่านง่าย

### ✅ Move VM Validation

ทุกการ transfer ผ่าน Move VM:
1. ตรวจสอบ amount ว่าถูกต้อง (> 0)
2. ตรวจสอบ from ≠ to
3. สร้าง TransferRecord ผ่าน Move
4. ตรวจสอบ balance เพียงพอ
5. ดำเนินการ decrease/increase ด้วย Move Balance

## การใช้งาน

### 1. Mint Coins (KANARI)

```bash
# Mint 5 KANARI
kanari-bank mint -a 5.0 -r <address>

# Mint 0.5 KANARI
kanari-bank mint -a 0.5 -r <address>
```

**Move Operations:**
- สร้าง `BalanceRecord::new(amount_mist)`
- เรียก `balance.increase(amount)`
- บันทึก total_supply

### 2. Transfer Coins

```bash
# Transfer 1.5 KANARI
kanari-bank signed-transfer \
  -f <from_address> \
  -t <to_address> \
  -a 1.5 \
  -p <password>
```

**Move Operations:**
1. `runtime.validate_transfer()` - ตรวจสอบด้วย Move
2. `runtime.create_transfer_record()` - สร้าง record ด้วย Move
3. `from_balance.decrease(amount)` - ลดยอดผู้ส่ง
4. `to_balance.increase(amount)` - เพิ่มยอดผู้รับ

### 3. List Wallets

```bash
kanari-bank list-wallets
```

แสดงยอด balance ทุก wallet เป็น KANARI

### 4. Wallet Info

```bash
kanari-bank wallet-info \
  -a <address> \
  -p <password> \
  --show-secrets
```

## ข้อมูลทางเทคนิค

### Balance Record Structure

```rust
pub struct BalanceRecord {
    pub value: u64,  // Amount in MIST
}

impl BalanceRecord {
    pub fn zero() -> Self
    pub fn new(value: u64) -> Self
    pub fn is_sufficient(&self, amount: u64) -> bool
    pub fn increase(&mut self, amount: u64) -> Result<()>
    pub fn decrease(&mut self, amount: u64) -> Result<()>
}
```

### Move VM State

```rust
pub struct MoveVMState {
    accounts: HashMap<String, BalanceRecord>,  // ใช้ BalanceRecord แทน u64
    transfers: Vec<TransferRecord>,
    total_supply: u64,  // ติดตามจำนวนเหรียญทั้งหมด
}
```

### ความแตกต่างจากเวอร์ชันเก่า

| Feature | เวอร์ชันเก่า | เวอร์ชันใหม่ (Move) |
|---------|------------|-------------------|
| Balance Storage | `u64` | `BalanceRecord` |
| Balance Operations | Direct arithmetic | Move Balance module |
| Overflow Protection | Manual check | Built-in `checked_add` |
| Underflow Protection | Manual check | Built-in validation |
| Transfer Validation | Basic check | Full Move VM validation |
| Total Supply | ❌ | ✅ Tracked |

## ข้อดีของการใช้ Move

### 1. **Type Safety**
- Balance เป็น type แยก ไม่ใช่แค่ u64
- Generic type `Balance<T>` รองรับหลาย token type

### 2. **Safety Guarantees**
- ป้องกัน overflow/underflow อัตโนมัติ
- ตรวจสอบ balance เพียงพอก่อน transfer
- ไม่สามารถสร้าง balance ติดลบได้

### 3. **Move VM Validation**
- ทุก transaction validated ด้วย Move
- ป้องกันการโอนไปที่อยู่เดียวกัน
- ตรวจสอบ amount > 0

### 4. **Scalability**
- สามารถเพิ่ม token type ใหม่ได้ง่าย
- รองรับ multi-token system
- Compatible กับ Sui/Aptos ecosystem

## การทดสอบ

```bash
# 1. Reset data
kanari-bank reset --confirm

# 2. Mint coins
kanari-bank mint -a 10.0 -r 0x...

# 3. List balances
kanari-bank list-wallets

# 4. Transfer
kanari-bank signed-transfer -f 0x... -t 0x... -a 2.5 -p password
```

## Best Practices

### 1. ใช้ Move Balance Operations
```rust
// ✅ Good - ใช้ Move Balance
balance.increase(amount)?;
balance.decrease(amount)?;

// ❌ Bad - Direct manipulation
balance.value += amount;
```

### 2. ตรวจสอบผ่าน Move VM
```rust
// ✅ Good - Validate with Move
runtime.validate_transfer(&from, &to, amount)?;

// ❌ Bad - Skip validation
if from != to && amount > 0 { ... }
```

### 3. จัดการ Errors
```rust
// ✅ Good - Handle Result
balance.increase(amount)
    .context("Failed to increase balance")?;

// ❌ Bad - Unwrap
balance.increase(amount).unwrap();
```

## Roadmap

### Phase 1: ✅ Complete
- [x] Move Balance integration
- [x] KANARI token support
- [x] Move VM validation
- [x] CLI interface

### Phase 2: 🔄 In Progress
- [ ] Coin module integration
- [ ] Multi-token support
- [ ] Staking/rewards

### Phase 3: 📋 Planned
- [ ] Smart contract deployment
- [ ] DeFi features
- [ ] Cross-chain bridge

## อ้างอิง

- [Move Language Docs](https://move-language.github.io/move/)
- [Sui Move Docs](https://docs.sui.io/guides/developer/first-app/write-package)
- [Kanari Types Documentation](../kanari-types/README.md)
- [Move VM Usage Guide](MOVE_VM_USAGE.md)
