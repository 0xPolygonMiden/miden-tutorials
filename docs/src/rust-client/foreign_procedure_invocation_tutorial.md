# Foreign Procedure Invocation Tutorial

*Using foreign procedure invocation to craft read-only cross-contract calls in the Miden VM*

## Overview

In previous tutorials we deployed a public counter contract and incremented the count from a different client instance. 

In this tutorial we will cover the basics of "foreign procedure invocation" (FPI) in the Miden VM, by building a "Count Copy" smart contract that reads the count from our previously deployed counter contract and copies the count to its own local storage.

Foreign procedure invocation (FPI) is a powerful tool for building smart contracts in the Miden VM. FPI allows one smart contract to call "read-only" procedures in other smart contracts.

The term "foreign procedure invocation" might sound a bit verbose, but it is as simple as one smart contract calling a non-state modifying procedure in another smart contract. The "EVM equivalent" of foreign procedure invocation would be a smart contract calling a read-only function in another contract.

FPI is useful for developing smart contracts that extend the functionality of existing contracts on Miden. FPI is the core primitive used by price oracles on Miden.

## What we'll cover

- Foreign Procedure Invocation (FPI)
- Building a "Count Copy" Smart Contract

## Prerequisites

This tutorial assumes you have a basic understanding of Miden assembly and completed the previous tutorial on deploying the counter contract. We will be working within the same `miden-counter-contract` repository that we created in the [Interacting with Public Smart Contracts](./public_account_interaction_tutorial.md) tutorial.

## Step 1: Set up your repository

We will be using the same repository used in the "Interacting with Public Smart Contracts" tutorial. To set up your repository for this tutorial, first follow up until step two [here](./public_account_interaction_tutorial.md).

## Step 2: Set up the "count reader" contract

Inside of the `masm/accounts/` directory, create the `count_reader.masm` file. This is the smart contract that will read the "count" value from the counter contract.

`masm/accounts/count_reader.masm`:

```masm
use.miden::account
use.miden::tx
use.std::sys

# Reads the count from the counter contract 
# and then copies the value to storage
export.copy_count
  # => []
  push.{get_count_proc_hash}

  # => [GET_COUNT_HASH]
  push.{account_id_suffix}

  # => [account_id_suffix]
  push.{account_id_prefix}

  # => [account_id_prefix, account_id_suffix, GET_COUNT_HASH]
  exec.tx::execute_foreign_procedure

  # => [count]
  debug.stack

  # => [count]
  push.0 

  # [index, count]
  exec.account::set_item

  # => []
  push.1 exec.account::incr_nonce

  # => []
  exec.sys::truncate_stack
end
```

In the count reader smart contract we have a `copy_count` procedure that uses `tx::execute_foreign_procedure` to call the `get_count` procedure in the counter contract. 

To call the `get_count` procedure, we push its hash along with the counter contract's ID suffix and prefix.

This is what the stack state should look like before we call `tx::execute_foreign_procedure`:

```
# => [account_id_prefix, account_id_suffix, GET_COUNT_HASH]
```

After calling the `get_count` procedure in the counter contract, we call `debug.stack` and then save the count of the counter contract to index 0 in storage.


**Note**: *The bracket symbols used in the count copy contract are not valid MASM syntax. These are simply placeholder elements that we will replace with the actual values before compilation.*


Inside the `masm/scripts/` directory, create the `reader_script.masm` file:
```masm
begin
    # => []
    call.{copy_count}
end
```

### Step 3: Set up your `src/main.rs` file:

