module kanari_system::genesis {
    use std::vector;
    use kanari_system::tx_context;
    use kanari_system::tx_context::TxContext;
    use kanari_system::kanari;

    /// Initialize the chain: create the KANARI currency and distribute
    /// the total supply evenly across 2 recipient addresses.
    /// This should be executed by the genesis script (sender @0x0, epoch 0).
    public entry fun init(ctx: &mut TxContext) {
        // Only allow genesis caller
        assert!(tx_context::sender(ctx) == @0x0, 1);
        assert!(tx_context::epoch(ctx) == 0, 2);

        // Build recipient and amount vectors and delegate to kanari initializer.
        let recipients = vector::empty<address>();
        vector::push_back(&mut recipients, @0x3603a1c5737316534fbb1cc0fa599258e401823059f3077d2a8d86a998825739);
        vector::push_back(&mut recipients, @0x68fa44652bb2e4e1eb8257b807838a46102e9a48b0f2169a9ec3d2a99ed24301);


        // equal share per recipient (split across 2 recipients)
        let share = 10_000_000_000_000_000_000u64 / 2u64;
        let amounts = vector::empty<u64>();
        vector::push_back(&mut amounts, share);
        vector::push_back(&mut amounts, share);

        kanari::init_genesis(ctx, recipients, amounts);
    }
}



