# Deploying a Counter Contract

*Using the Miden client in Rust to deploy and interact with a custom smart contract on Miden*

## Overview

In this tutorial, we will build a simple counter smart contract that maintains a count, deploy it to the Miden testnet, and interact with it by incrementing the count. You can also deploy the counter contract on a locally running Miden node, similar to previous tutorials.

Using a script, we will invoke the increment function within the counter contract to update the count. This tutorial provides a foundational understanding of developing and deploying custom smart contracts on Miden.

## What we'll cover

- Deploying a custom smart contract on Miden
- Getting up to speed with the basics of Miden assembly
- Calling procedures in an account
- Pure vs state changing procedures

## Prerequisites

This tutorial assumes you have a basic understanding of Miden assembly. To quickly get up to speed with Miden assembly (MASM), please play around with running Miden programs in the [Miden playground](https://0xpolygonmiden.github.io/examples/).

## Step 1: Initialize your repository

Create a new Rust repository for your Miden project and navigate to it with the following command:

```bash
cargo new miden-counter-contract
cd miden-counter-contract
```

Add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
miden-client = { version = "0.7", features = ["testing", "concurrent", "tonic", "sqlite"] }
miden-lib = { version = "0.7", default-features = false }
miden-objects = { version = "0.7.2", default-features = false }
miden-crypto = { version = "0.13.2", features = ["executable"] }
rand = { version = "0.8" }
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1.0", features = ["raw_value"] }
tokio = { version = "1.40", features = ["rt-multi-thread", "net", "macros"] }
rand_chacha = "0.3.1"
```

### Set up your `src/main.rs` file

In the previous section, we explained how to instantiate the Miden client. We can reuse the same `initialize_client` function for our counter contract.

Copy and paste the following code into your `src/main.rs` file:

```rust
use std::{fs, path::Path, sync::Arc};

use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use tokio::time::Duration;

use miden_client::{
    account::{AccountStorageMode, AccountType},
    crypto::RpoRandomCoin,
    rpc::{Endpoint, TonicRpcClient},
    store::{sqlite_store::SqliteStore, StoreAuthenticator},
    transaction::{TransactionKernel, TransactionRequestBuilder},
    Client, ClientError, Felt,
};

use miden_objects::{
    account::{AccountBuilder, AccountComponent, AuthSecretKey, StorageSlot},
    assembly::Assembler,
    crypto::{dsa::rpo_falcon512::SecretKey, hash::rpo::RpoDigest},
    Word,
};

pub async fn initialize_client() -> Result<Client<RpoRandomCoin>, ClientError> {
    // RPC endpoint and timeout
    let endpoint = Endpoint::new(
        "https".to_string(),
        "rpc.testnet.miden.io".to_string(),
        Some(443),
    );
    let timeout_ms = 10_000;

    // Build RPC client
    let rpc_api = Box::new(TonicRpcClient::new(endpoint, timeout_ms));

    // Seed RNG
    let mut seed_rng = rand::thread_rng();
    let coin_seed: [u64; 4] = seed_rng.gen();

    // Create random coin instance
    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));

    // SQLite path
    let store_path = "store.sqlite3";

    // Initialize SQLite store
    let store = SqliteStore::new(store_path.into())
        .await
        .map_err(ClientError::StoreError)?;
    let arc_store = Arc::new(store);

    // Create authenticator referencing the store and RNG
    let authenticator = StoreAuthenticator::new_with_rng(arc_store.clone(), rng.clone());

    // Instantiate client (toggle debug mode as needed)
    let client = Client::new(rpc_api, rng, arc_store, Arc::new(authenticator), true);

    Ok(client)
}