```rust
use std::{fs, path::Path, sync::Arc};

use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use tokio::time::Duration;

use miden_client::{
    account::{Account, AccountCode, AccountId, AccountStorageMode, AccountType},
    asset::AssetVault,
    crypto::RpoRandomCoin,
    rpc::{
        domain::account::{AccountDetails, AccountStorageRequirements},
        Endpoint, TonicRpcClient,
    },
    store::{sqlite_store::SqliteStore, StoreAuthenticator},
    transaction::{ForeignAccount, TransactionKernel, TransactionRequestBuilder},
    Client, ClientError, Felt,
};

use miden_objects::{
    account::{AccountBuilder, AccountComponent, AccountStorage, AuthSecretKey, StorageSlot},
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
    // STEP 1: Create the Count Reader Contract
    // -------------------------------------------------------------------------
    println!("\n[STEP 1] Creating count reader contract.");

    // Load the MASM file for the counter contract
    let file_path = Path::new("./masm/accounts/count_reader.masm");
    let raw_account_code = fs::read_to_string(file_path).unwrap();

    // Define the counter contract account id and `get_count` procedure hash
    let counter_contract_id = AccountId::from_hex("0x4eedb9db1bdcf90000036bcebfe53a").unwrap();
    let get_count_hash = "0x92495ca54d519eb5e4ba22350f837904d3895e48d74d8079450f19574bb84cb6";

    let count_reader_code = raw_account_code
        .replace("{get_count_proc_hash}", &get_count_hash)
        .replace(
            "{account_id_prefix}",
            &counter_contract_id.prefix().to_string(),
        )
        .replace(
            "{account_id_suffix}",
            &counter_contract_id.suffix().to_string(),
        );

    // Initialize assembler (debug mode = true)
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);

    // Compile the account code into `AccountComponent` with one storage slot
    let count_reader_component = AccountComponent::compile(
        count_reader_code,
        assembler,
        vec![StorageSlot::Value(Word::default())],
    )
    .unwrap()
    .with_supports_all_types();

    // Init seed for the count reader contract
    let init_seed = ChaCha20Rng::from_entropy().gen();

    // Using latest block as the anchor block
    let anchor_block = client.get_latest_epoch_block().await.unwrap();

    // Build the count reader contract with the component
    let (count_reader_contract, counter_seed) = AccountBuilder::new(init_seed)
        .anchor((&anchor_block).try_into().unwrap())
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_component(count_reader_component.clone())
        .build()
        .unwrap();

    println!(
        "count reader contract id: {:?}",
        count_reader_contract.id().to_hex()
    );
    println!(
        "count reader  storage: {:?}",
        count_reader_contract.storage()
    );

    // Since anyone should be able to write to the counter contract, auth_secret_key is not required.
    // However, to import to the client, we must generate a random value.
    let (_counter_pub_key, auth_secret_key) = get_new_pk_and_authenticator();

    client
        .add_account(
            &count_reader_contract.clone(),
            Some(counter_seed),
            &auth_secret_key,
            false,
        )
        .await
        .unwrap();

    // Getting the hash of the `copy_count` procedure
    let get_proc_export = count_reader_component
        .library()
        .exports()
        .find(|export| export.name.as_str() == "copy_count")
        .unwrap();

    let get_proc_mast_id = count_reader_component
        .library()
        .get_export_node_id(get_proc_export);

    let copy_count_proc_hash = count_reader_component
        .library()
        .mast_forest()
        .get_node_by_id(get_proc_mast_id)
        .unwrap()
        .digest()
        .to_hex();

    println!("copy_count procedure hash: {:?}", copy_count_proc_hash);
    Ok(())
}
```

Run the following command to execute src/main.rs:
```bash
cargo run --release 
```

The output of our program will look something like this:
```
Client initialized successfully.
Latest block: 243826

[STEP 1] Creating count reader contract.
count reader contract id: "0xa47d7e5d8b1b90000003cd45a45a78"
count reader  storage: AccountStorage { slots: [Value([0, 0, 0, 0])] }
copy_count procedure hash: "0xa2ab9f6a150e9c598699741187589d0c61de12c35c1bbe591d658950f44ab743"
```

## Step 4: Build and read the state of the counter contract deployed on testnet

Add this snippet to the end of your file in the `main()` function that we created in the previous step:

