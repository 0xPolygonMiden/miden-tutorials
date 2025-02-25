# How To Create a Custom Note on Miden

*Creating custom notes*

## Overview

In this guide, we will create a custom note on Miden that can only be consumed by someone who knows the preimage of the hash stored in the note. This approach securely embeds assets into the note and restricts spending to those who possess the correct secret number.

By following the steps below and using the Miden Assembly code and Rust example, you will learn how to:

* Create a note with custom logic.
* Leverage Miden’s privacy features to keep certain transaction details private.

Unlike Ethereum, where all transaction details are publicly visible in the mempool, Miden enables you to partially or completely hide transaction details.

## What we'll cover

- Creating custom note logic
- Consuming notes

## Step-by-Step Process

### 1. Creating Two Accounts: Alice & Bob

First, we create two basic accounts for the two users:
- **Alice:** The account that creates and funds the custom note.
- **Bob:** The account that will consume the note if they know the correct secret.

### 2. Hashing the Secret Number

The security of the custom note hinges on a secret number. Here, we will:
- Choose a secret number (for example, an array of four integers).
- For simplicity, we're only hashing 4 elements. Therefore, we prepend an empty word—consisting of 4 zero integers—as a placeholder. This is required by the RPO hashing algorithm to ensure the input has the correct structure and length for proper processing.
- Compute the hash of the secret. The resulting hash will serve as the note’s input, meaning that the note can only be consumed if the secret number’s hash preimage is provided during consumption.

### 3. Creating the Custom Note

Now, combine the minted asset and the secret hash to build the custom note. The note is created using the following key steps:

1. **Note Inputs:**  
   - The note is set up with the asset and the hash of the secret number as its input.
   
2. **Miden Assembly Code:**  
   - The Miden assembly note script ensures that the note can only be consumed if the provided secret, when hashed, matches the hash stored in the note input.

Below is the Miden Assembly code for the note:

```masm
use.miden::note
use.miden::contracts::wallets::basic->wallet

# => [HASH_PREIMAGE_SECRET]
begin

    # Hashing the secret number
    hperm
    # => [F,E,D]
    # E is digest
    
    dropw swapw dropw
    # => [DIGEST]
    
    # Writing the note inputs to memory
    push.0 exec.note::get_inputs drop drop
    # => [DIGEST]
    
    # Pad stack and load note inputs from memory
    padw push.0 mem_loadw
    # => [INPUTS, DIGEST]
    
    # Assert that the note input matches the digest
    # Will fail if the two hashes do not match
    assert_eqw
    # => []
    
    # Write the asset in note to memory address 0
    push.0 exec.note::get_assets
    # => [num_assets, dest_ptr]
    
    drop
    # => [dest_ptr]
    
    # Load asset from memory
    mem_loadw
    # => [ASSET]
    
    # Call receive asset in wallet
    call.wallet::receive_asset
    # => []
end
```

### How the Assembly Code Works:

1. **Passing the Secret:**  
   The secret number is passed as `Note Arguments` into the note.
   
2. **Hashing the Secret:**  
   The `hperm` instruction applies a hash permutation to the secret number, resulting in a hash that takes up four stack elements.
   
3. **Stack Cleanup and Comparison:**  
   The assembly code extracts the digest, loads the note inputs from memory and checks if the computed hash matches the note’s stored hash.
   
4. **Asset Transfer:**  
   If the hash of the number passed in as `Note Arguments` matches the hash stored in the note inputs, the script continues, and the asset stored in the note is loaded from memory and passed to Bob’s wallet via the `wallet::receive_asset` function.

### 5. Consuming the Note

With the note created, Bob can now consume it—but only if he provides the correct secret. When Bob initiates the transaction to consume the note, he must supply the same secret number used when Alice created the note. The custom note’s logic will hash the secret and compare it with its stored hash. If they match, Bob’s wallet receives the asset.

---

## Full Rust Code Example

The following Rust code demonstrates how to implement the steps outlined above using the Miden client library:

