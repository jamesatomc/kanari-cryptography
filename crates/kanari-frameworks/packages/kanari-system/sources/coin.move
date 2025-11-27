module kanari_system::coin {
    use std::option;
    use std::string;
    use std::ascii;
    use kanari_system::url;
    use kanari_system::object;
    use kanari_system::balance::{Self, Balance, Supply};
    use kanari_system::tx_context::TxContext;
    use kanari_system::transfer;
    
    /// Coin resource wrapper with balance
    struct Coin<phantom T> has store, drop {
        balance: Balance<T>,
    }

    /// Capability allowing the bearer to mint and burn coins
    struct TreasuryCap<phantom T> has store, drop {
        id: object::UID,
        total_supply: u64,
    }

    /// Treasury: holds authority to mint into a Supply (deprecated, use TreasuryCap)
    struct Treasury<phantom T> has store, drop {
    }



    /// Metadata resource for a currency (stored as an object with UID)
    struct CoinMetadata<phantom T> has key, store, drop {
        id: object::UID,
        decimals: u8,
        name: string::String,
        symbol: ascii::String,
        description: string::String,
        icon_url: option::Option<url::Url>,
    }

    /// Error codes (local to coin module)
    const ERR_OVERFLOW: u64 = 2;

    /// Create a new currency with TreasuryCap for minting control
    public fun create_currency<T: drop>(
        witness: T,
        decimals: u8,
        symbol: ascii::String,
        name: string::String,
        description: string::String,
        icon_url: option::Option<url::Url>,
        ctx: &mut TxContext,
    ): (TreasuryCap<T>, CoinMetadata<T>) {
        // Token witness is consumed automatically as it has drop ability
        let _ = witness;
        let _ = decimals;
        let _ = icon_url;
        let _ = ctx;
        
        (
            TreasuryCap { id: object::new(ctx), total_supply: 0 },
            CoinMetadata { id: object::new(ctx), decimals, name, symbol, description, icon_url },
        )
    }

    /// Mint new coins using TreasuryCap
    public fun mint<T>(
        cap: &mut TreasuryCap<T>,
        amount: u64,
        _ctx: &mut TxContext,
    ): Coin<T> {
        let new_total = cap.total_supply + amount;
        assert!(new_total >= cap.total_supply, ERR_OVERFLOW);
        cap.total_supply = new_total;
        Coin {
            balance: balance::create(amount),
        }
    }

    /// Mint and transfer to recipient
    public fun mint_and_transfer<T>(
        cap: &mut TreasuryCap<T>,
        amount: u64,
        recipient: address,
        ctx: &mut TxContext,
    ) {
        let coin = mint(cap, amount, ctx);
        transfer::public_transfer(coin, recipient);
    }

    /// Burn coins, decreasing total supply
    public fun burn<T>(cap: &mut TreasuryCap<T>, coin: Coin<T>): u64 {
        let Coin { balance } = coin;
        let value = balance::destroy(balance);
        assert!(cap.total_supply >= value, ERR_OVERFLOW);
        cap.total_supply = cap.total_supply - value;
        value
    }

    /// Convert a `Coin<T>` into its inner `Balance<T>`.
    /// This helper is provided so other modules can obtain the balance
    /// without attempting to destructure `Coin<T>` directly (not allowed outside this module).
    public fun into_balance<T>(coin: Coin<T>): Balance<T> {
        let Coin { balance } = coin;
        balance
    }

    /// Get total supply from TreasuryCap
    public fun total_supply<T>(cap: &TreasuryCap<T>): u64 {
        cap.total_supply
    }

    /// Get coin value
    public fun value<T>(coin: &Coin<T>): u64 {
        balance::value(&coin.balance)
    }

    /// Split a coin into two
    public fun split<T>(coin: &mut Coin<T>, amount: u64, ctx: &mut TxContext): Coin<T> {
        let _ = ctx;
        Coin {
            balance: balance::split(&mut coin.balance, amount),
        }
    }

    /// Join two coins together
    public fun join<T>(coin: &mut Coin<T>, other: Coin<T>) {
        let Coin { balance } = other;
        balance::merge(&mut coin.balance, balance);
    }

    /// Convert a treasury (or treasury cap) into a supply handle (deprecated)
    /// This version borrows the `TreasuryCap` so callers can continue to
    /// use the cap after obtaining a `Supply` handle.
    public fun treasury_into_supply<T>(cap: &mut TreasuryCap<T>): Supply<T> {
        let _ = cap;
        balance::new_supply<T>()
    }

}