pub fn get_new_pk_and_authenticator() -> (Word, AuthSecretKey) {
    // Create a deterministic RNG with zeroed seed
    let seed = [0_u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    // Generate Falcon-512 secret key
    let sec_key = SecretKey::with_rng(&mut rng);

    // Convert public key to `Word` (4xFelt)
    let pub_key: Word = sec_key.public_key().into();

    // Wrap secret key in `AuthSecretKey`
    let auth_secret_key = AuthSecretKey::RpoFalcon512(sec_key);

    (pub_key, auth_secret_key)
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let mut client = initialize_client().await?;
    println!("Client initialized successfully.");

    let sync_summary = client.sync_state().await.unwrap();
    println!("Latest block: {}", sync_summary.block_num);

    Ok(())
}
```

*When running the code above, there will be some unused imports, however, we will use these imports later on in the tutorial.*

## Step 2: Build the counter contract

For better code organization, we will separate the Miden assembly code from our Rust code.

Create a directory named `masm` at the **root** of your `miden-counter-contract` directory. This will contain our contract and script masm code. 

Initialize the `masm` directory:

```bash
mkdir -p masm/accounts masm/scripts
```

This will create:

```
masm/
├── accounts/
└── scripts/
```

### Custom Miden smart contract

Below is our counter contract. It has a two exported procedures: `get_count` and `increment_count`.

At the beginning of the MASM file, we define our imports. In this case, we import `miden::account` and `std::sys`.

The import `miden::account` contains useful procedures for interacting with a smart contract's state. 

The import `std::sys` contains a useful procedure for truncating the operand stack at the end of a procedure.

#### Here's a breakdown of what the `get_count` procedure does:

1. Pushes `0` onto the stack, representing the index of the storage slot to read.
2. Calls `account::get_item` with the index of `0`.
3. Calls `sys::truncate_stack` to truncate the stack to size 16.
4. The value returned from `account::get_item` is still on the stack and will be returned when this procedure is called.

#### Here's a breakdown of what the `increment_count` procedure does:

1. Pushes `0` onto the stack, representing the index of the storage slot to read.
2. Calls `account::get_item` with the index of `0`.
3. Pushes `1` onto the stack.
4. Adds `1` to the count value returned from `account::get_item`.
5. *For demonstration purposes*, calls `debug.stack` to see the state of the stack
6. Pushes `0` onto the stack, which is the index of the storage slot we want to write to.
7. Calls `account::set_item` which saves the incremented count to storage at index `0`
8. Calls `sys::truncate_stack` to truncate the stack to size 16.

Inside of the `masm/accounts/` directory, create the `counter.masm` file:

```masm
use.miden::account
use.std::sys

export.get_count
    # => []
    push.0
    
    # => [index]
    exec.account::get_item

    # => [count]
    exec.sys::truncate_stack
end

export.increment_count
    # => []
    push.0

    # => [index]
    exec.account::get_item

    # => [count]
    push.1 add

    # debug statement with client
    debug.stack

    # => [count+1]
    push.0

    # [index, count+1]
    exec.account::set_item

    # => []
    push.1 exec.account::incr_nonce

    # => []
    exec.sys::truncate_stack
end
```

**Note**: *It's a good habit to add comments above each line of MASM code with the expected stack state. This improves readability and helps with debugging.*

### Concept of function visibility and modifiers in Miden smart contracts

The `increment_count` function in our Miden smart contract behaves like an "external" Solidity function without a modifier, meaning any user can call it to increment the contract's count. This is because it calls `account::incr_nonce` during execution.

If the `increment_count` procedure did not call the `account::incr_nonce` procedure during its execution, only the deployer of the counter contract would be able to increment the count of the smart contract (if the RpoFalcon512 component was added to the account, in this case we didn't add it).

In essence, if a procedure performs a state change in the Miden smart contract, and does not call `account::incr_nonce` at some point during its execution, this function can be equated to having an `onlyOwner` Solidity modifer, meaning only the user with knowledge of the private key of the account can execute transactions that result in a state change.

**Note**: *Adding the `account::incr_nonce` to a state changing procedure allows any user to call the procedure.*

### Custom script

This is a Miden assembly script that will call the `increment_count` procedure during the transaction. 

The string `{increment_count}` will be replaced with the hash of the `increment_count` procedure in our rust program.

Inside of the `masm/scripts/` directory, create the `counter_script.masm` file:

```masm
begin
    # => []
    call.{increment_count}
end
```

## Step 3: Build the counter smart contract in Rust

To build the counter contract copy and paste the following code at the end of your `src/main.rs` file:

```rust
// -------------------------------------------------------------------------
// STEP 1: Create a basic counter contract
// -------------------------------------------------------------------------
println!("\n[STEP 1] Creating counter contract.");

// Load the MASM file for the counter contract
let file_path = Path::new("./masm/accounts/counter.masm");
let account_code = fs::read_to_string(file_path).unwrap();

// Prepare assembler (debug mode = true)
let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);

// Compile the account code into `AccountComponent` with one storage slot
let counter_component = AccountComponent::compile(
    account_code,
    assembler,
    vec![StorageSlot::Value(Word::default())],
)
.unwrap()
.with_supports_all_types();

// Init seed for the counter contract
let init_seed = ChaCha20Rng::from_entropy().gen();

// Anchor block of the account
let anchor_block = client.get_latest_epoch_block().await.unwrap();

