# Leveraging Ephemeral Notes for Fast Transaction Settlement

## Overview

In this guide, we will explore how to leverage ephemeral notes on Miden to settle transactions faster than the standard blocktime of 5 seconds. Ephemeral notes are essentially UTXOs that have not yet been fully committed into a block. This feature allows them to be created and consumed within the same block, enabling rapid asset transfers. On a MacBook M4 chip, for example, these transactions can be processed in around 2 seconds. 

In this guide, we construct a chain of transactions using the unauthenticated notes method on the transaction builder. Ephemeral notes—also known as "unauthenticated notes", enable rapid note creation and consumption. We also show how a note can be serialized and deserialized, which demonstrates the possibility of transferring notes between client instances for extremely fast settlement.

For example, our demo creates a circle of ephemeral note transactions:

```markdown
Alice ➡ Bob ➡ Charlie ➡ Dave ➡ Eve ➡ Frank ➡ ...
```

## What We'll Cover

- **Introduction to Ephemeral Notes:** Understand what ephemeral notes are and how they differ from standard notes.
- **Serialization Example:** See how to serialize and deserialize a note to simulate rapid asset transfers.
- **Performance Insights:** Observe how ephemeral notes can reduce transaction times dramatically.

## Step-by-Step Process

1. **Client Initialization:**
   - Set up an RPC client to connect with the Miden testnet.
   - Initialize a random coin generator and a store for persisting account data.

2. **Deploying a Fungible Faucet:**
   - Use a random seed to deploy a fungible faucet.
   - Configure the faucet parameters (symbol, decimals, and max supply) and add it to the client.

3. **Creating Wallet Accounts:**
   - Build multiple wallet accounts using a secure key generation process.
   - Add these accounts to the client, making them ready for transactions.

4. **Minting and Transacting with Ephemeral Notes:**
   - Mint tokens for one of the accounts (Alice) from the deployed faucet.
   - Create a note representing the minted tokens.
   - Build and submit a transaction that uses the ephemeral note via the "unauthenticated" method.
   - Serialize the note to demonstrate how it could be transferred to another client instance.
   - Consume the note in a subsequent transaction, effectively creating a chain of ephemeral transactions.

5. **Performance Timing and Syncing:**
   - Measure the time taken for each transaction iteration.
   - Sync the client state and print account balances to verify the transactions.

## Full Rust Code Example

