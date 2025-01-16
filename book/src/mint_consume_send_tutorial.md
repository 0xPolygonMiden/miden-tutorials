# Mint, Consume, and Send Assets

*Using the Miden Client in Rust & TypeScript to Mint, Consume, and Send Assets*

## Overview

In the previous section we covered how to create an account and deploy a faucet. In this section we will mint tokens from the faucet for *Alice*, consume these created notes, and then show how to send assets to other accounts.

## What We'll Cover
* Minting tokens from a faucet
* Consuming notes
* Sending tokens to other users

### Running the Example

To run the tutorial code:
```bash
cargo run --release --bin create_mint_consume
```

## Step 1: Minting Tokens from the Faucet

To mint notes with tokens from the faucet we created, Alice needs to call the faucet with a mint transaction request.

<details>
<summary>Rust</summary>

```rust
let amount: i32 = 100;
let fungible_asset = FungibleAsset::new(faucet_account.id(), amount)
    .unwrap();

let transaction_request = TransactionRequest::mint_fungible_asset(
    fungible_asset.clone(), // fungible asset id
    alice_account.id(),     // target account id
    NoteType::Public,       // minted note type
    client.rng(),           // rng 
)
.unwrap();

let tx_execution_result = client
    .new_transaction(faucet_account.id(), transaction_request)
    .await?;


client.submit_transaction(tx_execution_result).await?;
```

</details>

<details>
<summary>TypeScript</summary>

```TypeScript
const newTxnResult = await webClient.new_mint_transaction(
  _walletId,          // target wallet id
  _faucetId,          // faucet id
  NoteType.public(),  // Note Type
  amount              // amount
);
```

</details>

## Step 2: Identifying Consumable Notes

Once Alice has minted a note from the faucet, she will eventually want to spend the tokens that she received in the note created by the mint transaction. To identify notes that are ready to consume, the miden-client has a useful function `get_consumable_notes`. It is also important to sync the state of the client before calling the `get_consumable_notes` function. 

*Tip: If you know how many notes to expect after a transaction, use an await or loop condition to check how many notes of the type you expect are available for consumption instead of using a set timeout before calling `get_consumable_notes`. This ensures your application isn't idle for longer than necessary.*

#### Identifying which notes are available:
<details>
<summary>Rust</summary>

```rust
let consumable_notes = client.get_consumable_notes(Some(alice_account.id())).await?;
```

</details>


<details>
<summary>TypeScript</summary>

```TypeScript
let notes = await webClient.get_consumable_notes(_accountId);
```

</details>

## Step 3: Consuming multiple notes in a single transaction:

Now that we've identified the notes ready to consume, we can consume multiple notes in a single transaction. After consuming the notes, Alice's wallet balance will be updated.

<details>
<summary>Rust</summary>

```Rust
let transaction_request = TransactionRequest::consume_notes(list_of_note_ids);
let tx_execution_result = client
    .new_transaction(alice_account.id(), transaction_request)
    .await?;

client.submit_transaction(tx_execution_result).await?;
```

</details>


<details>
<summary>TypeScript</summary>

```TypeScript
const consumeTransactionResult = await webClient.new_consume_transaction(
  _targetAccountId,
  [noteId]
);
```

</details>


## Step 4: Sending Tokens to Other Accounts

Now that Alice has tokens in her wallet, she wants to send some tokens to some of her friends. She has two options. She can create a separate transaction for each transfer to each friend, or she can batch the transfer in a single transaction. 

The standard asset transfer note on Miden is the P2ID note (Pay to Id). There is also the P2IDR variant which allows the creator of the note to reclaim the note after a certain block height. 

In our example, Alice will now send 50 tokens to 5 different accounts.

For the sake of the example, the first four P2ID transfers are handled in a single transaction, and the fifth transfer is a standard P2ID transfer. 

### Output multiple P2ID notes in a single transaction

<details>
<summary>Rust</summary>

```Rust
let output_notes: Vec<OutputNote> = p2id_notes.into_iter().map(OutputNote::Full).collect();

let transaction_request = TransactionRequest::new()
    .with_own_output_notes(output_notes)
    .unwrap();

let tx_execution_result = client
    .new_transaction(alice_account.id(), transaction_request)
    .await?;

client.submit_transaction(tx_execution_result).await?;
```

</details>

<details>
<summary>Typescript</summary>

```Typescript
let transaction_request = new TransactionRequest().with_own_output_notes(
  new OutputNotesArray([OutputNote.full(note_1), OutputNote.full(note_2)])
);

await webClient.fetch_and_cache_account_auth_by_pub_key(senderAccountId);
let transaction_result = await webClient.new_transaction(
  senderAccountId,
  transaction_request
);

await webClient.submit_transaction(transaction_result);
```

</details>

### Basic P2ID transfer

<details>
<summary>Rust</summary>

```Rust
let transaction_request = TransactionRequest::pay_to_id(
    payment_transaction,
    None,             // recall_height: None
    NoteType::Public, // note type is public
    client.rng(),     // rng
)
.unwrap();

let tx_execution_result = client
    .new_transaction(alice_account.id(), transaction_request)
    .await?;

client.submit_transaction(tx_execution_result).await?;
```

</details>


<details>
<summary>TypeScript</summary>

```Typescript
const sendResult = await webClient.new_send_transaction(
  senderAccountId,
  targetAccId,
  faucetAccountId,
  NoteType.private(),
  amount
);
```

</details>