// Build the new `Account` with the component
let (counter_contract, counter_seed) = AccountBuilder::new(init_seed)
    .anchor((&anchor_block).try_into().unwrap())
    .account_type(AccountType::RegularAccountImmutableCode)
    .storage_mode(AccountStorageMode::Public)
    .with_component(counter_component.clone())
    .build()
    .unwrap();

println!(
    "counter_contract hash: {:?}",
    counter_contract.hash().to_hex()
);
println!("contract id: {:?}", counter_contract.id().to_hex());

// Since anyone should be able to write to the counter contract, auth_secret_key is not required.
// However, to import to the client, we must generate a random value.
let (_counter_pub_key, auth_secret_key) = get_new_pk_and_authenticator();

client
    .add_account(
        &counter_contract.clone(),
        Some(counter_seed),
        &auth_secret_key,
        false,
    )
    .await
    .unwrap();
```

Run the following command to execute src/main.rs:

```bash
cargo run --release 
```

After the program executes, you should see the counter contract hash and contract id printed to the terminal, for example:

```
counter_contract hash: "0xd693494753f51cb73a436916077c7b71c680a6dddc64dc364c1fe68f16f0c087"
contract id: "0x082ed14c8ad9a866"
```

## Step 4: Computing the prodedure hashes

Each Miden assembly procedure has an associated hash. When calling a procedure in a smart contract, we need to know the hash of the procedure. The hashes of the procedures form a [Merkelized Abstract Syntax Tree (MAST).](https://0xpolygonmiden.github.io/miden-vm/design/programs.html)

To get the hash of the `increment_count` procedure, add this code snippet to the end of your `main()` function:

```rust
// Print the procedure root hash
let get_increment_export = counter_component
    .library()
    .exports()
    .find(|export| export.name.as_str() == "increment_count")
    .unwrap();

let get_increment_count_mast_id = counter_component
    .library()
    .get_export_node_id(get_increment_export);

let increment_count_hash = counter_component
    .library()
    .mast_forest()
    .get_node_by_id(get_increment_count_mast_id)
    .unwrap()
    .digest().to_hex();

println!("increment_count procedure hash: {:?}", increment_count_hash);
```

Run the following command to execute src/main.rs:

```
cargo run --release 
```

After the program executes, you should see the procedure hashes printed to the terminal, for example:
```
increment_count procedure hash: "0xecd7eb223a5524af0cc78580d96357b298bb0b3d33fe95aeb175d6dab9de2e54"
```

This is the hash of the `increment_count` procedure.

## Step 4: Incrementing the count

Now that we know the hash of the `increment_count` procedure, we can call the procedure in the counter contract. In the Rust code below, we replace the `{increment_count}` string with the hash of the `increment_count` procedure. 

Then we create a new transaction request with our custom script, and then pass the transaction request to the client. 

Paste the following code at the end of your `src/main.rs` file:

```rust
// -------------------------------------------------------------------------
// STEP 2: Call the Counter Contract with a script
// -------------------------------------------------------------------------
println!("\n[STEP 2] Call Counter Contract With Script");

// Load the MASM script referencing the increment procedure
let file_path = Path::new("./masm/scripts/counter_script.masm");
let original_code = fs::read_to_string(file_path).unwrap();

// Replace the placeholder with the actual procedure call
let replaced_code = original_code.replace("{increment_count}", &increment_count_hash);
println!("Final script:\n{}", replaced_code);

// Compile the script referencing our procedure
let tx_script = client.compile_tx_script(vec![], &replaced_code).unwrap();

// Build a transaction request with the custom script
let tx_increment_request = TransactionRequestBuilder::new()
    .with_custom_script(tx_script)
    .unwrap()
    .build();

// Execute the transaction locally
let tx_result = client
    .new_transaction(counter_contract.id(), tx_increment_request)
    .await
    .unwrap();

let tx_id = tx_result.executed_transaction().id();
println!(
    "View transaction on MidenScan: https://testnet.midenscan.com/tx/{:?}",
    tx_id
);

// Submit transaction to the network
let _ = client.submit_transaction(tx_result).await;

// Wait, then re-sync
tokio::time::sleep(Duration::from_secs(3)).await;
client.sync_state().await.unwrap();

