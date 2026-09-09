use rand::Rng;
use std::{path::PathBuf, sync::Arc};

use miden_client::{
    account::{component::BasicWallet, AccountBuilder, AccountType},
    auth::{AuthSecretKey, AuthSingleSig},
    builder::ClientBuilder,
    keystore::{FilesystemKeyStore, Keystore},
    rpc::{GrpcClient, VerifyingRpcClient},
    transaction::{TransactionProver, TransactionRequestBuilder},
    ClientError, RemoteTransactionProver,
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

    // Create Alice's account
    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let key_pair = AuthSecretKey::new_falcon512_poseidon2_with_rng(client.rng());

    let alice_account = AccountBuilder::new(init_seed)
        .account_type(AccountType::Private)
        .with_component(AuthSingleSig::from_public_key(key_pair.public_key()))
        .with_component(BasicWallet)
        .build()
        .unwrap();

    client.add_account(&alice_account, false).await?;
    keystore
        .add_key(&key_pair, alice_account.id())
        .await
        .unwrap();
    fund_account_for_fees(&mut client, alice_account.id(), &fee_config).await?;

    // -------------------------------------------------------------------------
    // Set up the delegated (remote) tx prover
    // -------------------------------------------------------------------------
    // Delegated proving outsources ZK proof generation to a remote service. This is
    // the public prover for the selected network; run your own
    // (https://crates.io/crates/miden-remote-prover) and swap the URL to use it.
    // The upstream constant keeps this URL synchronized with the selected network.
    let remote_tx_prover = RemoteTransactionProver::new(network.remote_prover_url());
    let tx_prover: Arc<dyn TransactionProver> = Arc::new(remote_tx_prover);

    // We use a dummy transaction request to showcase delegated proving.
    // In addition to paying the network fee, this transaction increments Alice's nonce.
    let initial_nonce = client
        .get_account(alice_account.id())
        .await?
        .expect("Alice exists")
        .nonce();
    println!("Alice nonce initial: {:?}", initial_nonce);
    let script_code = "@transaction_script pub proc main push.1 drop end";
    let tx_script = client
        .code_builder()
        .compile_tx_script(script_code)
        .unwrap();

    let transaction_request = TransactionRequestBuilder::new()
        .custom_script(tx_script)
        .build()
        .unwrap();

    // Step 1: Execute the transaction locally
    println!("Executing transaction...");
    client.sync_state().await?;
    let tx_result = client
        .execute_transaction(alice_account.id(), transaction_request)
        .await?;

    // Step 2: Prove the transaction using the delegated (remote) prover
    println!("Proving transaction with the delegated prover...");
    let proven_transaction = client.prove_transaction_with(&tx_result, tx_prover).await?;

    // Step 3: Submit the proven transaction
    println!("Submitting proven transaction...");
    let submission_height = client
        .submit_proven_transaction(proven_transaction, &tx_result)
        .await?;

    // Step 4: Apply the transaction to local store
    client
        .apply_transaction(&tx_result, submission_height)
        .await?;
    rust_client::wait_for_transaction(&mut client, tx_result.id()).await?;

    println!("Transaction submitted successfully using the delegated prover!");

    client.sync_state().await.unwrap();

    let account = client
        .get_account(alice_account.id())
        .await
        .unwrap()
        .expect("alice account not found");

    println!("Alice nonce has increased: {:?}", account.nonce());
    assert_eq!(account.nonce(), initial_nonce + miden_client::Felt::ONE);

    Ok(())
}
