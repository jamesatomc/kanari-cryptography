module james::james {

    use kanari_system::coin;
    use kanari_system::coin::{Coin, TreasuryCap};
    use kanari_system::tx_context::{Self, TxContext};
    use std::string;
    use std::ascii;
    use std::option;
    use kanari_system::transfer;

    const EAlreadyMinted: u64 = 0;
    /// Sender is not @0x0 the system address.
    const ENotSystemAddress: u64 = 1;

    const MAX_TOTAL_SUPPLY: u64 = 1_000_000_000; // 1 trillion JAMES

    /// Name of the coin
    struct JAMES has drop {}

    #[allow(unused_function)]
    // Register the `JAMES` Coin to acquire its `Supply`.
    // This should be called only once during genesis creation.
    // Mints the entire supply and transfers it to dev address @0x9.
    fun new(ctx: &mut TxContext): TreasuryCap<JAMES> {
        assert!(tx_context::sender(ctx) == @0x0, ENotSystemAddress);
        assert!(tx_context::epoch(ctx) == 0, EAlreadyMinted);

        let (treasury, metadata) = coin::create_currency(
            JAMES {},
            9,
            ascii::string(b"JAMES"),
            string::utf8(b"JAMES Network Coin"),
            string::utf8(b""),
            option::none(),
            ctx
        );
        transfer::public_freeze_object(metadata);

        // make a mutable binding for minting (use a different name than the original)
        let treasury_cap = treasury;

        // Mint the entire supply (in Mist) and transfer to dev @0x9
        let dev_address: address = @0x840512ff2c03135d82d55098f7461579cfe87f5c10c62718f818c0beeca138ea;
        let minted_coin: Coin<JAMES> = coin::mint(&mut treasury_cap, MAX_TOTAL_SUPPLY, ctx);
        transfer::public_transfer(minted_coin, dev_address);

        // Return the treasury cap for further authorized minting if needed
        treasury_cap
    }


    /// JAMES tokens to the treasury
    public entry fun transfer(c: coin::Coin<JAMES>, recipient: address) {
        transfer::public_transfer(c, recipient)
    }

    /// Burns JAMES tokens, decreasing total supply
    public entry fun burn(treasury_cap: &mut TreasuryCap<JAMES>, coin: Coin<JAMES>) {
        coin::burn(treasury_cap, coin);
    }
}