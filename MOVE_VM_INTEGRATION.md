# การเชื่อมต่อ Move VM กับ Kanari Bank

## สถานะปัจจุบัน

ระบบ Kanari Bank ตอนนี้มี **2 โหมดการทำงาน**:

### 1. โหมด Simulation (ปัจจุบัน - ใช้งานได้)

- ใช้ Rust HashMap เก็บข้อมูลใน memory
- บันทึกลง JSON file เพื่อ persistence
- **ไม่ได้ใช้ Move VM จริงๆ**
- เร็วและเสถียร
- เหมาะสำหรับ testing และ development

### 2. โหมด Move VM (อยู่ระหว่างพัฒนา)

- เชื่อมต่อกับ Move VM จริง
- รัน Move bytecode จริงๆ
- ต้อง compile Move modules ก่อน
- เหมาะสำหรับ production

## โครงสร้างไฟล์

```
kanari-cp/
├── crates/
│   ├── kanari/
│   │   ├── src/
│   │   │   ├── main.rs            # CLI application
│   │   │   └── move_runtime.rs    # Move VM integration
│   │   └── Cargo.toml
│   └── packages/
│       └── system/
│           ├── sources/
│           │   ├── simple_coin.move    # Simplified coin module
│           │   ├── coin.move           # Full IOTA framework version
│           │   ├── transfer.move       # Transfer system
│           │   └── system.move
│           ├── Move.toml               # IOTA framework config
│           └── Move_simple.toml        # Standalone config
```

## การใช้งาน

### โหมด Simulation (ปัจจุบัน)

```powershell
# สร้างบัญชี
cargo run --bin kanari -- create-account --address 0x1234

# Mint เหรียญ
cargo run --bin kanari -- mint --amount 1000 --recipient 0x1234

# โอนเงิน
cargo run --bin kanari -- transfer --from 0x1234 --to 0x5678 --amount 500

# ดูยอด
cargo run --bin kanari -- balance --address 0x1234

# ดูบัญชีทั้งหมด
cargo run --bin kanari -- list
```

### การเชื่อมต่อ Move VM (Experimental)

#### ขั้นตอนที่ 1: Compile Move Modules

```powershell
cd crates/packages/system

# แบบที่ 1: ใช้ Move CLI (ถ้าติดตั้งแล้ว)
move build

# แบบที่ 2: ใช้ IOTA Move CLI
iota move build --skip-fetch-latest-git-deps

# แบบที่ 3: ใช้ผ่าน Kanari CLI
cargo run --bin kanari -- compile-move --path crates/packages/system
```

#### ขั้นตอนที่ 2: Initialize Move VM

```powershell
cargo run --bin kanari -- init-move
```

#### ขั้นตอนที่ 3: ใช้งานกับ Move VM

```powershell
# (ยังอยู่ระหว่างพัฒนา)
cargo run --bin kanari -- --use-move-vm transfer --from 0x1234 --to 0x5678 --amount 500
```

## Move Modules

### simple_coin.move (แนะนำสำหรับเริ่มต้น)

Module นี้เป็น standalone และไม่ต้องพึ่ง IOTA framework:

```move
module system::simple_coin {
    // Account balance resource
    struct Balance has key {
        value: u64
    }
    
    public fun create_account(account: &signer)
    public fun mint(account: &signer, amount: u64)
    public fun transfer(from: &signer, to: address, amount: u64)
    public fun balance(addr: address): u64
    public fun burn(account: &signer, amount: u64)
}
```

**คุณสมบัติ:**

- ✅ ไม่ต้องพึ่ง external dependencies
- ✅ Compile ได้เร็ว
- ✅ เหมาะสำหรับ testing
- ❌ ไม่มี advanced features (escrow, stream, etc.)

### coin.move + transfer.move (Full Feature)

Modules เหล่านี้ใช้ IOTA/Sui framework:

```move
module system::coin {
    // Full featured coin with treasury cap
    // Supports: mint, burn, split, join, events
}

module system::transfer {
    // Advanced transfer features:
    // - Escrow
    // - Scheduled transfers
    // - Stream transfers
    // - Batch transfers
}
```

**คุณสมบัติ:**

- ✅ Full features
- ✅ Production ready
- ❌ ต้องพึ่ง IOTA framework
- ❌ Compile ช้ากว่า

## การพัฒนาต่อ

### ที่ทำเสร็จแล้ว ✅

1. **Move Runtime Wrapper** (`move_runtime.rs`)
   - สร้าง `MoveVM` instance
   - Load compiled modules
   - Simple storage implementation
   - Function execution

2. **CLI Integration**
   - คำสั่ง `compile-move`
   - คำสั่ง `init-move`
   - Flag `--use-move-vm`

3. **Simplified Move Module** (`simple_coin.move`)
   - Standalone module
   - Basic coin operations
   - Ready to compile

### ที่กำลังทำ 🚧

1. **State Synchronization**
   - Sync ข้อมูลระหว่าง Rust HashMap และ Move VM
   - Persistent storage for Move resources

2. **Function Execution**
   - Execute Move functions จาก CLI
   - Handle arguments และ return values
   - Error handling

3. **Testing**
   - Integration tests
   - Move unit tests
   - End-to-end tests

### แผนต่อไป 📋

1. **Full Integration**
   - ใช้ Move VM เป็นหลัก
   - Remove simulation mode
   - Production deployment

2. **Advanced Features**
   - Escrow implementation
   - Stream transfers
   - Multi-signature
   - Governance

3. **Performance**
   - Caching
   - Batch operations
   - Parallel execution

## ปัญหาที่พบและวิธีแก้

### ปัญหา: Move compilation ล้มเหลว

**สาเหตุ:** IOTA framework dependencies ไม่สามารถ fetch ได้

**วิธีแก้:**

```powershell
# 1. ใช้ simple_coin.move แทน
cd crates/packages/system
mv Move.toml Move.toml.bak
mv Move_simple.toml Move.toml

# 2. หรือใช้ flag skip-fetch
iota move build --skip-fetch-latest-git-deps
```

### ปัญหา: Move VM initialization ล้มเหลว

**สาเหตุ:** ยังไม่มี compiled modules

**วิธีแก้:**

```powershell
# Compile ก่อน
cargo run --bin kanari -- compile-move

# แล้วค่อย init
cargo run --bin kanari -- init-move
```

### ปัญหา: Type mismatch ใน Move VM

**สาเหตุ:** Move VM API ต้องการ `Type` แทน `TypeTag`

**วิธีแก้:** ใช้ empty type args หรือ convert type tags

## การ Debug

### ดูข้อมูล Move modules

```powershell
# ดู compiled bytecode
ls crates/packages/system/build/*/bytecode_modules/*.mv

# ดูขนาดไฟล์
Get-ChildItem crates/packages/system/build -Recurse -Filter *.mv | Select-Object Name, Length
```

### ดู Move VM logs

```powershell
# เปิด logging
$env:RUST_LOG="debug"
cargo run --bin kanari -- init-move
```

## สรุป

**ตอนนี้ระบบทำงานได้ดีแล้วใน Simulation mode**

- ข้อมูลถาวร (persistent)
- ใช้งานง่าย
- เร็วและเสถียร

**การเชื่อมต่อ Move VM อยู่ระหว่างพัฒนา**

- Infrastructure พร้อมแล้ว
- ต้องทำ state synchronization
- ต้อง handle type conversions

**แนะนำ:**

- ใช้ Simulation mode สำหรับตอนนี้
- พัฒนา Move modules แยกต่างหาก
- Integrate ทีละส่วน

---

**Updated:** November 24, 2025
