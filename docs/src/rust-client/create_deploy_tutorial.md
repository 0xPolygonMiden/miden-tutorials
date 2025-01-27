# Creating Accounts and Faucets 

*Using the Miden client in Rust to create accounts and deploy faucets*

## Overview
In this tutorial, we will create a Miden account for *Alice* and deploy a fungible faucet. In the next section, we will mint tokens from the faucet to fund her account and transfer tokens from Alice's account to other Miden accounts.

## What we'll cover
* Understanding the differences between public and private accounts & notes
* Instantiating the Miden client
* Creating new accounts (public or private)
* Deploying a faucet to fund an account


## Prerequisites
Before you begin, ensure that a Miden node is running locally in a separate terminal window. To get the Miden node running locally, you can follow the instructions on the [Miden Node Setup](../miden_node_setup_tutorial.md) page.

## Public vs. private accounts & notes
Before diving into coding, let's clarify the concepts of public and private accounts & notes on Miden:

* Public accounts: The account's data and code are stored on-chain and are openly visible, including its assets.
* Private accounts: The account's state and logic are off-chain, only known to its owner.
* Public notes: The note's state is visible to anyone - perfect for scenarios where transparency is desired.
* Private notes: The note's state is stored off-chain, you will need to share the note data with the relevant parties (via email or Telegram) for them to be able to consume the note.

Note: *The term "account" can be used interchangeably with the term "smart contract" since account abstraction on Miden is handled natively.*

*It is useful to think of notes on Miden as "cryptographic cashier's checks" that allow users to send tokens. If the note is private, the note transfer is only known to the sender and receiver.*


## Step 1: Initialize your repository
Create a new Rust repository for your Miden project and navigate to it with the following command:
```bash
cargo new miden-rust-client
cd miden-rust-client 
```

Add the following dependencies to your `Cargo.toml` file:
```toml
[dependencies]
miden-client = { version = "0.6", features = ["testing", "concurrent", "tonic", "sqlite"] }
miden-lib = { version = "0.6", default-features = false }
miden-objects = { version = "0.6", default-features = false }
miden-crypto = { version = "0.13.0", features = ["executable"] }
rand = { version = "0.8" }
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1.0", features = ["raw_value"] }
tokio = { version = "1.40", features = ["rt-multi-thread", "net", "macros"] }
rand_chacha = "0.3.1"
```

## Step 2: Initialize the client
Before interacting with the Miden network, we must instantiate the client. In this step, we specify several parameters:

* **RPC endpoint** - The URL of the Miden node to which we connect.
* **SQLite file** – A database file (store.sqlite3) used by the client to store account and note data.
* **Client RNG** - The random number generator used by the client, ensuring that the serial number of newly created notes are unique.
* **Authenticator RNG** - The random number generator used by the transaction authenticator during signature generation for the Miden VM.
* **Transaction Prover** - The URL for delegated proving, useful when using a resource-constrained environment (e.g., a cellphone) that cannot handle local proving efficiently.

Copy and paste the following code into your `src/main.rs` file. 
```rust
use miden_client::{
    accounts::{AccountStorageMode,AccountTemplate},
    assets::TokenSymbol,
    config::RpcConfig,
    crypto::RpoRandomCoin,
    rpc::TonicRpcClient,
    store::{
        sqlite_store::{config::SqliteStoreConfig, SqliteStore},
        StoreAuthenticator,
    },
    transactions::{LocalTransactionProver, ProvingOptions},
    Client, ClientError, Felt,
};
use rand::Rng;
use std::sync::Arc;

pub async fn initialize_client() -> Result<Client<RpoRandomCoin>, ClientError> {

    // Default values for store and rpc config
    let store_config = SqliteStoreConfig::default();
    let rpc_config = RpcConfig::default();

    // Create an SQLite store
    let store = SqliteStore::new(&store_config)
        .await
        .map_err(ClientError::StoreError)?;
    let arc_store = Arc::new(store);

    // Seed both random coin instances
    let mut seed_rng = rand::thread_rng();
    let coin_seed: [u64; 4] = seed_rng.gen();
    let rng_for_auth = RpoRandomCoin::new(coin_seed.map(Felt::new));
    let rng_for_client = RpoRandomCoin::new(coin_seed.map(Felt::new));

    // Create an authenticator that references the store
    let authenticator = StoreAuthenticator::new_with_rng(arc_store.clone(), rng_for_auth);

    // Local prover (you could swap out for delegated proving)
    let tx_prover = LocalTransactionProver::new(ProvingOptions::default());

    // Build the RPC client
    let rpc_client = Box::new(TonicRpcClient::new(&rpc_config));

    // Finally, create the client
    let client = Client::new(
        rpc_client,
        rng_for_client,
        arc_store,
        Arc::new(authenticator),
        Arc::new(tx_prover),
        true,
    );

    Ok(client)
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let mut client = initialize_client().await?;

    let sync_summary = client.sync_state().await.unwrap();
    let block_number = sync_summary.block_num;

    println!("Latest block number: {}", block_number);

    Ok(())
}
```