// Retrieve updated contract data to see the incremented counter
let account = client.get_account(counter_contract.id()).await.unwrap();
println!(
    "counter contract storage: {:?}",
    account.unwrap().account().storage().get_item(0)
);
```

**Note**: *Once our counter contract is deployed, other users can increment the count of the smart contract simply by knowing the account id of the contract and the procedure hash of the `increment_count` procedure.*

## Summary

The final `src/main.rs` file should look like this:

```rust
use std::{fs, path::Path, sync::Arc};

use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use tokio::time::Duration;

use miden_client::{
    account::{AccountStorageMode, AccountType},
    crypto::RpoRandomCoin,
    rpc::{Endpoint, TonicRpcClient},
    store::{sqlite_store::SqliteStore, StoreAuthenticator},
    transaction::{TransactionKernel, TransactionRequestBuilder},
    Client, ClientError, Felt,
};

use miden_objects::{
    account::{AccountBuilder, AccountComponent, AuthSecretKey, StorageSlot},
    assembly::Assembler,
    crypto::dsa::rpo_falcon512::SecretKey,
    Word,
};

pub async fn initialize_client() -> Result<Client<RpoRandomCoin>, ClientError> {
    // RPC endpoint and timeout
    let endpoint = Endpoint::new(
        "https".to_string(),
        "rpc.testnet.miden.io".to_string(),
        Some(443),
    );
    let timeout_ms = 10_000;

    // Build RPC client
    let rpc_api = Box::new(TonicRpcClient::new(endpoint, timeout_ms));

    // Seed RNG
    let mut seed_rng = rand::thread_rng();
    let coin_seed: [u64; 4] = seed_rng.gen();

    // Create random coin instance
    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));

    // SQLite path
    let store_path = "store.sqlite3";

    // Initialize SQLite store
    let store = SqliteStore::new(store_path.into())
        .await
        .map_err(ClientError::StoreError)?;
    let arc_store = Arc::new(store);

    // Create authenticator referencing the store and RNG
    let authenticator = StoreAuthenticator::new_with_rng(arc_store.clone(), rng.clone());

    // Instantiate client (toggle debug mode as needed)
    let client = Client::new(rpc_api, rng, arc_store, Arc::new(authenticator), true);

    Ok(client)
}

pub fn get_new_pk_and_authenticator() -> (Word, AuthSecretKey) {
    // Create a deterministic RNG with zeroed seed
    let seed = [0_u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    // Generate Falcon-512 secret key
    let sec_key = SecretKey::with_rng(&mut rng);

    // Convert public key to `Word` (4xFelt)
    let pub_key: Word = sec_key.public_key().into();

    // Wrap secret key in `AuthSecretKey`
    let auth_secret_key = AuthSecretKey::RpoFalcon512(sec_key);

    (pub_key, auth_secret_key)
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // Initialize client
    let mut client = initialize_client().await?;
    println!("Client initialized successfully.");

    // Fetch latest block from node
    let sync_summary = client.sync_state().await.unwrap();
    println!("Latest block: {}", sync_summary.block_num);

    // -------------------------------------------------------------------------
    // STEP 1: Create a basic counter contract
    // -------------------------------------------------------------------------
    println!("\n[STEP 1] Creating counter contract.");

    // Load the MASM file for the counter contract
    let file_path = Path::new("./masm/accounts/counter.masm");
    let account_code = fs::read_to_string(file_path).unwrap();

    // Prepare assembler (debug mode = true)
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);

    // Compile the account code into `AccountComponent` with one storage slot
    let counter_component = AccountComponent::compile(
        account_code,
        assembler,
        vec![StorageSlot::Value([
            Felt::new(0),
            Felt::new(0),
            Felt::new(0),
            Felt::new(0),
        ])],
    )
    .unwrap()
    .with_supports_all_types();

    // Init seed for the counter contract
    let init_seed = ChaCha20Rng::from_entropy().gen();

    // Anchor block of the account
    let anchor_block = client.get_latest_epoch_block().await.unwrap();

    // Build the new `Account` with the component
    let (counter_contract, counter_seed) = AccountBuilder::new(init_seed)
        .anchor((&anchor_block).try_into().unwrap())
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_component(counter_component.clone())
        .build()
        .unwrap();

    println!(
        "counter_contract hash: {:?}",
        counter_contract.hash().to_hex()
    );
    println!("contract id: {:?}", counter_contract.id().to_hex());
    println!("account_storage: {:?}", counter_contract.storage());

    // Since anyone should be able to write to the counter contract, auth_secret_key is not required.
    // However, to import to the client, we must generate a random value.
    let (_counter_pub_key, auth_secret_key) = get_new_pk_and_authenticator();

    client
        .add_account(
            &counter_contract.clone(),
            Some(counter_seed),
            &auth_secret_key,
            false,
        )
        .await
        .unwrap();

    // Print the procedure root hash
    let get_increment_export = counter_component
        .library()
        .exports()
        .find(|export| export.name.as_str() == "increment_count")
        .unwrap();

    let get_increment_count_mast_id = counter_component
        .library()
        .get_export_node_id(get_increment_export);

    let increment_count_hash = counter_component
        .library()
        .mast_forest()
        .get_node_by_id(get_increment_count_mast_id)
        .unwrap()
        .digest()
        .to_hex();

    println!("increment_count procedure hash: {:?}", increment_count_hash);

    // -------------------------------------------------------------------------
    // STEP 2: Call the Counter Contract with a script
    // -------------------------------------------------------------------------
    println!("\n[STEP 2] Call Counter Contract With Script");

    // Load the MASM script referencing the increment procedure
    let file_path = Path::new("./masm/scripts/counter_script.masm");
    let original_code = fs::read_to_string(file_path).unwrap();

    // Replace the placeholder with the actual procedure call
    let replaced_code = original_code.replace("{increment_count}", &increment_count_hash);
    println!("Final script:\n{}", replaced_code);

    // Compile the script referencing our procedure
    let tx_script = client.compile_tx_script(vec![], &replaced_code).unwrap();

    // Build a transaction request with the custom script
    let tx_increment_request = TransactionRequestBuilder::new()
        .with_custom_script(tx_script)
        .unwrap()
        .build();

    // Execute the transaction locally
    let tx_result = client
        .new_transaction(counter_contract.id(), tx_increment_request)
        .await
        .unwrap();

    let tx_id = tx_result.executed_transaction().id();
    println!(
        "View transaction on MidenScan: https://testnet.midenscan.com/tx/{:?}",
        tx_id
    );

    // Submit transaction to the network
    let _ = client.submit_transaction(tx_result).await;

    // Wait, then re-sync
    tokio::time::sleep(Duration::from_secs(3)).await;
    client.sync_state().await.unwrap();

    // Retrieve updated contract data to see the incremented counter
    let account = client.get_account(counter_contract.id()).await.unwrap();
    println!(
        "counter contract storage: {:?}",
        account.unwrap().account().storage().get_item(0)
    );

    Ok(())
}
```

The output of our program will look something like this:

```
Client initialized successfully.
Latest block: 118178

