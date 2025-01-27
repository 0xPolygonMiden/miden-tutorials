# Mint, Consume, and Create Notes
*Using the Miden client in Rust to mint, consume, and create notes*

## Overview
In the previous section, we initialized our repository and covered how to create an account and deploy a faucet. In this section, we will mint tokens from the faucet for *Alice*, consume the newly created notes, and demonstrate how to send assets to other accounts.

## What we'll cover
* Minting tokens from a faucet
* Consuming notes to fund an account
* Sending tokens to other users

## Step 1: Minting tokens from the faucet
To mint notes with tokens from the faucet we created, Alice needs to call the faucet with a mint transaction request. 

*In essence, a transaction request is a structured template that outlines the data required to generate a zero-knowledge proof of a state change of an account. It specifies which input notes (if any) will be consumed, includes an optional transaction script to execute, and enumerates the set of notes expected to be created (if any).*

Below is an example of a transaction request minting tokens from the faucet for Alice. This code snippet will create 5 transaction mint transaction requests. 

Add this snippet to the end of your file in the `main()` function that we created in the previous chapter:
```rust
let amount: u64 = 100;
let fungible_asset = FungibleAsset::new(faucet_account.id(), amount).unwrap();

for _ in 0..5 {
    let transaction_request = TransactionRequest::mint_fungible_asset(
        fungible_asset.clone(), // fungible asset id
        alice_account.id(),     // target account id
        NoteType::Public,       // minted note type
        client.rng(),           // rng for the note serial number
    )
    .unwrap();

    let tx_execution_result = client
        .new_transaction(faucet_account.id(), transaction_request)
        .await?;

    client.submit_transaction(tx_execution_result).await?;
}
```

## Step 2: Identifying consumable notes
Once Alice has minted a note from the faucet, she will eventually want to spend the tokens that she received in the note created by the mint transaction. 

Minting a note from a faucet on Miden means a faucet account creates a new note targeted to the requesting account. The requesting account needs to consume this new note to have the assets appear in their account.

To identify consumable notes, the Miden client provides the `get_consumable_notes` function. Before calling it, ensure that the client state is synced.

*Tip: If you know how many notes to expect after a transaction, use an await or loop condition to check how many notes of the type you expect are available for consumption instead of using a set timeout before calling `get_consumable_notes`. This ensures your application isn't idle for longer than necessary.*

#### Identifying which notes are available:

```rust
let consumable_notes = client.get_consumable_notes(Some(alice_account.id())).await?;
```

## Step 3: Consuming multiple notes in a single transaction:
Now that we know how to identify notes ready to consume, let's consume the notes created by the faucet in a single transaction. After consuming the notes, Alice's wallet balance will be updated.

The following code snippet identifies consumable notes and consumes them in a single transaction.

Add this snippet to the end of your file in the `main()` function:
```Rust
loop {
    // Re-sync state to ensure we have the latest info
    client.sync_state().await?;

    // Fetch all consumable notes for Alice
    let consumable_notes = client.get_consumable_notes(Some(alice_account.id())).await?;
    let list_of_note_ids: Vec<_> = consumable_notes.iter().map(|(note, _)| note.id()).collect();

    if list_of_note_ids.len() == 5 {
        println!(
            "Alice has {} consumable notes. Consuming them now...",
            list_of_note_ids.len()
        );

        let transaction_request = TransactionRequest::consume_notes(list_of_note_ids);
        let tx_execution_result = client
            .new_transaction(alice_account.id(), transaction_request)
            .await?;

        client.submit_transaction(tx_execution_result).await?;
        println!("Successfully consumed all of Alice's notes.");
        break;
    } else {
        println!(
            "Currently, Alice has {} consumable notes. Waiting for 5",
            list_of_note_ids.len()
        );
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
```

## Step 4: Sending tokens to other accounts
After consuming the notes, Alice has tokens in her wallet. Now, she wants to send tokens to her friends. She has two options: create a separate transaction for each transfer or batch multiple transfers into a single transaction.

*The standard asset transfer note on Miden is the P2ID note (Pay to Id). There is also the P2IDR (Pay to Id Reclaimable) variant which allows the creator of the note to reclaim the note after a certain block height.*