```rust
use std::{fs, path::Path, sync::Arc};

use rand::Rng;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

use miden_client::{
    account::{
        component::{BasicWallet, RpoFalcon512}, AccountBuilder, AccountId, AccountStorageMode, AccountType
    },
    asset::FungibleAsset,
    crypto::RpoRandomCoin,
    note::{
        Note, NoteAssets, NoteExecutionHint, NoteExecutionMode, NoteInputs, NoteMetadata,
        NoteRecipient, NoteScript, NoteTag, NoteType,
    },
    rpc::{Endpoint, TonicRpcClient},
    store::{sqlite_store::SqliteStore, StoreAuthenticator},
    transaction::{OutputNote, TransactionKernel, TransactionRequestBuilder},
    Client, ClientError, Felt,
};

use miden_crypto::{hash::rpo::Rpo256 as Hasher, rand::FeltRng};

use miden_objects::{
    account::AuthSecretKey, assembly::Assembler, crypto::dsa::rpo_falcon512::SecretKey, Word,
};

// Initialize client helper
pub async fn initialize_client() -> Result<Client<RpoRandomCoin>, ClientError> {
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

// Helper to create keys & authenticator
pub fn get_new_pk_and_authenticator() -> (Word, AuthSecretKey) {
    let seed = [0_u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    let sec_key = SecretKey::with_rng(&mut rng);
    let pub_key: Word = sec_key.public_key().into();
    let auth_secret_key = AuthSecretKey::RpoFalcon512(sec_key);

    (pub_key, auth_secret_key)
}

// Helper to create a basic account (for Alice and Bob)
async fn create_basic_account(
    client: &mut Client<RpoRandomCoin>,
) -> Result<miden_client::account::Account, ClientError> {
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);
    let key_pair = SecretKey::with_rng(client.rng());
    let anchor_block = client.get_latest_epoch_block().await.unwrap();
    let builder = AccountBuilder::new(init_seed)
        .anchor((&anchor_block).try_into().unwrap())
        .account_type(AccountType::RegularAccountUpdatableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_component(RpoFalcon512::new(key_pair.public_key()))
        .with_component(BasicWallet);
    let (account, seed) = builder.build().unwrap();
    client
        .add_account(
            &account,
            Some(seed),
            &AuthSecretKey::RpoFalcon512(key_pair),
            false,
        )
        .await?;
    Ok(account)
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // Initialize client
    let mut client = initialize_client().await?;
    println!("Client initialized successfully.");

    // Fetch latest block from node
    let sync_summary = client.sync_state().await.unwrap();
    println!("Latest block: {}", sync_summary.block_num);

    //------------------------------------------------------------
    // STEP 1: Create two basic accounts (Alice and Bob)
    //------------------------------------------------------------
    println!("\n[STEP 1] Creating new accounts");
    let alice_account = create_basic_account(&mut client).await?;
    println!("Alice's account ID: {:?}", alice_account.id().to_hex());
    let bob_account = create_basic_account(&mut client).await?;
    println!("Bob's account ID: {:?}", bob_account.id().to_hex());

    //------------------------------------------------------------
    // STEP 2: Mint and consume tokens for Alice with Ephemeral P2ID
    //------------------------------------------------------------
    println!("\n[STEP 2] Mint tokens with Ephemeral P2ID");

0xf42ab06ffd227c2000005b2767bc5e    let faucet_id = AccountId::from_hex("0xf42ab06ffd227c2000005b2767bc5e").unwrap();
    let amount: u64 = 100;
    let fungible_asset_mint_amount = FungibleAsset::new(faucet_id, amount).unwrap();

    let transaction_request = TransactionRequestBuilder::mint_fungible_asset(
        fungible_asset_mint_amount.clone(),
        alice_account.id(),
        NoteType::Public,
        client.rng(),
    )
    .unwrap()
    .build();

    let tx_execution_result = client
        .new_transaction(faucet_id, transaction_request)
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
        .new_transaction(alice_account.id(), transaction_request)
        .await?;
    client.submit_transaction(tx_execution_result).await?;
    client.sync_state().await?;

    // -------------------------------------------------------------------------
    // STEP 3: Hash Secret Number and Build Note
    // -------------------------------------------------------------------------
    println!("\n[STEP 3] Create note");

    // Hashing secret number combination
    let mut note_secret_number = vec![Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)];
    // Prepend an empty word (4 zero Felts) for the RPO
    note_secret_number.splice(0..0, Word::default().iter().cloned());
    let secret_number_digest = Hasher::hash_elements(&note_secret_number);
    println!("digest: {:?}", secret_number_digest);

    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let file_path = Path::new("../masm/notes/hash_preimage_note.masm");
    let code = fs::read_to_string(file_path).unwrap();
    let rng = client.rng();
    let serial_num = rng.draw_word();
    let note_script = NoteScript::compile(code, assembler).unwrap();

    let inputs: [Felt; 4] = secret_number_digest.into();
    let note_inputs = NoteInputs::new(inputs.into()).unwrap();

    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);
    let tag = NoteTag::for_public_use_case(0, 0, NoteExecutionMode::Local).unwrap();
    let aux = Felt::new(0);
    let metadata = NoteMetadata::new(
        alice_account.id(),
        NoteType::Public,
        tag,
        NoteExecutionHint::always(),
        aux,
    )?;
    let vault = NoteAssets::new(vec![fungible_asset_mint_amount.clone().into()])?;

    let increment_note = Note::new(vault, metadata, recipient);
    println!("note hash: {:?}", increment_note.hash());

    let output_note = OutputNote::Full(increment_note.clone());
    let incr_note_create_request = TransactionRequestBuilder::new()
        .with_own_output_notes([output_note].to_vec())
        .unwrap()
        .build();

    let tx_result = client
        .new_transaction(alice_account.id(), incr_note_create_request)
        .await
        .unwrap();
    let tx_id = tx_result.executed_transaction().id();
    println!(
        "View transaction on MidenScan: https://testnet.midenscan.com/tx/{:?}",
        tx_id
    );
    let _ = client.submit_transaction(tx_result).await;
    client.sync_state().await.unwrap();

    // -------------------------------------------------------------------------
    // STEP 4: Consume Note
    // -------------------------------------------------------------------------
    println!("\n[STEP 4] Bob consumes the Ephemeral Hash Preimage Note with Correct Secret");
    let secret = [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)];

    let tx_note_consume_request = TransactionRequestBuilder::new()
        .with_unauthenticated_input_notes([(increment_note, Some(secret))])
        .build();

    let tx_result = client
        .new_transaction(bob_account.id(), tx_note_consume_request)
        .await
        .unwrap();
    let tx_id = tx_result.executed_transaction().id();
    println!(
        "Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/{:?}",
        tx_id
    );
    let _ = client.submit_transaction(tx_result).await;

    Ok(())
}
```

