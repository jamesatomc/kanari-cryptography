# Move VM Integration - คู่มือการใช้งาน

## ภาพรวม

ระบบ Kanari Bank ใช้ Move VM เต็มรูปแบบสำหรับการจัดการ transfer logic โดยมีการเรียกใช้ฟังก์ชัน Move จริง ๆ

## โครงสร้าง

### 1. Move Modules (`crates/packages/system/sources/`)

```move
module system::simple_transfer {
    /// Transfer record
    struct Transfer has copy, drop {
        from: u64,
        to: u64,
        amount: u64,
    }

    /// สร้าง transfer record
    public fun create_transfer(from: u64, to: u64, amount: u64): Transfer

    /// ตรวจสอบความถูกต้องของจำนวนเงิน
    public fun is_valid_amount(amount: u64): bool
    
    /// ดึงข้อมูลจาก transfer record
    public fun get_amount(transfer: &Transfer): u64
    public fun get_from(transfer: &Transfer): u64
    public fun get_to(transfer: &Transfer): u64
}
```

### 2. System Modules (`crates/kanari-types/src/move_module.rs`)

จัดการ Module IDs และ addresses:

```rust
use kanari_types::move_module::SystemModules;

// ดึง module ID
let module_id = SystemModules::get_transfer_module_id()?;

// ดึง system address
let addr = SystemModules::system_address()?;

// ดึง custom module ID
let custom_id = SystemModules::get_module_id("0x3", "my_module")?;
```

### 3. Move Runtime (`crates/kanari/src/move_runtime.rs`)

Wrapper สำหรับ Move VM:

```rust
// สร้าง runtime
let mut runtime = MoveRuntime::new()?;

// โหลด module
let module_bytes = compile_move_source(...)?;
runtime.load_module(module_bytes)?;

// เรียกใช้ฟังก์ชัน Move
let results = runtime.execute_function(
    sender_address,
    &module_id,
    "function_name",
    vec![], // type arguments
    vec![arg1_bytes, arg2_bytes], // arguments
)?;
```

## การใช้งานหลัก

### 1. ตรวจสอบความถูกต้อง (Validation)

```rust
// เรียกใช้ Move function เพื่อ validate
let is_valid = runtime.validate_transfer(from, to, amount)?;

// ถ้า module ยังไม่โหลด จะใช้ fallback validation
// ถ้าโหลดแล้ว จะเรียก is_valid_amount() จาก Move
```

### 2. สร้าง Transfer Record

```rust
// เรียก create_transfer() จาก Move VM
let transfer_bytes = runtime.create_transfer_record(from, to, amount)?;

// transfer_bytes คือ serialized Transfer struct
```

### 3. ดึงข้อมูลจาก Transfer Record

```rust
// เรียก get_amount() จาก Move VM
let amount = runtime.get_transfer_amount(transfer_bytes)?;
```

## Flow การทำงาน

```
1. Compile Move source → bytecode
   ↓
2. Load module เข้า Move VM
   ↓
3. Execute function ผ่าน Move VM
   ↓
4. Serialize arguments ด้วย BCS
   ↓
5. รับ return values กลับมา
   ↓
6. Deserialize results
```

## ตัวอย่างการใช้งาน

### สร้างและ validate transfer

```rust
use kanari::move_runtime::MoveRuntime;
use kanari::move_vm_state::MoveVMState;

// สร้าง runtime และ state
let mut runtime = MoveRuntime::new()?;
let mut state = MoveVMState::new();

// โหลด Move modules
let package_dir = Path::new("crates/packages/system");
let module_bytes = compile_simple_package(package_dir)?;
for bytes in module_bytes {
    runtime.load_module(bytes)?;
}

// ทำ transfer (ใช้ Move VM validation)
state.transfer(
    &mut runtime,
    from_address,
    to_address,
    1000, // amount
)?;
```

## ข้อดีของการใช้ Move VM

1. **Type Safety**: Move's type system ป้องกัน bugs
2. **Formal Verification**: สามารถ verify correctness ของ logic
3. **Gas Metering**: สามารถ limit computation
4. **Resource Safety**: ป้องกัน double-spending และ resource leaks
5. **Modularity**: แยก business logic เป็น modules
6. **Upgradability**: สามารถอัพเดท modules ได้

## คำสั่ง CLI

```bash
# Compile Move package
cargo run --package kanari -- compile-move

# Initialize Move VM
cargo run --package kanari -- init-move

# Transfer (จะใช้ Move VM ถ้ามี modules โหลดแล้ว)
cargo run --package kanari -- transfer <from> <to> <amount>
```

## การทดสอบ

```bash

# รัน tests ทั้งหมด
cargo test --package kanari

# รัน Move VM tests
cargo test --package kanari move_runtime

# รัน tests ของ Move modules
cd crates/packages/system
cargo move test
```

## Serialization (BCS)

Move VM ใช้ Binary Canonical Serialization (BCS):

```rust
use bcs;

// Serialize argument
let arg_bytes = bcs::to_bytes(&amount)?;

// Deserialize result
let result: bool = bcs::from_bytes(&result_bytes)?;
```

## ข้อควรระวัง

1. **Module Loading**: ต้องโหลด modules ก่อนเรียกใช้ฟังก์ชัน
2. **Type Arguments**: ต้อง match กับ function signature
3. **Argument Serialization**: ต้องใช้ BCS format
4. **Gas Metering**: ใช้ UnmeteredGasMeter สำหรับ testing, production ควรใช้ metered
5. **Error Handling**: Move errors จะถูก propagate เป็น Rust errors

## การพัฒนาต่อ

### เพิ่ม Move Functions

1. เพิ่มฟังก์ชันใน `crates/packages/system/sources/`
2. Compile module ใหม่
3. เพิ่ม wrapper function ใน `move_runtime.rs`
4. เขียน tests

### เพิ่ม Module ใหม่

1. สร้างไฟล์ `.move` ใหม่
2. เพิ่ม module ID ใน `SystemModules`
3. Update `Move.toml` ถ้าจำเป็น
4. Compile และ test

## Resources

- [Move Language Book](https://move-language.github.io/move/)
- [Move VM Documentation](https://github.com/move-language/move/tree/main/language/move-vm)
- [BCS Specification](https://github.com/diem/bcs)
