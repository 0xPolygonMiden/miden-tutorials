use miden_client::{
    accounts::AccountData,
    transactions::{TransactionKernel, TransactionRequest},
    ClientError, Word,
};
use miden_objects::accounts::AccountComponent;
use miden_objects::accounts::StorageSlot;
use miden_objects::assembly::Assembler;
use miden_objects::crypto::hash::rpo::RpoDigest;

use rust_client::common::{create_new_account, initialize_client};
use std::fs;
use std::path::Path;

use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let mut client = initialize_client().await?;

    //------------------------------------------------------------
    // STEP 1: Create a basic counter contract
    //------------------------------------------------------------
    println!("\n[STEP 1] Creating Counter Contract.");

    // Initializing Account
    let file_path = Path::new("../masm/accounts/counter.masm");

    // Read the file contents
    let account_code = fs::read_to_string(file_path).expect("Failed to read the file");

    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let account_component = AccountComponent::compile(
        account_code,
        assembler,
        vec![StorageSlot::Value(Word::default())],
    )
    .unwrap()
    .with_supports_all_types();

    let (counter_contract, counter_seed, auth_secret_key) = create_new_account(account_component);

    println!("counter_contract hash: {:?}", counter_contract.hash());
    println!("counter_contract accountId: {:?}", counter_contract.id());

    let counter_contract_account_data = AccountData::new(
        counter_contract.clone(),
        counter_seed,
        auth_secret_key.clone(),
    );

    // Import to client
    client
        .import_account(counter_contract_account_data)
        .await
        .unwrap();

    // procedure roots
    let procedures = counter_contract.code().procedure_roots();
    let procedures_vec: Vec<RpoDigest> = procedures.collect();
    for (index, procedure) in procedures_vec.iter().enumerate() {
        println!("Procedure {}: {:?}", index + 1, procedure.to_hex());
    }

    println!("number of procedures: {}", procedures_vec.len());

    //------------------------------------------------------------
    // STEP 2: Call Counter Contract with script
    //------------------------------------------------------------
    println!("\n[STEP 2] Call Counter Contract With Script");

    // --- 1) Grab the procedure #2 hash and prepare it for insertion into the script
    let procedure_2_hash = procedures_vec[0].to_hex();
    let procedure_call = format!("{}", procedure_2_hash);

    // --- 2) Load MASM script
    let file_path = Path::new("../masm/scripts/counter_script.masm");
    let original_code = fs::read_to_string(file_path).expect("Failed to read the file");

    // --- 3) Replace {increment_count} in the script with the actual call line
    let replaced_code = original_code.replace("{increment_count}", &procedure_call);
    println!("Final script:\n{}", replaced_code);

    // --- 4) Compile the script (now containing the procedure #2 hash)
    let tx_script = client.compile_tx_script(vec![], &replaced_code).unwrap();

    // --- 5) Execute the transaction
    let tx_increment_request = TransactionRequest::new()
        .with_custom_script(tx_script)
        .unwrap();

    let tx_result = client
        .new_transaction(counter_contract.id(), tx_increment_request)
        .await
        .unwrap();

    println!("tx result id: {:?}", tx_result.executed_transaction().id());

    let _ = client.submit_transaction(tx_result).await;

    tokio::time::sleep(Duration::from_secs(3)).await;
    client.sync_state().await.unwrap();

    let (account, _data) = client.get_account(counter_contract.id()).await.unwrap();

    println!("storage item 0: {:?}", account.storage().get_item(0));

    Ok(())
}