The output of our program will look something like this:

```
Client initialized successfully.
Latest block: 469311

[STEP 1] Creating new accounts
Alice's account ID: "0x65cb9b6e274edc100007b554a3ab4e"
Bob's account ID: "0x401f2c6a7709e21000077ea1a20838"

[STEP 2] Mint tokens with Ephemeral P2ID
one or more warnings were emitted

[STEP 3] Create note
digest: RpoDigest([14371582251229115050, 1386930022051078873, 17689831064175867466, 9632123050519021080])
note hash: RpoDigest([15305456882644423626, 2295646210412621236, 9366146763779887291, 17539058510438797771])
View transaction on MidenScan: https://testnet.midenscan.com/tx/0xce04521b3f5ef572ea5a0a922979369737685c307294ab923c6f1cff1b8bd0f7

[STEP 4] Bob consumes the Ephemeral Hash Preimage Note with Correct Secret
one or more warnings were emitted
Consumed Note Tx on MidenScan: https://testnet.midenscan.com/tx/0x0dc591cd2accc0b3da06960dc594b2f49d5c91eb48fa0feec3e5b37b62a3ff22
```

## Conclusion

You have now seen how to create a custom note on Miden that requires a secret preimage to be consumed. We covered:

1) Creating and funding accounts (Alice and Bob)
2) Hashing a secret number
3) Building the note with a custom Miden Assembly script
4) Consuming the note by providing the correct secret

By leveraging Miden’s privacy features, you can create customized logic for secure asset transfers that depend on keeping parts of the transaction private.

### Running the example

To run the custom note example, navigate to the `rust-client` directory in the [miden-tutorials](https://github.com/0xPolygonMiden/miden-tutorials/) repository and run this command:

```bash
cd rust-client
cargo run --release --bin hash_preimage_note
```