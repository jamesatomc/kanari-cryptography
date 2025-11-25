
<a name="0x2_kanari"></a>

# Module `0x2::kanari`

Kanari Coin - เหรียญหลักของระบบ Kanari
Module นี้เป็น main entry point สำหรับการจัดการเหรียญ Kanari
จะประกอบด้วยฟังก์ชันสำหรับลงทะเบียนเหรียญ Kanari ในขั้น genesis


-  [Struct `KANARI`](#0x2_kanari_KANARI)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x2_kanari_new)


<pre><code><b>use</b> <a href="dependencies/move-stdlib/ascii.md#0x1_ascii">0x1::ascii</a>;
<b>use</b> <a href="dependencies/move-stdlib/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="balance.md#0x2_balance">0x2::balance</a>;
<b>use</b> <a href="coin.md#0x2_coin">0x2::coin</a>;
<b>use</b> <a href="transfer.md#0x2_transfer">0x2::transfer</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_kanari_KANARI"></a>

## Struct `KANARI`

Name of the coin


<pre><code><b>struct</b> <a href="kanari.md#0x2_kanari_KANARI">KANARI</a> <b>has</b> drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x2_kanari_EAlreadyMinted"></a>



<pre><code><b>const</b> <a href="kanari.md#0x2_kanari_EAlreadyMinted">EAlreadyMinted</a>: u64 = 0;
</code></pre>



<a name="0x2_kanari_ENotSystemAddress"></a>

Sender is not @0x0 the system address.


<pre><code><b>const</b> <a href="kanari.md#0x2_kanari_ENotSystemAddress">ENotSystemAddress</a>: u64 = 1;
</code></pre>



<a name="0x2_kanari_MIST_PER_KANARI"></a>

The amount of Mist per Kanari token based on the fact that mist is
10^-9 of a Kanari token


<pre><code><b>const</b> <a href="kanari.md#0x2_kanari_MIST_PER_KANARI">MIST_PER_KANARI</a>: u64 = 1000000000;
</code></pre>



<a name="0x2_kanari_TOTAL_SUPPLY_KANARI"></a>

The total supply of Kanari denominated in whole Kanari tokens (10 Billion)


<pre><code><b>const</b> <a href="kanari.md#0x2_kanari_TOTAL_SUPPLY_KANARI">TOTAL_SUPPLY_KANARI</a>: u64 = 10000000000;
</code></pre>



<a name="0x2_kanari_TOTAL_SUPPLY_MIST"></a>

The total supply of Kanari denominated in Mist (10 Billion * 10^9)


<pre><code><b>const</b> <a href="kanari.md#0x2_kanari_TOTAL_SUPPLY_MIST">TOTAL_SUPPLY_MIST</a>: u64 = 10000000000000000000;
</code></pre>



<a name="0x2_kanari_new"></a>

## Function `new`



<pre><code><b>fun</b> <a href="kanari.md#0x2_kanari_new">new</a>(ctx: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;<a href="kanari.md#0x2_kanari_KANARI">kanari::KANARI</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="kanari.md#0x2_kanari_new">new</a>(ctx: &<b>mut</b> TxContext): Balance&lt;<a href="kanari.md#0x2_kanari_KANARI">KANARI</a>&gt; {
    <b>assert</b>!(<a href="tx_context.md#0x2_tx_context_sender">tx_context::sender</a>(ctx) == @0x0, <a href="kanari.md#0x2_kanari_ENotSystemAddress">ENotSystemAddress</a>);
    <b>assert</b>!(<a href="tx_context.md#0x2_tx_context_epoch">tx_context::epoch</a>(ctx) == 0, <a href="kanari.md#0x2_kanari_EAlreadyMinted">EAlreadyMinted</a>);

    <b>let</b> (treasury, metadata) = <a href="coin.md#0x2_coin_create_currency">coin::create_currency</a>(
        <a href="kanari.md#0x2_kanari_KANARI">KANARI</a> {},
        9,
        b"<a href="kanari.md#0x2_kanari_KANARI">KANARI</a>",
        b"<a href="kanari.md#0x2_kanari_KANARI">KANARI</a>",
        // TODO: add appropriate description and logo <a href="url.md#0x2_url">url</a>
        b"",
        <a href="dependencies/move-stdlib/option.md#0x1_option_none">option::none</a>(),
        ctx,
    );
    <a href="transfer.md#0x2_transfer_public_freeze_object">transfer::public_freeze_object</a>(metadata);
    <b>let</b> supply = <a href="coin.md#0x2_coin_treasury_into_supply">coin::treasury_into_supply</a>(treasury);
    <b>let</b> total_kanari = <a href="coin.md#0x2_coin_increase_supply">coin::increase_supply</a>(supply, <a href="kanari.md#0x2_kanari_TOTAL_SUPPLY_MIST">TOTAL_SUPPLY_MIST</a>);
    total_kanari
}
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")

[Move Language]: https://github.com/move-language/move
[Kanari]: https://github.com/jamesatomc/kanari-cp
[Move Book]: https://move-language.github.io/move/
[Transfer Module]: transfer.md
