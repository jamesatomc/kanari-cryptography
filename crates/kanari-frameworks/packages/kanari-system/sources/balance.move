/// Balance Module - จัดการยอดคงเหลือของ Kanari Coin
module kanari_system::balance {
    use std::error;

    /// Error codes
    const ERR_INSUFFICIENT_BALANCE: u64 = 1;
    const ERR_OVERFLOW: u64 = 2;

    /// Balance resource - เก็บยอดคงเหลือ (generic per token type)
    struct Balance<phantom T> has store, drop {
        value: u64,
    }

    /// Supply: mutable minting handle consumed to create balances
    struct Supply<phantom T> has store {
        total: u64,
    }

    /// สร้าง Balance ใหม่
    public fun zero<T>(): Balance<T> {
        Balance<T> { value: 0 }
    }

    /// สร้าง Balance ด้วยจำนวนเริ่มต้น
    public fun create<T>(value: u64): Balance<T> {
        Balance<T> { value }
    }

    /// ดูยอดคงเหลือ
    public fun value<T>(balance: &Balance<T>): u64 {
        balance.value
    }

    /// เพิ่มยอดคงเหลือ
    public fun increase<T>(balance: &mut Balance<T>, amount: u64) {
        let new_value = balance.value + amount;
        assert!(new_value >= balance.value, error::invalid_argument(ERR_OVERFLOW));
        balance.value = new_value;
    }

    /// ลดยอดคงเหลือ
    public fun decrease<T>(balance: &mut Balance<T>, amount: u64) {
        assert!(balance.value >= amount, ERR_INSUFFICIENT_BALANCE);
        balance.value = balance.value - amount;
    }

    /// โอนยอดจาก Balance หนึ่งไปอีก Balance หนึ่ง
    public fun transfer<T>(from: &mut Balance<T>, to: &mut Balance<T>, amount: u64) {
        decrease<T>(from, amount);
        increase<T>(to, amount);
    }

    /// ตรวจสอบว่ามียอดเพียงพอหรือไม่
    public fun has_sufficient<T>(balance: &Balance<T>, amount: u64): bool {
        balance.value >= amount
    }

    /// ทำลาย Balance และคืนค่า
    public fun destroy<T>(balance: Balance<T>): u64 {
        let Balance { value } = balance;
        value
    }

    /// Create a new (empty) supply handle
    public fun new_supply<T>(): Supply<T> {
        Supply<T> { total: 0 }
    }

    /// Increase supply: add `amount` to `s` and return a `Balance` for the newly minted amount.
    public fun increase_supply<T>(s: &mut Supply<T>, amount: u64): Balance<T> {
        let new_total = s.total + amount;
        assert!(new_total >= s.total, error::invalid_argument(ERR_OVERFLOW));
        s.total = new_total;
        create<T>(amount)
    }

    /// Decrease/destroy a supply handle (legacy)
    public fun destroy_supply<T>(s: Supply<T>) {
        let Supply { total: _ } = s;
    }

    

    /// รวม Balance สองอัน
    public fun merge<T>(dst: &mut Balance<T>, src: Balance<T>) {
        let value = destroy<T>(src);
        increase<T>(dst, value);
    }

    /// แยก Balance
    public fun split<T>(balance: &mut Balance<T>, amount: u64): Balance<T> {
        decrease<T>(balance, amount);
        create<T>(amount)
    }

    #[test]
    fun test_balance_operations() {
        let balance = create<u8>(1000);
        assert!(value(&balance) == 1000, 0);

        increase<u8>(&mut balance, 500);
        assert!(value(&balance) == 1500, 1);

        decrease<u8>(&mut balance, 300);
        assert!(value(&balance) == 1200, 2);

        let final_value = destroy<u8>(balance);
        assert!(final_value == 1200, 3);
    }

    #[test]
    fun test_transfer() {
        let balance1 = create<u8>(1000);
        let balance2 = create<u8>(500);

        transfer<u8>(&mut (balance1), &mut (balance2), 300);

        assert!(value(&balance1) == 700, 0);
        assert!(value(&balance2) == 800, 1);

        destroy<u8>(balance1);
        destroy<u8>(balance2);
    }

    #[test]
    fun test_split_merge() {
        let balance1 = create<u8>(1000);
        let balance2 = split<u8>(&mut (balance1), 400);

        assert!(value(&balance1) == 600, 0);
        assert!(value(&balance2) == 400, 1);

        merge<u8>(&mut balance1, balance2);
        assert!(value(&balance1) == 1000, 2);

        destroy<u8>(balance1);
    }

    #[test]
    #[expected_failure(abort_code = ERR_INSUFFICIENT_BALANCE)]
    fun test_insufficient_balance() {
        let balance = create<u8>(100);
        decrease<u8>(&mut (balance), 200);
        destroy<u8>(balance);
    }
}
