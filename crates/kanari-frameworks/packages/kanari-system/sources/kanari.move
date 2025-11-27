/// Coin<KANARI> is the token used to pay for gas in KANARI.
/// It has 9 decimals, and the smallest unit (10^-9) is called "KA".
module kanari_system::kanari {
    use std::option;
    use std::vector;
    use std::ascii;
    use std::string;
    use kanari_system::balance;
    use kanari_system::balance::{Balance};
    use kanari_system::tx_context;
    use kanari_system::tx_context::TxContext;
    use kanari_system::transfer;
    use kanari_system::coin;
    use kanari_system::coin::{Coin, TreasuryCap};

    const EAlreadyMinted: u64 = 0;
    /// Sender is not @0x0 the system address.
    const ENotSystemAddress: u64 = 1;
    /// Exceeded maximum supply
    const EMAX_SUPPLY: u64 = 2;

    #[allow(unused_const)]
    /// The amount of Mist per Kanari token based on the fact that mist is
    /// 10^-9 of a Kanari token
    const MIST_PER_KANARI: u64 = 1_000_000_000;

    #[allow(unused_const)]
    /// The total supply of Kanari denominated in whole Kanari tokens (10 Billion)
    const TOTAL_SUPPLY_KANARI: u64 = 10_000_000_000;

    /// The total supply of Kanari denominated in Mist (10 Billion * 10^9)
    const TOTAL_SUPPLY_MIST: u64 = 10_000_000_000_000_000_000;

    /// Name of the coin
    struct KANARI has drop {}

    #[allow(unused_function)]
    // Register the `KANARI` Coin to acquire its `Supply`.
    // This should be called only once during genesis creation.
    // Returns the `TreasuryCap` so the caller can mint from the same treasury.
    fun new(ctx: &mut TxContext): (TreasuryCap<KANARI>, Balance<KANARI>) {
        assert!(tx_context::sender(ctx) == @0x0, ENotSystemAddress);
        assert!(tx_context::epoch(ctx) == 0, EAlreadyMinted);

        let (treasury, metadata) = coin::create_currency(
            KANARI {},
            9,
            ascii::string(b"KANARI"),
            string::utf8(b"Kanari Network Coin"),
            string::utf8(b""),
            option::none(),
            ctx
        );
        transfer::public_freeze_object(metadata);
        let supply = coin::treasury_into_supply(&mut (treasury));
        let total_KANARI = balance::increase_supply(&mut (supply), TOTAL_SUPPLY_MIST);
        balance::destroy_supply(supply);
        (treasury, total_KANARI)
    }

    /// Genesis initializer that creates KANARI and distributes total supply
    /// according to the provided `recipients` and `amounts` vectors.
    /// The vectors must have the same length and the sum of `amounts`
    /// must equal `TOTAL_SUPPLY_MIST`.
    /// This is an entry function and must be called by the genesis
    /// transaction (sender @0x0, epoch 0).
    public entry fun init_genesis(
        ctx: &mut TxContext,
        recipients: vector<address>,
        amounts: vector<u64>,
    ) {
        assert!(tx_context::sender(ctx) == @0x0, ENotSystemAddress);
        assert!(tx_context::epoch(ctx) == 0, EAlreadyMinted);

        // use the registered currency / treasury created by `new`
        let (treasury, _balance) = new(ctx);

        let len_recip = vector::length(&recipients);
        let len_amt = vector::length(&amounts);
        // lengths must match
        assert!(len_recip == len_amt, EAlreadyMinted);

        // sum amounts and check equals total supply
        let i = 0u64;
        let sum = 0u64;
        while (i < len_amt) {
            let a = *vector::borrow(&amounts, i);
            sum = sum + a;
            // overflow check
            assert!(sum >= a, EMAX_SUPPLY);
            i = i + 1;
        };
        assert!(sum == TOTAL_SUPPLY_MIST, EMAX_SUPPLY);

        // mint to each recipient according to amounts
        let j = 0u64;
        while (j < len_recip) {
            let amt = *vector::borrow(&amounts, j);
            let recip = *vector::borrow(&recipients, j);
            coin::mint_and_transfer(&mut (treasury), amt, recip, ctx);
            j = j + 1;
        };
    }

    /// KANARI tokens to the treasury
    public entry fun transfer(c: coin::Coin<KANARI>, recipient: address) {
        transfer::public_transfer(c, recipient)
    }

    /// Mint `amount` of KANARI and send to `recipient`.
    /// Caller must supply the `TreasuryCap<KANARI>` (authorization to mint).
    public entry fun mint_to(
        treasury_cap: &mut TreasuryCap<KANARI>,
        amount: u64,
        recipient: address,
        ctx: &mut TxContext,
    ) {
        let current = coin::total_supply(treasury_cap);
        let new_total = current + amount;
        // check overflow and max supply
        assert!(new_total >= current, EMAX_SUPPLY);
        assert!(new_total <= TOTAL_SUPPLY_MIST, EMAX_SUPPLY);
        coin::mint_and_transfer(treasury_cap, amount, recipient, ctx);
    }

    /// Mint `amount` of KANARI and return as a `Balance<KANARI>` to the caller.
    /// Caller must supply the `TreasuryCap<KANARI>`.
    public entry fun mint_balance(
        treasury_cap: &mut TreasuryCap<KANARI>,
        amount: u64,
        ctx: &mut TxContext,
    ): Balance<KANARI> {
        let current = coin::total_supply(treasury_cap);
        let new_total = current + amount;
        assert!(new_total >= current, EMAX_SUPPLY);
        assert!(new_total <= TOTAL_SUPPLY_MIST, EMAX_SUPPLY);
        let c = coin::mint(treasury_cap, amount, ctx);
        coin::into_balance(c)
    }

    /// Burns KANARI tokens, decreasing total supply
    public entry fun burn(treasury_cap: &mut TreasuryCap<KANARI>, coin: Coin<KANARI>) {
        coin::burn(treasury_cap, coin);
    }
}