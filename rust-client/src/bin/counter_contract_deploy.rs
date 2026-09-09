use rand::Rng;
use rust_client::TutorialClientExt;
use std::{path::PathBuf, sync::Arc};

use miden_client::{
    account::{
        component::{AccountComponentMetadata, BasicWallet},
        AccountBuilder, AccountComponent, AccountType, StorageSlot, StorageSlotName,
    },
    auth::NoAuth,
    builder::ClientBuilder,
    keystore::FilesystemKeyStore,
    rpc::{GrpcClient, VerifyingRpcClient},
    transaction::TransactionRequestBuilder,
    ClientError, Word,
};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use rust_client::{fund_account_for_fees, FeeConfig, TutorialNetwork};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // Initialize client
    let network = TutorialNetwork::from_env()?;
    let endpoint = network.endpoint();
    let timeout_ms = 10_000;
    let rpc_client = Arc::new(VerifyingRpcClient::new(GrpcClient::new(
        &endpoint, timeout_ms,
    )));

    // Initialize keystore
    let keystore_path = PathBuf::from("./keystore");
    let keystore = Arc::new(FilesystemKeyStore::new(keystore_path).unwrap());

    let store_path = PathBuf::from("./store.sqlite3");

    let mut client = ClientBuilder::new()
        .rpc(rpc_client)
        .sqlite_store(store_path)
        .authenticator(keystore.clone())
        .build()
        .await?;

    let sync_summary = client.sync_state().await.unwrap();
    println!("Latest block: {}", sync_summary.block_num);
    let fee_config = FeeConfig::from_client(&client, network).await?;

    // -------------------------------------------------------------------------
    // STEP 1: Create a basic counter contract
    // -------------------------------------------------------------------------
    println!("\n[STEP 1] Creating counter contract.");

    // Load the MASM file for the counter contract. `include_str!` resolves at
    // compile time relative to this source file, so the binary is independent
    // of the working directory it is run from.
    let counter_code = include_str!("../../../masm/accounts/counter.masm");

    // Compile the account code into `AccountComponent` with one storage slot.
    let counter_slot_name =
        StorageSlotName::new("miden::tutorials::counter").expect("valid slot name");
    let component_code = client
        .code_builder()
        .compile_component_code("external_contract::counter_contract", counter_code)
        .unwrap();
    let counter_component = AccountComponent::new(
        component_code,
        vec![StorageSlot::with_value(
            counter_slot_name.clone(),
            Word::default(),
        )],
        AccountComponentMetadata::new("external_contract::counter_contract"),
    )
    .unwrap();

    // Init seed for the counter contract
    let mut seed = [0_u8; 32];
    client.rng().fill_bytes(&mut seed);

    // Build the new `Account` with the component
    let counter_contract = AccountBuilder::new(seed)
        .account_type(AccountType::Public)
        .with_component(counter_component.clone())
        .with_component(BasicWallet)
        .with_component(NoAuth)
        .build()
        .unwrap();

    println!(
        "counter_contract commitment: {:?}",
        counter_contract.to_commitment()
    );
    println!("counter_contract id: {:?}", counter_contract.id());
    println!("counter_contract storage: {:?}", counter_contract.storage());

    client.add_account(&counter_contract, false).await.unwrap();
    fund_account_for_fees(&mut client, counter_contract.id(), &fee_config).await?;

    // -------------------------------------------------------------------------
    // STEP 2: Call the Counter Contract with a script
    // -------------------------------------------------------------------------
    println!("\n[STEP 2] Call Counter Contract With Script");

    // Load the MASM script referencing the increment procedure
    let script_code = include_str!("../../../masm/scripts/counter_script.masm");

    // Compile the script with the counter contract code linked as a module
    // on the same `CodeBuilder` chain.
    let tx_script = client
        .code_builder()
        .with_linked_module("external_contract::counter_contract", counter_code)
        .unwrap()
        .compile_tx_script(script_code)
        .unwrap();

    // Build a transaction request with the custom script
    let tx_increment_request = TransactionRequestBuilder::new()
        .custom_script(tx_script)
        .build()
        .unwrap();

    // Execute and submit the transaction
    let tx_id = client
        .submit_tutorial_transaction(counter_contract.id(), tx_increment_request)
        .await
        .unwrap();

    println!(
        "View transaction on MidenScan: {}/tx/{:?}",
        network.explorer_url(),
        tx_id
    );

    println!(
        "Counter contract id: {:?}",
        counter_contract.id().to_bech32(network.network_id())
    );

    client.sync_state().await.unwrap();

    // Retrieve updated contract data to see the incremented counter
    let account = client
        .get_account(counter_contract.id())
        .await
        .unwrap()
        .expect("counter contract not found");
    println!(
        "counter contract storage: {:?}",
        account.storage().get_item(&counter_slot_name)
    );
    assert_eq!(
        account.storage().get_item(&counter_slot_name).unwrap()[0].as_canonical_u64(),
        1,
        "the deployed counter must increment from zero to one",
    );

    Ok(())
}