In our example, Alice will now send 50 tokens to 5 different accounts.

For the sake of the example, the first four P2ID transfers are handled in a single transaction, and the fifth transfer is a standard P2ID transfer. 

### Output multiple P2ID notes in a single transaction
To output multiple notes in a single transaction we need to create a list of our expected output notes. The expected output notes are the notes that we expect to create in our transaction request.

In the snippet below, we create an empty vector to store five P2ID output notes, loop over five iterations `(using 0..=4)` to create five unique dummy account IDs, build a P2ID note for each one, and push each note onto the vector. Finally, we build a transaction request using `.with_own_output_notes()`—passing in all five notes—and submit it to the node.

Add this snippet to the end of your file in the `main()` function:
```Rust
let mut p2id_notes = vec![];
for _ in 0..=4 {
    // Generate a unique random seed based on the loop index `i`
    let init_seed = {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill(&mut seed);
        seed[0] = 0 as u8;
        seed
    };

    // Create a new dummy account ID
    let target_account_id =
        AccountId::new_dummy(init_seed, AccountType::RegularAccountUpdatableCode);

    // Specify send amount
    let send_amount = 50;
    let fungible_asset = FungibleAsset::new(faucet_account.id(), send_amount)
        .expect("Failed to create fungible asset for sending.");

    let p2id_note = create_p2id_note(
        alice_account.id(),
        target_account_id,
        vec![fungible_asset.into()],
        NoteType::Public,
        Felt::new(0),
        client.rng(),
    )
    .unwrap();

    p2id_notes.push(p2id_note);
}

let output_notes: Vec<OutputNote> = p2id_notes.into_iter().map(OutputNote::Full).collect();

let transaction_request = TransactionRequest::new()
    .with_own_output_notes(output_notes)
    .unwrap();

let tx_execution_result = client
    .new_transaction(alice_account.id(), transaction_request)
    .await?;

client.submit_transaction(tx_execution_result).await?;
```

### Basic P2ID transfer
Now as an example, Alice will send some tokens to an account in a single transaction.

Add this snippet to the end of your file in the `main()` function:
```Rust
// Generate a unique random seed
let init_seed = {
    let mut seed = [0u8; 32];
    rand::thread_rng().fill(&mut seed);
    seed[0] = 0 as u8;
    seed
};

// Create a new dummy account ID
let target_account_id =
    AccountId::new_dummy(init_seed, AccountType::RegularAccountUpdatableCode);

let send_amount = 50;
let fungible_asset = FungibleAsset::new(faucet_account.id(), send_amount).unwrap();

let payment_transaction = PaymentTransactionData::new(
    vec![fungible_asset.into()],
    alice_account.id(),
    target_account_id,
);

// Create a pay-to-id transaction
let transaction_request = TransactionRequest::pay_to_id(
    payment_transaction,
    None,             // recall_height: None
    NoteType::Public, // note type is public
    client.rng(),     // rng for the note serial number
)
.unwrap();

let tx_execution_result = client
    .new_transaction(alice_account.id(), transaction_request)
    .await?;

client.submit_transaction(tx_execution_result).await?;
```

Note: *In a production environment do not use `AccountId::new_dummy()`, this is simply for the sake of the tutorial example.*

