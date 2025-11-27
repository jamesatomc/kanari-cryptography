module kanari_system::object {
    use kanari_system::tx_context;
    use kanari_system::tx_context::TxContext;

    /// Simple UID wrapper used for resource IDs in this package.
    /// The UID contains an object-style address generated from the
    /// transaction context so it's unique per creation.
    struct UID has store, drop {
        addr: address,
    }

    /// Create a new UID by deriving a fresh object address from the
    /// transaction context.
    public fun new(ctx: &mut TxContext): UID {
        UID { addr: tx_context::fresh_object_address(ctx) }
    }

    /// Return the underlying address for a UID
    public fun uid_address(u: &UID): address {
        u.addr
    }
}
