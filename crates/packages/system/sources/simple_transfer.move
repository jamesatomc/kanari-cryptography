/// Ultra Simple Transfer Module - Works with vanilla Move VM
/// No global storage, just pure function logic
module system::simple_transfer {
    use std::vector;

    /// Transfer record
    struct Transfer has copy, drop {
        from: u64,
        to: u64,
        amount: u64,
    }

    /// Create a transfer record
    public fun create_transfer(from: u64, to: u64, amount: u64): Transfer {
        Transfer { from, to, amount }
    }

    /// Get transfer details
    public fun get_amount(transfer: &Transfer): u64 {
        transfer.amount
    }

    public fun get_from(transfer: &Transfer): u64 {
        transfer.from
    }

    public fun get_to(transfer: &Transfer): u64 {
        transfer.to
    }

    /// Calculate total from multiple transfers
    public fun total_amount(transfers: &vector<Transfer>): u64 {
        let total = 0u64;
        let len = vector::length(transfers);
        let i = 0u64;
        
        while (i < len) {
            let transfer = vector::borrow(transfers, i);
            total = total + transfer.amount;
            i = i + 1;
        };
        
        total
    }

    /// Check if amount is valid (non-zero)
    public fun is_valid_amount(amount: u64): bool {
        amount > 0
    }

    #[test]
    fun test_create_transfer() {
        let t = create_transfer(100, 200, 500);
        assert!(get_from(&t) == 100, 0);
        assert!(get_to(&t) == 200, 1);
        assert!(get_amount(&t) == 500, 2);
    }

    #[test]
    fun test_total_amount() {
        let transfers = vector::empty<Transfer>();
        vector::push_back(&mut transfers, create_transfer(1, 2, 100));
        vector::push_back(&mut transfers, create_transfer(2, 3, 200));
        vector::push_back(&mut transfers, create_transfer(3, 4, 300));
        
        assert!(total_amount(&transfers) == 600, 0);
    }
}
