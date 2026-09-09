use rust_client::TutorialClientExt;
use std::{path::PathBuf, sync::Arc};

use miden_client::{
    account::{AccountId, StorageSlotName},
    builder::ClientBuilder,
    keystore::FilesystemKeyStore,
    rpc::{GrpcClient, VerifyingRpcClient},
    transaction::TransactionRequestBuilder,
    ClientError,
};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use rust_client::TutorialNetwork;

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

    // -------------------------------------------------------------------------
    // STEP 1: Read the Public State of the Counter Contract
    // -------------------------------------------------------------------------
    println!("\n[STEP 1] Reading data from public state");

    // Pass the account ID printed by `counter_contract_deploy` as the first argument, or via
    // `MIDEN_COUNTER_ACCOUNT_ID`.
    let counter_contract_bech32 = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("MIDEN_COUNTER_ACCOUNT_ID").ok())
        .expect("pass the counter account ID from counter_contract_deploy");
    let (account_network, counter_contract_id) =
        AccountId::from_bech32(&counter_contract_bech32).expect("invalid counter account ID");
    assert_eq!(
        account_network,
        network.network_id(),
        "counter account must match the selected tutorial network"
    );

    client
        .import_account_by_id(counter_contract_id)
        .await
        .unwrap();

    let counter_contract = client
        .get_account(counter_contract_id)
        .await
        .unwrap()
        .expect("counter contract not found");
    println!(
        "Account details: {:?}",
        counter_contract.storage().slots().first().unwrap()
    );
    let counter_slot_name =
        StorageSlotName::new("miden::tutorials::counter").expect("valid slot name");
    let count_before = counter_contract
        .storage()
        .get_item(&counter_slot_name)
        .unwrap()[0];

    // -------------------------------------------------------------------------
    // STEP 2: Call the Counter Contract with a script
    // -------------------------------------------------------------------------
    println!("\n[STEP 2] Call the increment_count procedure in the counter contract");

    // Load the MASM sources at compile time so the binary is independent of
    // the working directory it is run from.
    let script_code = include_str!("../../../masm/scripts/counter_script.masm");
    let counter_code = include_str!("../../../masm/accounts/counter.masm");

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
        .submit_tutorial_transaction(counter_contract_id, tx_increment_request)
        .await
        .unwrap();

    println!(
        "View transaction on MidenScan: {}/tx/{:?}",
        network.explorer_url(),
        tx_id
    );

    client.sync_state().await.unwrap();

    // Retrieve updated contract data to see the incremented counter
    let account = client
        .get_account(counter_contract_id)
        .await
        .unwrap()
        .expect("counter contract not found");
    println!(
        "counter contract storage: {:?}",
        account.storage().get_item(&counter_slot_name)
    );
    assert_eq!(
        account.storage().get_item(&counter_slot_name).unwrap()[0],
        count_before + miden_client::ONE,
        "the imported counter must increment exactly once",
    );
    Ok(())
}