```rust
// -------------------------------------------------------------------------
// STEP 2: Build & Get State of the Counter Contract
// -------------------------------------------------------------------------
println!("\n[STEP 2] Building counter contract from public state");

// Define the Counter Contract account id from counter contract deploy
let account_details = client
    .test_rpc_api()
    .get_account_update(counter_contract_id)
    .await
    .unwrap();

let AccountDetails::Public(counter_contract_details, _) = account_details else {
    panic!("counter contract must be public");
};

// Getting the value of the count from slot 0 and the nonce of the counter contract
let count_value = counter_contract_details.storage().slots().get(0).unwrap();
let counter_nonce = counter_contract_details.nonce();

println!("count val: {:?}", count_value.value());
println!("counter nonce: {:?}", counter_nonce);

// Load the MASM file for the counter contract
let file_path = Path::new("./masm/accounts/counter.masm");
let account_code = fs::read_to_string(file_path).unwrap();

// Prepare assembler (debug mode = true)
let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);

// Compile the account code into `AccountComponent` with the count value returned by the node
let counter_component = AccountComponent::compile(
    account_code,
    assembler,
    vec![StorageSlot::Value(count_value.value())],
)
.unwrap()
.with_supports_all_types();

// Initialize the AccountStorage with the count value returned by the node
let counter_storage =
    AccountStorage::new(vec![StorageSlot::Value(count_value.value())]).unwrap();

// Build AccountCode from components
let counter_code = AccountCode::from_components(
    &[counter_component.clone()],
    AccountType::RegularAccountImmutableCode,
)
.unwrap();

// The counter contract doesn't have any assets so we pass an empty vector
let vault = AssetVault::new(&[]).unwrap();

// Build the counter contract from parts
let counter_contract = Account::from_parts(
    counter_contract_id,
    vault,
    counter_storage,
    counter_code,
    counter_nonce,
);

// Since anyone should be able to write to the counter contract, auth_secret_key is not required.
// However, to import to the client, we must generate a random value.
let (_, auth_secret_key) = get_new_pk_and_authenticator();

client
    .add_account(&counter_contract.clone(), None, &auth_secret_key, true)
    .await
    .unwrap();
```

This step uses the logic we explained in the [Public Account Interaction Tutorial](./public_account_interaction_tutorial.md) to read the state of the Counter contract and import it to the client locally.

## Step 5: Call the counter contract via foreign procedure invocation

Add this snippet to the end of your file in the `main()` function:

```rust
// -------------------------------------------------------------------------
// STEP 3: Call the Counter Contract via Foreign Procedure Invocation (FPI)
// -------------------------------------------------------------------------
println!("\n[STEP 3] Call Counter Contract with FPI from Count Copy Contract");

// Load the MASM script referencing the increment procedure
let file_path = Path::new("./masm/scripts/reader_script.masm");
let original_code = fs::read_to_string(file_path).unwrap();

// Replace {get_count} and {account_id}
let replaced_code = original_code.replace("{copy_count}", &copy_count_proc_hash);

// Compile the script referencing our procedure
let tx_script = client.compile_tx_script(vec![], &replaced_code).unwrap();

let foreign_account =
    ForeignAccount::public(counter_contract_id, AccountStorageRequirements::default()).unwrap();

// Build a transaction request with the custom script
let tx_request = TransactionRequestBuilder::new()
    .with_foreign_accounts([foreign_account])
    .with_custom_script(tx_script)
    .unwrap()
    .build();

// Execute the transaction locally
let tx_result = client
    .new_transaction(count_reader_contract.id(), tx_request)
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
let account_1 = client.get_account(counter_contract.id()).await.unwrap();
println!(
    "counter contract storage: {:?}",
    account_1.unwrap().account().storage().get_item(0)
);

let account_2 = client
    .get_account(count_reader_contract.id())
    .await
    .unwrap();
println!(
    "count reader contract storage: {:?}",
    account_2.unwrap().account().storage().get_item(0)
);
```

The key here is the use of the `.with_foreign_accounts()` method on the  `TransactionRequestBuilder`. Using this method, it is possible to create transactions with multiple foreign procedure calls.

## Summary

In this tutorial created a smart contract that calls the `get_count` procedure in the counter contract using foreign procedure invocation, and then saves the returned value to its local storage.

The final `src/main.rs` file should look like this:

