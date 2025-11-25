
<a name="0x2_balance"></a>

# Module `0x2::balance`

Balance Module - จัดการยอดคงเหลือของ Kanari Coin


-  [Struct `Balance`](#0x2_balance_Balance)
-  [Constants](#@Constants_0)
-  [Function `zero`](#0x2_balance_zero)
-  [Function `create`](#0x2_balance_create)
-  [Function `value`](#0x2_balance_value)
-  [Function `increase`](#0x2_balance_increase)
-  [Function `decrease`](#0x2_balance_decrease)
-  [Function `transfer`](#0x2_balance_transfer)
-  [Function `has_sufficient`](#0x2_balance_has_sufficient)
-  [Function `destroy`](#0x2_balance_destroy)
-  [Function `merge`](#0x2_balance_merge)
-  [Function `split`](#0x2_balance_split)


<pre><code><b>use</b> <a href="dependencies/move-stdlib/error.md#0x1_error">0x1::error</a>;
</code></pre>



<a name="0x2_balance_Balance"></a>

## Struct `Balance`

Balance resource - เก็บยอดคงเหลือ (generic per token type)


<pre><code><b>struct</b> <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt; <b>has</b> drop, store
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

<a name="@Constants_0"></a>

## Constants


<a name="0x2_balance_ERR_INSUFFICIENT_BALANCE"></a>

Error codes


<pre><code><b>const</b> <a href="balance.md#0x2_balance_ERR_INSUFFICIENT_BALANCE">ERR_INSUFFICIENT_BALANCE</a>: u64 = 1;
</code></pre>



<a name="0x2_balance_ERR_OVERFLOW"></a>



<pre><code><b>const</b> <a href="balance.md#0x2_balance_ERR_OVERFLOW">ERR_OVERFLOW</a>: u64 = 2;
</code></pre>



<a name="0x2_balance_zero"></a>

## Function `zero`

สร้าง Balance ใหม่


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_zero">zero</a>&lt;T&gt;(): <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_zero">zero</a>&lt;T&gt;(): <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt; {
    <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt; { value: 0 }
}
</code></pre>



</details>

<a name="0x2_balance_create"></a>

## Function `create`

สร้าง Balance ด้วยจำนวนเริ่มต้น


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_create">create</a>&lt;T&gt;(value: u64): <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_create">create</a>&lt;T&gt;(value: u64): <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt; {
    <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt; { value }
}
</code></pre>



</details>

<a name="0x2_balance_value"></a>

## Function `value`

ดูยอดคงเหลือ


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_value">value</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: &<a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_value">value</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: &<a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt;): u64 {
    <a href="balance.md#0x2_balance">balance</a>.value
}
</code></pre>



</details>

<a name="0x2_balance_increase"></a>

## Function `increase`

เพิ่มยอดคงเหลือ


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_increase">increase</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_increase">increase</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt;, amount: u64) {
    <b>let</b> new_value = <a href="balance.md#0x2_balance">balance</a>.value + amount;
    <b>assert</b>!(new_value &gt;= <a href="balance.md#0x2_balance">balance</a>.value, <a href="dependencies/move-stdlib/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="balance.md#0x2_balance_ERR_OVERFLOW">ERR_OVERFLOW</a>));
    <a href="balance.md#0x2_balance">balance</a>.value = new_value;
}
</code></pre>



</details>

<a name="0x2_balance_decrease"></a>

## Function `decrease`

ลดยอดคงเหลือ


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_decrease">decrease</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_decrease">decrease</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt;, amount: u64) {
    <b>assert</b>!(<a href="balance.md#0x2_balance">balance</a>.value &gt;= amount, <a href="dependencies/move-stdlib/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="balance.md#0x2_balance_ERR_INSUFFICIENT_BALANCE">ERR_INSUFFICIENT_BALANCE</a>));
    <a href="balance.md#0x2_balance">balance</a>.value = <a href="balance.md#0x2_balance">balance</a>.value - amount;
}
</code></pre>



</details>

<a name="0x2_balance_transfer"></a>

## Function `transfer`

โอนยอดจาก Balance หนึ่งไปอีก Balance หนึ่ง


<pre><code><b>public</b> <b>fun</b> <a href="transfer.md#0x2_transfer">transfer</a>&lt;T&gt;(from: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;, <b>to</b>: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transfer.md#0x2_transfer">transfer</a>&lt;T&gt;(from: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt;, <b>to</b>: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt;, amount: u64) {
    <a href="balance.md#0x2_balance_decrease">decrease</a>&lt;T&gt;(from, amount);
    <a href="balance.md#0x2_balance_increase">increase</a>&lt;T&gt;(<b>to</b>, amount);
}
</code></pre>



</details>

<a name="0x2_balance_has_sufficient"></a>

## Function `has_sufficient`

ตรวจสอบว่ามียอดเพียงพอหรือไม่


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_has_sufficient">has_sufficient</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: &<a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;, amount: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_has_sufficient">has_sufficient</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: &<a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt;, amount: u64): bool {
    <a href="balance.md#0x2_balance">balance</a>.value &gt;= amount
}
</code></pre>



</details>

<a name="0x2_balance_destroy"></a>

## Function `destroy`

ทำลาย Balance และคืนค่า


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_destroy">destroy</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_destroy">destroy</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt;): u64 {
    <b>let</b> <a href="balance.md#0x2_balance_Balance">Balance</a> { value } = <a href="balance.md#0x2_balance">balance</a>;
    value
}
</code></pre>



</details>

<a name="0x2_balance_merge"></a>

## Function `merge`

รวม Balance สองอัน


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_merge">merge</a>&lt;T&gt;(dst: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;, src: <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_merge">merge</a>&lt;T&gt;(dst: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt;, src: <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt;) {
    <b>let</b> value = <a href="balance.md#0x2_balance_destroy">destroy</a>&lt;T&gt;(src);
    <a href="balance.md#0x2_balance_increase">increase</a>&lt;T&gt;(dst, value);
}
</code></pre>



</details>

<a name="0x2_balance_split"></a>

## Function `split`

แยก Balance


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_split">split</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;, amount: u64): <a href="balance.md#0x2_balance_Balance">balance::Balance</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="balance.md#0x2_balance_split">split</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>: &<b>mut</b> <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt;, amount: u64): <a href="balance.md#0x2_balance_Balance">Balance</a>&lt;T&gt; {
    <a href="balance.md#0x2_balance_decrease">decrease</a>&lt;T&gt;(<a href="balance.md#0x2_balance">balance</a>, amount);
    <a href="balance.md#0x2_balance_create">create</a>&lt;T&gt;(amount)
}
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")

[Move Language]: https://github.com/move-language/move
[Kanari]: https://github.com/jamesatomc/kanari-cp
[Move Book]: https://move-language.github.io/move/
[Transfer Module]: transfer.md