*When running the code above, there will be some unused imports, however, we will use these imports later on in the tutorial.*

In this step, we will initialize a Miden client capable of syncing with the blockchain (in this case, our local node). Run the following command to execute `src/main.rs`:
```bash
cargo run --release 
```

After the program executes, you should see the latest block number printed to the terminal, for example:
```
Latest block number: 3855 
```

## Step 3: Creating a wallet
Now that we've initialized the client, we can create a wallet for Alice.

To create a wallet for Alice using the Miden client, we define the account type as mutable or immutable and specify whether it is public or private. A mutable wallet means you can change the account code after deployment. A wallet on Miden is simply an account with standardized code.

In the example below we create a mutable public account for Alice. 

Add this snippet to the end of your file in the `main()` function:
```rust
let alice_template = AccountTemplate::BasicWallet {
    mutable_code: true,
    storage_mode: AccountStorageMode::Public,
};

let (alice_account, _alice_seed) = client.new_account(alice_template).await?;

println!("Alice's account id: {}", alice_account.id());
```

## Step 4: Deploying a fungible faucet
To provide Alice with testnet assets, we must first deploy a faucet. A faucet account on Miden mints fungible tokens.

We'll create a public faucet with a token symbol, decimals, and a max supply. We will use this faucet to mint tokens to Alice's account in the next section.

Add this snippet to the end of your file in the `main()` function:
```rust
let faucet_template = AccountTemplate::FungibleFaucet {
    token_symbol: TokenSymbol::new("MID").unwrap(),
    decimals: 8,
    max_supply: 1_000_000,
    storage_mode: AccountStorageMode::Public,
};

let (faucet_account, _faucet_seed) = client.new_account(faucet_template).await?;

println!("Faucet account id: {}", faucet_account.id());
```

*When tokens are minted from this faucet, each token batch is represented as a "note" (UTXO). You can think of a Miden Note as a cryptographic cashier's check that has certain spend conditions attached to it.*


## Summary
Your updated `main()` function in `src/main.rs` should look like this:

```rust
#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let mut client = initialize_client().await?;

    let sync_summary = client.sync_state().await.unwrap();
    let block_number = sync_summary.block_num;

    println!("Latest block number: {}", block_number);

    let alice_template = AccountTemplate::BasicWallet {
        mutable_code: true,
        storage_mode: AccountStorageMode::Public,
    };
    
    let (alice_account, _alice_seed) = client.new_account(alice_template).await?;

    println!("Alice's account id: {}", alice_account.id());

    let faucet_template = AccountTemplate::FungibleFaucet {
        token_symbol: TokenSymbol::new("MID").unwrap(),
        decimals: 8,
        max_supply: 1_000_000,
        storage_mode: AccountStorageMode::Public,
    };
    
    let (faucet_account, _faucet_seed) = client.new_account(faucet_template).await?;

    println!("Faucet account id: {}", faucet_account.id());

    Ok(())
}
```

Let's run the `src/main.rs` program again:
```bash
cargo run --release 
```

The output will look like this:
```bash
Latest block number: 607494
Alice's account id: 0x1a8eefbcfef43f48
Faucet account id: 0x2d7969e6125856d0
```

In this section we explained how to instantiate the Miden client, create a wallet account, and deploy a faucet. 

In the next section we will cover how to mint tokens from the faucet, consume notes, and send tokens to other accounts. 

### Running the example
To run a full working example navigate to the `rust-client` directory in the [miden-tutorials](https://github.com/0xPolygonMiden/miden-tutorials/) repository and run this command:
```bash
cd rust-client
cargo run --release --bin create_mint_consume_send
```

### Continue learning
Next tutorial: [Mint, Consume, and Create Notes](mint_consume_create_tutorial.md)