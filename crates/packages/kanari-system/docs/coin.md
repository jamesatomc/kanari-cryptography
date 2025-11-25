
<a name="0x2_coin"></a>

# Module `0x2::coin`



-  [Struct `Coin`](#0x2_coin_Coin)
-  [Struct `Treasury`](#0x2_coin_Treasury)
-  [Struct `Supply`](#0x2_coin_Supply)
-  [Struct `CurrencyMetadata`](#0x2_coin_CurrencyMetadata)
-  [Function `create_currency`](#0x2_coin_create_currency)
-  [Function `treasury_into_supply`](#0x2_coin_treasury_into_supply)
-  [Function `increase_supply`](#0x2_coin_increase_supply)
-  [Function `destroy_supply`](#0x2_coin_destroy_supply)
-  [Function `create_coin`](#0x2_coin_create_coin)
-  [Function `value`](#0x2_coin_value)
-  [Function `burn`](#0x2_coin_burn)


<pre><code><b>use</b> <a href="dependencies/move-stdlib/ascii.md#0x1_ascii">0x1::ascii</a>;
<b>use</b> <a href="dependencies/move-stdlib/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="balance.md#0x2_balance">0x2::balance</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x2_coin_Coin"></a>

## Struct `Coin`

Minimal Coin resource wrapper (per token type)


<pre><code><b>struct</b> <a href="coin.md#0x2_coin_Coin">Coin</a>&lt;T&gt; <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>value: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x2_coin_Treasury"></a>

## Struct `Treasury`

Treasury: holds authority to mint into a Supply


<pre><code><b>struct</b> <a href="coin.md#0x2_coin_Treasury">Treasury</a>&lt;T&gt; <b>has</b> drop, store
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

<a name="0x2_coin_Supply"></a>

## Struct `Supply`

Supply: mutable minting handle consumed to create balances


<pre><code><b>struct</b> <a href="coin.md#0x2_coin_Supply">Supply</a>&lt;T&gt; <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>total: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x2_coin_CurrencyMetadata"></a>

## Struct `CurrencyMetadata`

Simple metadata for a currency


<pre><code><b>struct</b> <a href="coin.md#0x2_coin_CurrencyMetadata">CurrencyMetadata</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>symbol: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>name: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>description: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x2_coin_create_currency"></a>

## Function `create_currency`

Create a new currency. This is a minimal implementation returning
a <code><a href="coin.md#0x2_coin_Treasury">Treasury</a></code> and a <code><a href="coin.md#0x2_coin_CurrencyMetadata">CurrencyMetadata</a></code> object. The <code>ctx</code> parameter is
accepted to match expected caller signatures (e.g., genesis flows).


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_create_currency">create_currency</a>&lt;T: drop&gt;(_token: T, _decimals: u8, symbol: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>&lt;u8&gt;, name: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>&lt;u8&gt;, description: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>&lt;u8&gt;, _icon: <a href="dependencies/move-stdlib/option.md#0x1_option_Option">option::Option</a>&lt;<a href="dependencies/move-stdlib/ascii.md#0x1_ascii_String">ascii::String</a>&gt;, _ctx: &<b>mut</b> <a href="tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): (<a href="coin.md#0x2_coin_Treasury">coin::Treasury</a>&lt;T&gt;, <a href="coin.md#0x2_coin_CurrencyMetadata">coin::CurrencyMetadata</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_create_currency">create_currency</a>&lt;T: drop&gt;(
    _token: T,
    _decimals: u8,
    symbol: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    name: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    description: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    _icon: <a href="dependencies/move-stdlib/option.md#0x1_option_Option">option::Option</a>&lt;String&gt;,
    _ctx: &<b>mut</b> TxContext,
): (<a href="coin.md#0x2_coin_Treasury">Treasury</a>&lt;T&gt;, <a href="coin.md#0x2_coin_CurrencyMetadata">CurrencyMetadata</a>) {
    // Token witness is consumed automatically <b>as</b> it <b>has</b> drop ability
    (
        <a href="coin.md#0x2_coin_Treasury">Treasury</a>&lt;T&gt; {},
        <a href="coin.md#0x2_coin_CurrencyMetadata">CurrencyMetadata</a> { symbol, name, description },
    )
}
</code></pre>



</details>

<a name="0x2_coin_treasury_into_supply"></a>

## Function `treasury_into_supply`

Convert a treasury into a supply handle (consumes the treasury)


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_treasury_into_supply">treasury_into_supply</a>&lt;T&gt;(treasury: <a href="coin.md#0x2_coin_Treasury">coin::Treasury</a>&lt;T&gt;): <a href="coin.md#0x2_coin_Supply">coin::Supply</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_treasury_into_supply">treasury_into_supply</a>&lt;T&gt;(treasury: <a href="coin.md#0x2_coin_Treasury">Treasury</a>&lt;T&gt;): <a href="coin.md#0x2_coin_Supply">Supply</a>&lt;T&gt; {
    <b>let</b> <a href="coin.md#0x2_coin_Treasury">Treasury</a> {} = treasury;
    <a href="coin.md#0x2_coin_Supply">Supply</a>&lt;T&gt; { total: 0 }
}
</code></pre>



</details>

<a name="0x2_coin_increase_supply"></a>

## Function `increase_supply`

Increase supply and return a Balance<T> representing newly minted tokens


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_increase_supply">increase_supply</a>&lt;T&gt;(s: <a href="coin.md#0x2_coin_Supply">coin::Supply</a>&lt;T&gt;, amount: u64): <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_increase_supply">increase_supply</a>&lt;T&gt;(s: <a href="coin.md#0x2_coin_Supply">Supply</a>&lt;T&gt;, amount: u64): <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt; {
    <b>let</b> <a href="coin.md#0x2_coin_Supply">Supply</a> { total } = s;
    <b>let</b> _new_total = total + amount;
    <a href="balance.md#0x2_balance_create">balance::create</a>&lt;T&gt;(amount)
}
</code></pre>



</details>

<a name="0x2_coin_destroy_supply"></a>

## Function `destroy_supply`

Destroy supply handle (no-op for minimal implementation)


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_destroy_supply">destroy_supply</a>&lt;T&gt;(s: <a href="coin.md#0x2_coin_Supply">coin::Supply</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_destroy_supply">destroy_supply</a>&lt;T&gt;(s: <a href="coin.md#0x2_coin_Supply">Supply</a>&lt;T&gt;) {
    <b>let</b> <a href="coin.md#0x2_coin_Supply">Supply</a> { total: _ } = s;
}
</code></pre>



</details>

<a name="0x2_coin_create_coin"></a>

## Function `create_coin`

Create a coin instance (value holder)


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_create_coin">create_coin</a>&lt;T&gt;(amount: u64): <a href="coin.md#0x2_coin_Coin">coin::Coin</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_create_coin">create_coin</a>&lt;T&gt;(amount: u64): <a href="coin.md#0x2_coin_Coin">Coin</a>&lt;T&gt; {
    <a href="coin.md#0x2_coin_Coin">Coin</a>&lt;T&gt; { value: amount }
}
</code></pre>



</details>

<a name="0x2_coin_value"></a>

## Function `value`

Get value


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_value">value</a>&lt;T&gt;(c: &<a href="coin.md#0x2_coin_Coin">coin::Coin</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_value">value</a>&lt;T&gt;(c: &<a href="coin.md#0x2_coin_Coin">Coin</a>&lt;T&gt;): u64 { c.value }
</code></pre>



</details>

<a name="0x2_coin_burn"></a>

## Function `burn`

Burn and return underlying value


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_burn">burn</a>&lt;T&gt;(c: <a href="coin.md#0x2_coin_Coin">coin::Coin</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="coin.md#0x2_coin_burn">burn</a>&lt;T&gt;(c: <a href="coin.md#0x2_coin_Coin">Coin</a>&lt;T&gt;): u64 {
    <b>let</b> <a href="coin.md#0x2_coin_Coin">Coin</a> { value } = c;
    value
}
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")

[Move Language]: https://github.com/move-language/move
[Kanari]: https://github.com/jamesatomc/kanari-cp
[Move Book]: https://move-language.github.io/move/
[Transfer Module]: transfer.md
