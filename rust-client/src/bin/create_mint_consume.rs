use miden_client::{
    accounts::{AccountId, AccountStorageMode, AccountTemplate, AccountType},
    assets::{FungibleAsset, TokenSymbol},
    crypto::Digest,
    notes::NoteType,
    transactions::{OutputNote, PaymentTransactionData, TransactionRequest},
    ClientError, Felt,
};
use miden_lib::notes::create_p2id_note;
use miden_objects::accounts::get_account_seed;
use rand::Rng;

use rust_client::common::initialize_client;

use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let mut client = initialize_client().await?;

    //------------------------------------------------------------
    // STEP 1: Create a basic wallet account for Alice
    //------------------------------------------------------------
    println!("\n[STEP 1] Creating new account for Alice");

    let alice_template = AccountTemplate::BasicWallet {
        mutable_code: true,
        storage_mode: AccountStorageMode::Public,
    };

    // Create Alice's account
    let (alice_account, _alice_seed) = client.new_account(alice_template).await?;
    println!(
        "Successfully created Alice's wallet. ID: {:?}",
        alice_account.id()
    );

    //------------------------------------------------------------
    // STEP 2: Deploy a fungible faucet (token)
    //------------------------------------------------------------
    println!("\n[STEP 2] Deploying a new fungible faucet.");

    let faucet_template = AccountTemplate::FungibleFaucet {
        token_symbol: TokenSymbol::new("POL").unwrap(),
        decimals: 8,
        max_supply: 1_000_000,
        storage_mode: AccountStorageMode::Public,
    };

    let (faucet_account, _faucet_seed) = client.new_account(faucet_template).await?;
    println!(
        "Successfully created a new faucet. Faucet ID: {:?}",
        faucet_account.id()
    );

    //------------------------------------------------------------
    // STEP 3: Mint 5 notes of 100 tokens each for Alice
    //------------------------------------------------------------
    println!("\n[STEP 3] Minting 5 notes of 100 tokens each for Alice.");

    // Sync state to ensure faucet is picked up by the client
    client.sync_state().await?;
    tokio::time::sleep(Duration::from_secs(5)).await;

    for i in 1..=5 {
        let amount: i32 = 100;
        let fungible_asset = FungibleAsset::new(faucet_account.id(), 100).unwrap();

        let transaction_request = TransactionRequest::mint_fungible_asset(
            fungible_asset.clone(),
            alice_account.id(),
            NoteType::Public,
            client.rng(),
        )
        .unwrap();

        let tx_execution_result = client
            .new_transaction(faucet_account.id(), transaction_request)
            .await?;

        client.submit_transaction(tx_execution_result).await?;
        println!("Minted note #{} of {} tokens for Alice.", i, amount);
    }

    // Sync state to ensure all notes are visible to the client
    client.sync_state().await?;
    println!("All 5 notes minted for Alice successfully!");

    //------------------------------------------------------------
    // STEP 4: Alice consumes all her notes
    //------------------------------------------------------------
    println!("\n[STEP 4] Alice will now consume all of her notes to consolidate them.");

    // Wait until there are exactly 5 consumable notes for Alice
    loop {
        // Re-sync state to ensure we have the latest info
        client.sync_state().await?;

        // Fetch all consumable notes for Alice
        let consumable_notes = client
            .get_consumable_notes(Some(alice_account.id()))
            .await?;
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
                "Currently, Alice has {} consumable notes. Waiting for 5...",
                list_of_note_ids.len()
            );
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
    //------------------------------------------------------------
    // STEP 5: Using Alice's wallet, send 5 notes of 50 tokens each to list of users
    //------------------------------------------------------------
    println!("\n[STEP 5] Alice sends 5 notes of 50 tokens each to 5 different users.");

    let mut p2id_notes = vec![];
    for i in 1..=4 {
        // Generate a unique random seed based on the loop index `i`
        let init_seed = {
            let mut seed = [0u8; 32];
            rand::thread_rng().fill(&mut seed);
            seed[0] = i as u8;
            seed
        };

        // Create a new dummy account ID
        // let target_account_id = AccountId::new_dummy(init_seed, AccountType::RegularAccountUpdatableCode);
        let code_commitment = Digest::default();
        let storage_commitment = Digest::default();
        let seed = get_account_seed(
            init_seed,
            AccountType::RegularAccountUpdatableCode,
            AccountStorageMode::Public,
            code_commitment,
            storage_commitment,
        )
        .unwrap();
        let target_account_id = AccountId::new(seed, code_commitment, storage_commitment).unwrap();

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

    // Example of sending a single P2ID transaction

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
        client.rng(),     // rng
    )
    .unwrap();

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