## Summary
Your `src/main.rs` function should now look like this:
```rust
use miden_client::{
    accounts::{AccountId, AccountStorageMode, AccountTemplate, AccountType},
    assets::{FungibleAsset, TokenSymbol},
    config::RpcConfig,
    crypto::RpoRandomCoin,
    notes::NoteType,
    rpc::TonicRpcClient,
    store::{
        sqlite_store::{config::SqliteStoreConfig, SqliteStore},
        StoreAuthenticator,
    },
    transactions::{
        LocalTransactionProver, OutputNote, PaymentTransactionData, ProvingOptions,
        TransactionRequest,
    },
    Client, ClientError, Felt,
};
use miden_lib::notes::create_p2id_note;
use rand::Rng;
use std::sync::Arc;
use tokio::time::Duration;

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

    // Finally create the client
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
    println!("Client initialized successfully.");

    //------------------------------------------------------------
    // STEP 1: Create a basic wallet account for Alice
    //------------------------------------------------------------
    println!("\n[STEP 1] Creating new account for Alice");

    // Create a new account for Alice
    println!("Creating a new BasicWallet account for Alice...");
    let alice_template = AccountTemplate::BasicWallet {
        mutable_code: true,
        storage_mode: AccountStorageMode::Public,
    };
    let (alice_account, _alice_seed) = client.new_account(alice_template).await?;
    println!("Alice's account ID: {:?}", alice_account.id());

    //------------------------------------------------------------
    // STEP 2: Deploy a fungible faucet (token)
    //------------------------------------------------------------
    println!("\n[STEP 2] Deploying a new fungible faucet.");

    // Deploy a fungible faucet
    println!("Deploying a new fungible faucet...");
    let faucet_template = AccountTemplate::FungibleFaucet {
        token_symbol: TokenSymbol::new("MID").unwrap(),
        decimals: 8,
        max_supply: 1_000_000,
        storage_mode: AccountStorageMode::Public,
    };
    let (faucet_account, _faucet_seed) = client.new_account(faucet_template).await?;
    println!("Faucet account ID: {:?}", faucet_account.id());

    // Sync state to see newly deployed faucet
    client.sync_state().await?;
    tokio::time::sleep(Duration::from_secs(2)).await;

    //------------------------------------------------------------
    // STEP 3: Mint 5 notes of 100 tokens each for Alice
    //------------------------------------------------------------
    println!("\n[STEP 3] Minting 5 notes of 100 tokens each for Alice.");

    // Mint 5 notes of 100 tokens each for Alice
    println!("Minting 5 notes of 100 tokens each for Alice...");
    for i in 1..=5 {
        let amount: u64 = 100;
        let fungible_asset = FungibleAsset::new(faucet_account.id(), amount).unwrap();

        let transaction_request = TransactionRequest::mint_fungible_asset(
            fungible_asset.clone(),
            alice_account.id(),
            NoteType::Public,
            client.rng(),
        )?;
        let tx_execution_result = client
            .new_transaction(faucet_account.id(), transaction_request)
            .await?;

        client.submit_transaction(tx_execution_result).await?;
        println!("Minted note #{} of {} tokens for Alice.", i, amount);
    }
    println!("All 5 notes minted for Alice successfully!");

    // Re-sync so minted notes become visible in the client
    client.sync_state().await?;

    //------------------------------------------------------------
    // STEP 4: Alice consumes all her notes
    //------------------------------------------------------------
    println!("\n[STEP 4] Alice will now consume all of her notes to consolidate them.");

    // Consume all of Alice's minted notes in a single transaction
    println!("Checking for Alice's consumable notes to consume them...");
    loop {
        client.sync_state().await?;
        let consumable_notes = client
            .get_consumable_notes(Some(alice_account.id()))
            .await?;
        let list_of_note_ids: Vec<_> = consumable_notes.iter().map(|(note, _)| note.id()).collect();

        if list_of_note_ids.len() == 5 {
            println!("Found 5 consumable notes for Alice. Consuming them now...");
            let transaction_request = TransactionRequest::consume_notes(list_of_note_ids);
            let tx_execution_result = client
                .new_transaction(alice_account.id(), transaction_request)
                .await?;

            client.submit_transaction(tx_execution_result).await?;
            println!("All of Alice's notes consumed successfully.");
            break;
        } else {
            println!(
                "Currently, Alice has {} consumable notes. Waiting for 5...",
                list_of_note_ids.len()
            );
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }

    //------------------------------------------------------------
    // STEP 5: Using Alice's wallet, send 5 notes of 50 tokens each to list of users
    //------------------------------------------------------------
    println!("\n[STEP 5] Alice sends 5 notes of 50 tokens each to 5 different users.");

    // Send 50 tokens to 5 different accounts in a single transaction
    println!("Creating multiple P2ID notes for 5 target accounts in one transaction...");
    let mut p2id_notes = vec![];
    for _ in 1..=4 {
        // Generate a unique random seed
        let init_seed = {
            let mut seed = [0u8; 32];
            rand::thread_rng().fill(&mut seed);
            seed[0] = 99u8;
            seed
        };
        let target_account_id =
            AccountId::new_dummy(init_seed, AccountType::RegularAccountUpdatableCode);
        let send_amount = 50;
        let fungible_asset = FungibleAsset::new(faucet_account.id(), send_amount).unwrap();

        let p2id_note = create_p2id_note(
            alice_account.id(),
            target_account_id,
            vec![fungible_asset.into()],
            NoteType::Public,
            Felt::new(0),
            client.rng(),
        )?;
        p2id_notes.push(p2id_note);
    }
    let output_notes: Vec<OutputNote> = p2id_notes.into_iter().map(OutputNote::Full).collect();

    let transaction_request = TransactionRequest::new().with_own_output_notes(output_notes)?;

    let tx_execution_result = client
        .new_transaction(alice_account.id(), transaction_request)
        .await?;

    client.submit_transaction(tx_execution_result).await?;
    println!("Submitted a transaction with 4 P2ID notes.");

    // Send 50 tokens to 1 more account as a single P2ID transaction
    println!("Submitting one more single P2ID transaction...");
    let init_seed = {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill(&mut seed);
        seed[0] = 99u8;
        seed
    };
    let target_account_id =
        AccountId::new_dummy(init_seed, AccountType::RegularAccountUpdatableCode);
    let send_amount = 50;
    let fungible_asset = FungibleAsset::new(faucet_account.id(), send_amount).unwrap();
    let payment_transaction = PaymentTransactionData::new(
        vec![fungible_asset.into()],
        alice_account.id(),
        target_account_id,
    );
    let transaction_request = TransactionRequest::pay_to_id(
        payment_transaction,
        None,             // recall_height
        NoteType::Public, // note type
        client.rng(),     // rng for the note serial number
    )?;
    let tx_execution_result = client
        .new_transaction(alice_account.id(), transaction_request)
        .await?;

    client.submit_transaction(tx_execution_result).await?;

    println!("\nAll steps completed successfully!");
    println!("Alice created a wallet, a faucet was deployed,");
    println!("5 notes of 100 tokens were minted to Alice, those notes were consumed,");
    println!("and then Alice sent 5 separate 50-token notes to 5 different users.");

    Ok(())
}
```

