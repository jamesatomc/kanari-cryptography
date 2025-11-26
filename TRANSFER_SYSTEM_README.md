# Kanari Transfer System

ระบบโอนเงินแบบครบวงจรที่พัฒนาด้วย Move Language และ Rust

## โครงสร้างโปรเจกต์

```
kanari-cp/
├── crates/
│   ├── kanari/               # CLI Application (Rust)
│   ├── kanari-common/        # Shared utilities
│   ├── kanari-crypto/        # Cryptographic functions
│   ├── kanari-types/         # Type definitions
│   └── packages/
│       └── system/           # Move smart contracts
│           ├── sources/
│           │   ├── coin.move      # Token management
│           │   ├── transfer.move  # Transfer system
│           │   └── system.move    # System module
│           └── tests/
│               ├── coin_tests.move
│               └── transfer_tests.move
└── third_party/move/         # Move language runtime
```

## คุณสมบัติหลัก

### 1. Coin Module (`coin.move`)

- **Account Management**: สร้างและจัดการบัญชีผู้ใช้
- **Minting/Burning**: สร้างและทำลาย tokens
- **Deposit/Withdraw**: ฝากและถอนเงิน
- **Transfer**: โอนเงินระหว่างบัญชี
- **Split/Join**: แบ่งและรวม coins

### 2. Transfer Module (`transfer.move`)

- **Escrow**: ฝากเงินแบบมีเงื่อนไข สามารถยกเลิกได้
- **Scheduled Transfer**: โอนเงินตามเวลาที่กำหนด
- **Stream Transfer**: โอนเงินแบบค่อยๆ ปล่อยตามเวลา
- **Batch Transfer**: โอนเงินให้หลายที่อยู่พร้อมกัน
- **Multi-Signature**: โอนเงินที่ต้องการลายเซ็นหลายคน

### 3. Kanari CLI (`kanari`)

CLI application สำหรับจัดการบัญชีและโอนเงิน

## การติดตั้งและใช้งาน

### ข้อกำหนดเบื้องต้น

- Rust (1.70+)
- Move CLI (IOTA/Sui version)
- Git

### ติดตั้ง

```powershell
# Clone repository
git clone <repository-url>
cd kanari-cp

# Build project
cargo build --release
```

### คำสั่งพื้นฐาน

#### 1. สร้างบัญชี

```powershell
cargo run --bin kanari -- create-account --address 0x1234567890abcdef
```

#### 2. Mint เหรียญ

### 3. Kanari CLI (`kanari`)

```powershell
cargo run --bin kanari -- mint --amount 1000 --recipient 0x1234567890abcdef
```

#### 3. โอนเงิน

```powershell
### คำสั่งพื้นฐาน
    --from 0x1234567890abcdef `
    --to 0xfedcba0987654321 `
    --amount 500
```

cargo run --bin kanari -- create-account --address 0x1234567890abcdef

#### 4. ตรวจสอบยอดเงิน

```powershell
cargo run --bin kanari -- balance --address 0x1234567890abcdef
```

cargo run --bin kanari -- mint --amount 1000 --recipient 0x1234567890abcdef

#### 5. สร้าง Escrow

```powershell
cargo run --bin kanari -- escrow `
    --from 0x1234567890abcdef `
cargo run --bin kanari -- transfer `
    --amount 300
```

#### 6. โอนเงินแบบ Batch

```powershell
cargo run --bin kanari -- batch-transfer `
    --from 0x1234567890abcdef `
cargo run --bin kanari -- balance --address 0x1234567890abcdef
    --amounts "100,200,300"
```

## การ Build และ Test Move Modules

cargo run --bin kanari -- escrow `

```powershell
cd crates/packages/system
iota move build
```

### Run Move tests

cargo run --bin kanari -- batch-transfer `
iota move test

```

### Run specific test

```powershell
iota move test coin_tests
iota move test transfer_tests
```

## Architecture

### Move Smart Contracts

Move modules ทำงานบน IOTA/Sui blockchain โดยใช้:

- **Objects**: สำหรับเก็บ state (Account, Escrow, etc.)
- **Capabilities**: สำหรับควบคุมการ mint/burn
- **Events**: สำหรับ tracking การทำธุรกรรม

### Rust Integration

Rust CLI เชื่อมต่อกับ Move VM ผ่าน:

- `move-core-types`: type definitions
- `move-vm-runtime`: VM execution
- Local state management

## ตัวอย่างการใช้งาน

### Example 1: Basic Transfer

```powershell
# สร้างบัญชีผู้ใช้
kanari create-account --address 0xAlice
kanari create-account --address 0xBob

# Mint ให้ Alice
kanari mint --amount 1000 --recipient 0xAlice

# Transfer from Alice to Bob
kanari transfer --from 0xAlice --to 0xBob --amount 500

# ตรวจสอบยอด
kanari balance --address 0xBob    // 500
```

kanari mint --amount 1000 --recipient 0xAlice

### Example 2: Scheduled Transfer (ใน Move)

kanari transfer --from 0xAlice --to 0xBob --amount 500
// Create scheduled transfer that unlocks after 1 day
let coin = coin::mint(&mut treasury_cap, 1000, ctx);
kanari balance --address 0xAlice  // 500
kanari balance --address 0xBob    // 500
    recipient_address,
    clock::timestamp_ms(clock) + 86400000, // +1 day
    ctx
);

// Recipient claims after unlock time
let claimed_coin = transfer::claim_scheduled_transfer(
    &mut scheduled,
    clock,
    ctx
);

```

### Example 3: Stream Transfer

```move
// Stream 1000 coins over 10 days
let stream = transfer::create_stream_transfer(
    coin,
    recipient,
    start_time,
    start_time + (86400000 * 10), // 10 days
    ctx
);

// Recipient can claim gradually
let claimable = transfer::stream_claimable_amount(&stream, clock);
let claimed = transfer::claim_stream(&mut stream, clock, ctx);
```

## Security Features

- ✅ **Type Safety**: Move's strong type system
- ✅ **Resource Safety**: Assets cannot be duplicated or lost
- ✅ **Capability-based**: Only treasury holder can mint
- ✅ **Event Logging**: All transfers emit events
- ✅ **Time-locks**: Scheduled transfers with unlock times
- ✅ **Escrow Protection**: Safe conditional transfers

## Testing

### Unit Tests (Move)

```powershell
cd crates/packages/system
iota move test --coverage
```

### Integration Tests (Rust)

```powershell
cargo test --package kanari
```

### Run all tests

```powershell
# Move tests
cd crates/packages/system ; iota move test

cargo test --package kanari
cargo test --workspace
```

## การพัฒนาต่อ

### เพิ่ม Feature ใหม่

1. เพิ่มฟังก์ชันใน Move module (`sources/*.move`)
2. เขียน tests (`tests/*.move`)
3. อัปเดต Rust CLI (`kanari/src/main.rs`)
4. Build และ test

### Roadmap

- [ ] Multi-signature transfers
- [ ] Governance module
- [ ] Staking rewards
- [ ] Cross-chain bridges

3. อัปเดต Rust CLI (`kanari/src/main.rs`)

- [ ] Mobile app integration

## License

MIT License - see LICENSE file

## Contributors

Kanari Development Team

## Support

For issues and questions:

- GitHub Issues: <repository-url>/issues
- Documentation: ./docs/

---

**หมายเหตุ**: โปรเจกต์นี้อยู่ในระหว่างการพัฒนา API อาจมีการเปลี่ยนแปลง