```rust
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;

use miden_client::{
    account::{
        component::{BasicFungibleFaucet, BasicWallet, RpoFalcon512},
        AccountBuilder, AccountStorageMode, AccountType,
    },
    asset::{FungibleAsset, TokenSymbol},
    crypto::RpoRandomCoin,
    note::{create_p2id_note, Note, NoteType},
    rpc::{Endpoint, TonicRpcClient},
    store::{sqlite_store::SqliteStore, StoreAuthenticator},
    transaction::{OutputNote, TransactionRequestBuilder},
    utils::{Deserializable, Serializable},
    Client, ClientError, Felt,
};

use miden_objects::{account::AuthSecretKey, crypto::dsa::rpo_falcon512::SecretKey, Word};

pub async fn initialize_client() -> Result<Client<RpoRandomCoin>, ClientError> {
    // RPC endpoint and timeout
    let endpoint = Endpoint::new(
        "https".to_string(),
        "rpc.testnet.miden.io".to_string(),
        Some(443),
    );
    let timeout_ms = 10_000;

    let rpc_api = Box::new(TonicRpcClient::new(endpoint, timeout_ms));

    let mut seed_rng = rand::thread_rng();
    let coin_seed: [u64; 4] = seed_rng.gen();
    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));

    let store_path = "store.sqlite3";
    let store = SqliteStore::new(store_path.into())
        .await
        .map_err(ClientError::StoreError)?;
    let arc_store = Arc::new(store);
    let authenticator = StoreAuthenticator::new_with_rng(arc_store.clone(), rng.clone());

    let client = Client::new(rpc_api, rng, arc_store, Arc::new(authenticator), true);
    Ok(client)
}

pub fn get_new_pk_and_authenticator() -> (Word, AuthSecretKey) {
    let mut seed_rng = rand::thread_rng();
    let seed: [u8; 32] = seed_rng.gen();
    let mut rng = ChaCha20Rng::from_seed(seed);

    let sec_key = SecretKey::with_rng(&mut rng);
    let pub_key: Word = sec_key.public_key().into();
    let auth_secret_key = AuthSecretKey::RpoFalcon512(sec_key);

    (pub_key, auth_secret_key)
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // ===== Client Initialization =====
    let mut client = initialize_client().await?;
    println!("Client initialized successfully.");

    // Fetch latest block from node
    let sync_summary = client.sync_state().await.unwrap();
    println!("Latest block: {}", sync_summary.block_num);

    //------------------------------------------------------------
    // STEP 1: Deploy a fungible faucet
    //------------------------------------------------------------
    println!("\n[STEP 1] Deploying a new fungible faucet.");

    // Faucet seed
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    // Anchor block
    let anchor_block = client.get_latest_epoch_block().await.unwrap();

    // Faucet parameters
    let symbol = TokenSymbol::new("MID").unwrap();
    let decimals = 8;
    let max_supply = Felt::new(1_000_000);

    // Generate key pair
    let key_pair = SecretKey::with_rng(client.rng());

    // Build the account
    let builder = AccountBuilder::new(init_seed)
        .anchor((&anchor_block).try_into().unwrap())
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(AccountStorageMode::Public)
        .with_component(RpoFalcon512::new(key_pair.public_key()))
        .with_component(BasicFungibleFaucet::new(symbol, decimals, max_supply).unwrap());

    let (faucet_account, seed) = builder.build().unwrap();

    // Add the faucet to the client
    client
        .add_account(
            &faucet_account,
            Some(seed),
            &AuthSecretKey::RpoFalcon512(key_pair),
            false,
        )
        .await?;
    println!("Faucet account ID: {}", faucet_account.id().to_hex());

    // Resync to show newly deployed faucet
    tokio::time::sleep(Duration::from_secs(2)).await;
    client.sync_state().await?;

    //------------------------------------------------------------
    // STEP 2: Create basic wallet accounts
    //------------------------------------------------------------
    println!("\n[STEP 2] Creating new accounts");

    let mut accounts = vec![];
    let number_of_accounts = 10;

    for i in 0..number_of_accounts {
        let init_seed = ChaCha20Rng::from_entropy().gen();
        let key_pair = SecretKey::with_rng(client.rng());
        let builder = AccountBuilder::new(init_seed)
            .anchor((&anchor_block).try_into().unwrap())
            .account_type(AccountType::RegularAccountUpdatableCode)
            .storage_mode(AccountStorageMode::Public)
            .with_component(RpoFalcon512::new(key_pair.public_key()))
            .with_component(BasicWallet);

        let (account, seed) = builder.build().unwrap();
        accounts.push(account.clone());
        println!("account id {:?}: {}", i, account.id().to_hex());
        client
            .add_account(
                &account,
                Some(seed),
                &AuthSecretKey::RpoFalcon512(key_pair.clone()),
                true,
            )
            .await?;
    }

    // For demo purposes, Alice is the first account.
    let alice = &accounts[0];

    //------------------------------------------------------------
    // STEP 3: Mint and consume tokens for Alice
    //------------------------------------------------------------
    println!("\n[STEP 3] Mint tokens");
    println!("Minting tokens for Alice...");
    let amount: u64 = 100;
    let fungible_asset_mint_amount = FungibleAsset::new(faucet_account.id(), amount).unwrap();
    let transaction_request = TransactionRequestBuilder::mint_fungible_asset(
        fungible_asset_mint_amount.clone(),
        alice.id(),
        NoteType::Public,
        client.rng(),
    )
    .unwrap()
    .build();
    let tx_execution_result = client
        .new_transaction(faucet_account.id(), transaction_request)
        .await?;
    client
        .submit_transaction(tx_execution_result.clone())
        .await?;

    // The minted fungible asset is public so output is a `Full` note type
    let p2id_note: Note =
        if let OutputNote::Full(note) = tx_execution_result.created_notes().get_note(0) {
            note.clone()
        } else {
            panic!("Expected Full note type");
        };

    let transaction_request = TransactionRequestBuilder::new()
        .with_unauthenticated_input_notes([(p2id_note, None)])
        .build();
    let tx_execution_result = client
        .new_transaction(alice.id(), transaction_request)
        .await?;
    client.submit_transaction(tx_execution_result).await?;
    client.sync_state().await?;

    //------------------------------------------------------------
    // STEP 4: Create ephemeral note tx chain
    //------------------------------------------------------------
    println!("\n[STEP 4] Create ephemeral note tx chain");
    let mut landed_blocks = vec![];
    let start = Instant::now();

    for i in 0..number_of_accounts - 1 {
        let loop_start = Instant::now();
        println!("\nephemeral tx {:?}", i + 1);
        println!("sender: {}", accounts[i].id().to_hex());
        println!("target: {}", accounts[i + 1].id().to_hex());

        // Time the creation of the p2id note
        let send_amount = 20;
        let fungible_asset_send_amount =
            FungibleAsset::new(faucet_account.id(), send_amount).unwrap();

        // for demo purposes, ephemeral notes can be public or private
        let note_type = if i % 2 == 0 {
            NoteType::Private
        } else {
            NoteType::Public
        };

        let p2id_note = create_p2id_note(
            accounts[i].id(),
            accounts[i + 1].id(),
            vec![fungible_asset_send_amount.into()],
            note_type,
            Felt::new(0),
            client.rng(),
        )
        .unwrap();

        let output_note = OutputNote::Full(p2id_note.clone());

        // Time transaction request building
        let transaction_request = TransactionRequestBuilder::new()
            .with_own_output_notes(vec![output_note])
            .unwrap()
            .build();
        let tx_execution_result = client
            .new_transaction(accounts[i].id(), transaction_request)
            .await?;
        client.submit_transaction(tx_execution_result).await?;

        // Note serialization/deserialization
        // This demonstrates how you could send the serialized note to another client instance
        let serialized = p2id_note.to_bytes();
        let deserialized_p2id_note = Note::read_from_bytes(&serialized).unwrap();

        // Time consume note request building
        let consume_note_request =
            TransactionRequestBuilder::consume_notes(vec![deserialized_p2id_note.id()])
                .with_unauthenticated_input_notes([(deserialized_p2id_note, None)])
                .build();
        let tx_execution_result = client
            .new_transaction(accounts[i + 1].id(), consume_note_request)
            .await?;
        landed_blocks.push(tx_execution_result.block_num());
        client
            .submit_transaction(tx_execution_result.clone())
            .await?;

        println!(
            "Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/{:?}",
            tx_execution_result.executed_transaction().id()
        );
        println!(
            "Total time for loop iteration {}: {:?}",
            i,
            loop_start.elapsed()
        );
    }

    println!(
        "\nTotal execution time for ephemeral note txs: {:?}",
        start.elapsed()
    );
    println!("blocks: {:?}", landed_blocks);

    // Final resync and display account balances
    tokio::time::sleep(Duration::from_secs(3)).await;
    client.sync_state().await?;
    for account in accounts.clone() {
        let new_account = client.get_account(account.id()).await.unwrap().unwrap();
        let balance = new_account
            .account()
            .vault()
            .get_balance(faucet_account.id())
            .unwrap();
        println!("Account: {} balance: {}", account.id().to_hex(), balance);
    }

    Ok(())
}
```