[STEP 1] Creating counter contract.
counter_contract hash: "0xa1802c8cfba2bd9c1c0f0b10b875795445566bd61864a05103bdaff167775293"
contract id: "0x4eedb9db1bdcf90000036bcebfe53a"
account_storage: AccountStorage { slots: [Value([0, 0, 0, 0])] }
increment_count procedure hash: "0xecd7eb223a5524af0cc78580d96357b298bb0b3d33fe95aeb175d6dab9de2e54"

[STEP 2] Call Counter Contract With Script
Final script:
begin
    # => []
    call.0xecd7eb223a5524af0cc78580d96357b298bb0b3d33fe95aeb175d6dab9de2e54
end
Stack state before step 2384:
├──  0: 1
├──  1: 0
├──  2: 0
├──  3: 0
├──  4: 0
├──  5: 0
├──  6: 0
├──  7: 0
├──  8: 0
├──  9: 0
├── 10: 0
├── 11: 0
├── 12: 0
├── 13: 0
├── 14: 0
├── 15: 0
├── 16: 0
├── 17: 0
├── 18: 0
└── 19: 0

View transaction on MidenScan: https://testnet.midenscan.com/tx/0x4384619bba7e6c959a31769a52ce8c6c081ffab00be33e85f58f62cccfd32c21
counter contract storage: Ok(RpoDigest([0, 0, 0, 1]))
```

The line in the output `Stack state before step 2384` ouputs the stack state when we call "debug.stack" in the `counter.masm` file.

To increment the count of the counter contract all you need is to know the account id of the counter and the procedure hash of the `increment_count` procedure. To increment the count without deploying the counter each time, you can modify the program above to hardcode the account id of the counter and the procedure hash of the `increment_count` prodedure in the masm script.

### Running the example

To run the full example, navigate to the `rust-client` directory in the [miden-tutorials](https://github.com/0xPolygonMiden/miden-tutorials/) repository and run this command:

```bash
cd rust-client
cargo run --release --bin counter_contract_deploy
```

### Continue learning

Next tutorial: [Interacting with Public Smart Contracts](public_account_interaction_tutorial.md)