```rust
use std::{fs, path::Path, sync::Arc};

use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use tokio::time::Duration;

use miden_client::{
    account::{Account, AccountCode, AccountId, AccountStorageMode, AccountType},
    asset::AssetVault,
    crypto::RpoRandomCoin,
    rpc::{
        domain::account::{AccountDetails, AccountStorageRequirements},
        Endpoint, TonicRpcClient,
    },
    store::{sqlite_store::SqliteStore, StoreAuthenticator},
    transaction::{ForeignAccount, TransactionKernel, TransactionRequestBuilder},
    Client, ClientError, Felt,
};

use miden_objects::{
    account::{AccountBuilder, AccountComponent, AccountStorage, AuthSecretKey, StorageSlot},
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
    // STEP 1: Create the Count Reader Contract
    // -------------------------------------------------------------------------
    println!("\n[STEP 1] Creating count reader contract.");

    // Load the MASM file for the counter contract
    let file_path = Path::new("./masm/accounts/count_reader.masm");
    let raw_account_code = fs::read_to_string(file_path).unwrap();

    // Define the counter contract account id and `get_count` procedure hash
    let counter_contract_id = AccountId::from_hex("0x4eedb9db1bdcf90000036bcebfe53a").unwrap();
    let get_count_hash = "0x92495ca54d519eb5e4ba22350f837904d3895e48d74d8079450f19574bb84cb6";

    let count_reader_code = raw_account_code
        .replace("{get_count_proc_hash}", &get_count_hash)
        .replace(
            "{account_id_prefix}",
            &counter_contract_id.prefix().to_string(),
        )
        .replace(
            "{account_id_suffix}",
            &counter_contract_id.suffix().to_string(),
        );

    // Initialize assembler (debug mode = true)
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);

    // Compile the account code into `AccountComponent` with one storage slot
    let count_reader_component = AccountComponent::compile(
        count_reader_code,
        assembler,
        vec![StorageSlot::Value(Word::default())],
    )
    .unwrap()
    .with_supports_all_types();

    // Init seed for the count reader contract
    let init_seed = ChaCha20Rng::from_entropy().gen();

    // Using latest block as the anchor block
    let anchor_block = client.get_latest_epoch_block().await.unwrap();

    // Build the count reader contract with the component
    let (count_reader_contract, counter_seed) = AccountBuilder::new(init_seed)
        .anchor((&anchor_block).try_into().unwrap())
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_component(count_reader_component.clone())
        .build()
        .unwrap();

    println!(
        "count reader contract id: {:?}",
        count_reader_contract.id().to_hex()
    );
    println!(
        "count reader  storage: {:?}",
        count_reader_contract.storage()
    );

    // Since anyone should be able to write to the counter contract, auth_secret_key is not required.
    // However, to import to the client, we must generate a random value.
    let (_counter_pub_key, auth_secret_key) = get_new_pk_and_authenticator();

    client
        .add_account(
            &count_reader_contract.clone(),
            Some(counter_seed),
            &auth_secret_key,
            false,
        )
        .await
        .unwrap();

    // Getting the hash of the `copy_count` procedure
    let get_proc_export = count_reader_component
        .library()
        .exports()
        .find(|export| export.name.as_str() == "copy_count")
        .unwrap();

    let get_proc_mast_id = count_reader_component
        .library()
        .get_export_node_id(get_proc_export);

    let copy_count_proc_hash = count_reader_component
        .library()
        .mast_forest()
        .get_node_by_id(get_proc_mast_id)
        .unwrap()
        .digest()
        .to_hex();

    println!("copy_count procedure hash: {:?}", copy_count_proc_hash);

    // -------------------------------------------------------------------------
    // STEP 2: Build & Get State of the Counter Contract
    // -------------------------------------------------------------------------
    println!("\n[STEP 2] Building counter contract from public state");

    // Define the Counter Contract account id from counter contract deploy
    let account_details = client
        .test_rpc_api()
        .get_account_update(counter_contract_id)
        .await
        .unwrap();

    let AccountDetails::Public(counter_contract_details, _) = account_details else {
        panic!("counter contract must be public");
    };

    // Getting the value of the count from slot 0 and the nonce of the counter contract
    let count_value = counter_contract_details.storage().slots().get(0).unwrap();
    let counter_nonce = counter_contract_details.nonce();

    println!("count val: {:?}", count_value.value());
    println!("counter nonce: {:?}", counter_nonce);

    // Load the MASM file for the counter contract
    let file_path = Path::new("./masm/accounts/counter.masm");
    let account_code = fs::read_to_string(file_path).unwrap();

    // Prepare assembler (debug mode = true)
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);

    // Compile the account code into `AccountComponent` with the count value returned by the node
    let counter_component = AccountComponent::compile(
        account_code,
        assembler,
        vec![StorageSlot::Value(count_value.value())],
    )
    .unwrap()
    .with_supports_all_types();

    // Initialize the AccountStorage with the count value returned by the node
    let counter_storage =
        AccountStorage::new(vec![StorageSlot::Value(count_value.value())]).unwrap();

    // Build AccountCode from components
    let counter_code = AccountCode::from_components(
        &[counter_component.clone()],
        AccountType::RegularAccountImmutableCode,
    )
    .unwrap();

    // The counter contract doesn't have any assets so we pass an empty vector
    let vault = AssetVault::new(&[]).unwrap();

    // Build the counter contract from parts
    let counter_contract = Account::from_parts(
        counter_contract_id,
        vault,
        counter_storage,
        counter_code,
        counter_nonce,
    );

    // Since anyone should be able to write to the counter contract, auth_secret_key is not required.
    // However, to import to the client, we must generate a random value.
    let (_, auth_secret_key) = get_new_pk_and_authenticator();

    client
        .add_account(&counter_contract.clone(), None, &auth_secret_key, true)
        .await
        .unwrap();

    // -------------------------------------------------------------------------
    // STEP 3: Call the Counter Contract via Foreign Procedure Invocation (FPI)
    // -------------------------------------------------------------------------
    println!("\n[STEP 3] Call Counter Contract with FPI from Count Copy Contract");

    // Load the MASM script referencing the increment procedure
    let file_path = Path::new("./masm/scripts/reader_script.masm");
    let original_code = fs::read_to_string(file_path).unwrap();

    // Replace {get_count} and {account_id}
    let replaced_code = original_code.replace("{copy_count}", &copy_count_proc_hash);

    // Compile the script referencing our procedure
    let tx_script = client.compile_tx_script(vec![], &replaced_code).unwrap();

    let foreign_account =
        ForeignAccount::public(counter_contract_id, AccountStorageRequirements::default()).unwrap();

    // Build a transaction request with the custom script
    let tx_request = TransactionRequestBuilder::new()
        .with_foreign_accounts([foreign_account])
        .with_custom_script(tx_script)
        .unwrap()
        .build();

    // Execute the transaction locally
    let tx_result = client
        .new_transaction(count_reader_contract.id(), tx_request)
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
    let account_1 = client.get_account(counter_contract.id()).await.unwrap();
    println!(
        "counter contract storage: {:?}",
        account_1.unwrap().account().storage().get_item(0)
    );

    let account_2 = client
        .get_account(count_reader_contract.id())
        .await
        .unwrap();
    println!(
        "count reader contract storage: {:?}",
        account_2.unwrap().account().storage().get_item(0)
    );

    Ok(())
}
```

The output of our program will look something like this:

```
Client initialized successfully.
Latest block: 242367

[STEP 1] Creating count reader contract.
count reader contract id: "0x95b00b4f410f5000000383ca114c9a"
count reader  storage: AccountStorage { slots: [Value([0, 0, 0, 0])] }
copy_count procedure hash: "0xa2ab9f6a150e9c598699741187589d0c61de12c35c1bbe591d658950f44ab743"

[STEP 2] Building counter contract from public state
count val: [0, 0, 0, 2]
counter nonce: 2

[STEP 3] Call Counter Contract with FPI from Count Copy Contract
Stack state before step 3351:
├──  0: 2
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
└── 15: 0

View transaction on MidenScan: https://testnet.midenscan.com/tx/0xe2fdb53926e7a11548863c2a85d81127094d6fe38f60509b4ef8ea38994f8cec
counter contract storage: Ok(RpoDigest([0, 0, 0, 2]))
count reader contract storage: Ok(RpoDigest([0, 0, 0, 2]))
```

### Running the example

To run the full example, navigate to the `rust-client` directory in the [miden-tutorials](https://github.com/0xPolygonMiden/miden-tutorials/) repository and run this command:

```bash
cd rust-client
cargo run --release --bin counter_contract_fpi
```