The output of our program will look something like this:

```
Client initialized successfully.
Latest block: 402875

[STEP 1] Deploying a new fungible faucet.
Faucet account ID: 0x86c03aeb90b2e3200006852488eb50

[STEP 2] Creating new accounts
account id 0: 0x71c184dcaae5ee1000064e93777b70
account id 1: 0x74f3b6cdee937110000655e334161b
account id 2: 0x698ca2e2f7fc7010000643863b9f1a
account id 3: 0x032dd4e8fad68c100006b82d9ca4db
account id 4: 0x5bcca043b5de62100006f8db1610ab
account id 5: 0x6717bbdf75239c10000687c33ce06f
account id 6: 0x752fe4cebebfeb100006e7f9a3129c
account id 7: 0xc8ee0c3e68d384100006aeab3b063d
account id 8: 0x65c8d4a279bf0a100006e1519eca84
account id 9: 0xac0e06f781ac2d1000067663c3aadf

[STEP 3] Mint tokens
Minting tokens for Alice...
one or more warnings were emitted

[STEP 4] Create ephemeral note tx chain

ephemeral tx 1
sender: 0x71c184dcaae5ee1000064e93777b70
target: 0x74f3b6cdee937110000655e334161b
one or more warnings were emitted
Total time for loop iteration 0: 2.990357s
Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/0x11b361d0f0aaa1bbff909dcc0eaa5683afb0d2ad000e09a016a70e190bb8552f

ephemeral tx 2
sender: 0x74f3b6cdee937110000655e334161b
target: 0x698ca2e2f7fc7010000643863b9f1a
one or more warnings were emitted
Total time for loop iteration 1: 2.880536333s
Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/0x64122981b3405a6b307748473f849b22ae9615706a76145786c553c60de11d31

ephemeral tx 3
sender: 0x698ca2e2f7fc7010000643863b9f1a
target: 0x032dd4e8fad68c100006b82d9ca4db
one or more warnings were emitted
Total time for loop iteration 2: 3.203270708s
Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/0xcaeb762b744db5e2874ed33dd30333eb22a0f92117ba648c4894892e59425660

ephemeral tx 4
sender: 0x032dd4e8fad68c100006b82d9ca4db
target: 0x5bcca043b5de62100006f8db1610ab
one or more warnings were emitted
Total time for loop iteration 3: 3.189577792s
Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/0xfddc5b0c0668cb1caae144ca230aa5a99da07808f63df213a28fffe3d120ae52

ephemeral tx 5
sender: 0x5bcca043b5de62100006f8db1610ab
target: 0x6717bbdf75239c10000687c33ce06f
one or more warnings were emitted
Total time for loop iteration 4: 2.904180125s
Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/0x4d9d23018669aea665daf65dabaedbf1a6f957a4dc85c1012380dfa0a25f1e1f

ephemeral tx 6
sender: 0x6717bbdf75239c10000687c33ce06f
target: 0x752fe4cebebfeb100006e7f9a3129c
one or more warnings were emitted
Total time for loop iteration 5: 2.886588458s
Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/0x39be34b79aa24720c007fad3895e585239ca231f230b6e1ed5f4551319895fd9

ephemeral tx 7
sender: 0x752fe4cebebfeb100006e7f9a3129c
target: 0xc8ee0c3e68d384100006aeab3b063d
one or more warnings were emitted
Total time for loop iteration 6: 3.071692334s
Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/0x6c38d351b9c4d86b076e6a3e69a667a1ceac94157008d9a0e81ef8370a16c334

ephemeral tx 8
sender: 0xc8ee0c3e68d384100006aeab3b063d
target: 0x65c8d4a279bf0a100006e1519eca84
one or more warnings were emitted
Total time for loop iteration 7: 2.89388675s
Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/0x260e2ce59ddab76dc7b403b1003ff891ca2519dda1ee8cd8a9966507b955ff8b

ephemeral tx 9
sender: 0x65c8d4a279bf0a100006e1519eca84
target: 0xac0e06f781ac2d1000067663c3aadf
one or more warnings were emitted
Total time for loop iteration 8: 2.897855958s
Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/0x3cdc6659cd270e07137499d204270211dfda8c34aa3a80c3b6dc8064ac8cb09a

Total execution time for ephemeral note txs: 26.920523209s
blocks: [BlockNumber(402884), BlockNumber(402884), BlockNumber(402884), BlockNumber(402884), BlockNumber(402884), BlockNumber(402884), BlockNumber(402884), BlockNumber(402884), BlockNumber(402884)]
Account: 0x71c184dcaae5ee1000064e93777b70 balance: 80
Account: 0x74f3b6cdee937110000655e334161b balance: 0
Account: 0x698ca2e2f7fc7010000643863b9f1a balance: 0
Account: 0x032dd4e8fad68c100006b82d9ca4db balance: 0
Account: 0x5bcca043b5de62100006f8db1610ab balance: 0
Account: 0x6717bbdf75239c10000687c33ce06f balance: 0
Account: 0x752fe4cebebfeb100006e7f9a3129c balance: 0
Account: 0xc8ee0c3e68d384100006aeab3b063d balance: 0
Account: 0x65c8d4a279bf0a100006e1519eca84 balance: 0
Account: 0xac0e06f781ac2d1000067663c3aadf balance: 20
```

## Conclusion

Ephemeral notes on Miden offer a powerful mechanism for achieving faster asset settlements by allowing notes to be both created and consumed within the same block. In this guide, we walked through:

- **Minting and Transacting with Ephemeral Notes:** Building, serializing, and consuming notes quickly using the "unauthenticated" method.
- **Performance Observations:** Measuring and demonstrating the rapid transaction processing enabled by ephemeral notes.

By following this guide, you should now have a clear understanding of how to build and deploy high-performance transactions using ephemeral notes on Miden—an ideal approach for applications like central limit order books or other DeFi platforms where transaction speed is critical.

### Running the example

To run the ephemeral note transfer example, navigate to the `rust-client` directory in the [miden-tutorials](https://github.com/0xPolygonMiden/miden-tutorials/) repository and run this command:

```bash
cd rust-client
cargo run --release --bin ephemeral_note_transfer
```