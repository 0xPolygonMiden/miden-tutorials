use std::{fs, path::Path, sync::Arc};

use rand::Rng;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

use miden_client::{
    account::{
        component::{BasicFungibleFaucet, BasicWallet, RpoFalcon512},
        AccountBuilder, AccountStorageMode, AccountType,
    },
    asset::{FungibleAsset, TokenSymbol},
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

// Helper to deploy a fungible faucet account
async fn deploy_faucet(
    client: &mut Client<RpoRandomCoin>,
    symbol: TokenSymbol,
    decimals: u8,
    max_supply: Felt,
) -> Result<miden_client::account::Account, ClientError> {
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);
    let anchor_block = client.get_latest_epoch_block().await.unwrap();
    let key_pair = SecretKey::with_rng(client.rng());
    let builder = AccountBuilder::new(init_seed)
        .anchor((&anchor_block).try_into().unwrap())
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(AccountStorageMode::Public)
        .with_component(RpoFalcon512::new(key_pair.public_key()))
        .with_component(BasicFungibleFaucet::new(symbol, decimals, max_supply).unwrap());
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
    // STEP 2: Deploy a fungible faucet
    //------------------------------------------------------------
    println!("\n[STEP 2] Deploying a new fungible faucet.");
    let symbol = TokenSymbol::new("MID").unwrap();
    let decimals = 8;
    let max_supply = Felt::new(1_000_000);
    let faucet_account = deploy_faucet(&mut client, symbol, decimals, max_supply).await?;
    println!("Faucet account ID: {:?}", faucet_account.id().to_hex());

    client.sync_state().await?;

    //------------------------------------------------------------
    // STEP 3: Mint and consume tokens for Alice with Ephemeral P2ID
    //------------------------------------------------------------
    println!("\n[STEP 3] Mint tokens with Ephemeral P2ID");
    let amount: u64 = 100;
    let fungible_asset_mint_amount = FungibleAsset::new(faucet_account.id(), amount).unwrap();

    let transaction_request = TransactionRequestBuilder::mint_fungible_asset(
        fungible_asset_mint_amount.clone(),
        alice_account.id(),
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
        .new_transaction(alice_account.id(), transaction_request)
        .await?;
    client.submit_transaction(tx_execution_result).await?;
    client.sync_state().await?;

    // -------------------------------------------------------------------------
    // STEP 4: Hash Secret Number and Build Note
    // -------------------------------------------------------------------------
    println!("\n[STEP 4] Create note");

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
    // STEP 5: Consume Note
    // -------------------------------------------------------------------------
    println!("\n[STEP 5] Bob consumes the Ephemeral Hash Preimage Note with Correct Secret");
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
