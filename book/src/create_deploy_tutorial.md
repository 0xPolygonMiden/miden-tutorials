# Creating Accounts and Faucets 

*Using the Miden Client in Rust & TypeScript to Create Accounts and Deploy Faucets*

In this tutorial, we're going to explore how to get started with the Polygon Miden client in both Rust and TypeScript, walking through creating accounts and deploying faucets.

## What We'll Cover
* Understanding the difference between public vs. private accounts & notes
* Instantiating the miden-client
* Creating new accounts (public or private)
* Deploying a faucet

## Public vs. Private Accounts & Notes
Before we dive into the coding side of things, let's clarify the concepts of public vs. private Notes and Accounts on Miden:

* Public Accounts: The account's data and code are stored on-chain and are openly visible, including its assets.
* Private Accounts: The account's state and logic are off-chain, only known to its owner.
* Public Notes: The note's state is visible to anyone - perfect for scenarios where transparency is desired.
* Private Notes: The note's state is stored off-chain, you will need to share the note data with the relevant parties (via email or Telegram) for them to be able to consume.

*It is useful to think of notes on Miden as "cryptographic cashier's checks" that allow users to send tokens. If the note is private, the note transfer is only known to the sender and receiver.*

## Overview

In this tutorial we will create a miden account for *Alice* and then deploy a fungible faucet. In the next section we will mint tokens from the faucet, and then sending the tokens from Alice's account to other Miden accounts.

You can follow along with the tutorial by taking a look at `/rust-client/create_mint_consume.rs`.  

### Node Setup

To follow along with the tutorial using Rust or TypeScript, you'll need to connect to a running Miden node. There are two ways of doing this. By default, the examples connect to the Miden testnet node, however, if you want to run the node locally, follow the instructions in `docs/miden_node_setup_tutorial.md`

### Running the Example

To run the tutorial code:
```bash
cargo run --release --bin create_mint_consume
```

## Step 1: Initializing the Client

Before we can interact with the Miden network, we need to instantiate the client. The main parameters when instantiating the miden-client are the rpc endpoint and the transaction prover. The full example of instantiating the client in rust is in the `rust-client/src/common.rs` file.

This is how to instantiate the client in Rust & TypeScript:

<details>
<summary>Rust</summary>

```rust
let client = Client::new(
    rpc_client,              // RPC endpoint  
    rng_for_client,          // rng
    arc_store,               // store 
    Arc::new(authenticator), // signature authenticator
    Arc::new(tx_prover),     // tx prover (local or delegated)
    true,                    // debug mode
);
```

</details>

<details>

<summary>TypeScript</summary>

```TypeScript
const nodeEndpoint = 'http://18.203.155.106:57291';
let webClient = webClient.create_client(nodeEndpoint)
```


</details>

## Step 2: Creating a Wallet

To create a wallet for Alice using the miden client, we specify the account type by specifying if the account code is mutable or immutable and whether the account is public or private. In the examples below we create a mutable public account for Alice.


<details>
<summary>Rust</summary>

```rust
let alice_template = AccountTemplate::BasicWallet {
    mutable_code: true,
    storage_mode: AccountStorageMode::Public,
};

let (alice_account, _alice_seed) = client.new_account(alice_template).await?;
```

</details>


<details>
<summary>TypeScript</summary>

```TypeScript
const accountId = await webClient.new_wallet(
  AccountStorageMode.public(), // storage type
  true                         // mutable
);
```

</details>


## Step 3: Deploying a Fungible Faucet

For Alice to have testnet assets, we need to first deploy a faucet. A faucet account on Miden, mints fungible tokens. We'll create a public faucet with a token symbol, decimals, and a max supply. We will use this faucet to mint tokens to Alice's account. 


<details>
<summary>Rust</summary>

```rust
let faucet_template = AccountTemplate::FungibleFaucet {
    token_symbol: TokenSymbol::new("MID").unwrap(),
    decimals: 8,
    max_supply: 1_000_000,
    storage_mode: AccountStorageMode::Public,
};

let (faucet_account, _faucet_seed) = client.new_account(faucet_template).await?;
```

</details>


<details>
<summary>TypeScript</summary>

```rust
const faucetId = await webClient.new_faucet(
AccountStorageMode.public(), // storage mode
false,                       // is non fungible (only fungible currently supported)
'MID',                       // Token symbol
8,                           // Decimals
BigInt(1_000_000)            // max supply
);
```

</details>

*When tokens are minted from this faucet, each token batch is represented as a "note" (UTXO). You can think of a Miden Note as a cryptographic cashier's check that has certain spend conditions attached to it.*


## Summary

In this section we explained how to instantiate the miden-client, create a wallet account, and deploy a faucet. In the next section we will cover how to mint tokens from the faucet, consume notes, and send tokens to other accounts. 