Let's run the `src/main.rs` program again:
```bash
cargo run --release 
```

The output will look like this:
```
[STEP 1] Creating new account for Alice
Creating a new BasicWallet account for Alice...
Alice's account ID: AccountId(1495148201262154012)

[STEP 2] Deploying a new fungible faucet.
Deploying a new fungible faucet...
Faucet account ID: AccountId(3394709781787022201)

[STEP 3] Minting 5 notes of 100 tokens each for Alice.
Minting 5 notes of 100 tokens each for Alice...
Minted note #1 of 100 tokens for Alice.
Minted note #2 of 100 tokens for Alice.
Minted note #3 of 100 tokens for Alice.
Minted note #4 of 100 tokens for Alice.
Minted note #5 of 100 tokens for Alice.
All 5 notes minted for Alice successfully!

[STEP 4] Alice will now consume all of her notes to consolidate them.
Checking for Alice's consumable notes to consume them...
Currently, Alice has 0 consumable notes. Waiting for 5...
Currently, Alice has 0 consumable notes. Waiting for 5...
Found 5 consumable notes for Alice. Consuming them now...
one or more warnings were emitted
All of Alice's notes consumed successfully.

[STEP 5] Alice sends 5 notes of 50 tokens each to 5 different users.
Creating multiple P2ID notes for 5 target accounts in one transaction...
Submitted a transaction with 4 P2ID notes.
Submitting one more single P2ID transaction...

All steps completed successfully!
Alice created a wallet, a faucet was deployed,
5 notes of 100 tokens were minted to Alice, those notes were consumed,
and then Alice sent 5 separate 50-token notes to 5 different users.
```

### Running the example
To run a full working example navigate to the `rust-client` directory in the [miden-tutorials](https://github.com/0xPolygonMiden/miden-tutorials/) repository and run this command:
```bash
cd rust-client
cargo run --release --bin create_mint_consume_send
```

### Continue learning
Next tutorial: [Deploying a Counter Contract](counter_contract_tutorial.md)