use rand::Rng;
use rust_client::TutorialClientExt;
use std::{path::PathBuf, sync::Arc};
use tokio::time::Duration;

use miden_client::{
    account::{
        component::{
            create_singlesig_user_fungible_faucet, BasicWallet, BurnPolicy, FungibleFaucet,
            MintPolicy, TokenName, TokenPolicyManager,
        },
        AccountBuilder, AccountId, AccountType,
    },
    asset::{AssetAmount, AssetCallbackFlag, AssetId, FungibleAsset, TokenSymbol},
    auth::{AuthSecretKey, AuthSingleSig},
    builder::ClientBuilder,
    keystore::{FilesystemKeyStore, Keystore},
    note::{Note, NoteType, P2idNote},
    rpc::{GrpcClient, VerifyingRpcClient},
    transaction::{PaymentNoteDescription, TransactionRequestBuilder},
    ClientError,
};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_protocol::account::AccountIdVersion;
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

    //------------------------------------------------------------
    // STEP 1: Create a basic wallet for Alice
    //------------------------------------------------------------
    println!("\n[STEP 1] Creating a new account for Alice");

    // Account seed
    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let key_pair = AuthSecretKey::new_falcon512_poseidon2_with_rng(client.rng());

    // Build the account
    let alice_account = AccountBuilder::new(init_seed)
        .account_type(AccountType::Public)
        .with_component(AuthSingleSig::from_public_key(key_pair.public_key()))
        .with_component(BasicWallet)
        .build()
        .unwrap();

    // Add the account to the client
    client.add_account(&alice_account, false).await?;

    // Add the key pair to the keystore
    keystore
        .add_key(&key_pair, alice_account.id())
        .await
        .unwrap();

    let alice_account_id_bech32 = alice_account.id().to_bech32(network.network_id());
    println!("Alice's account ID: {:?}", alice_account_id_bech32);

    fund_account_for_fees(&mut client, alice_account.id(), &fee_config).await?;

    //------------------------------------------------------------
    // STEP 2: Deploy a fungible faucet
    //------------------------------------------------------------
    println!("\n[STEP 2] Deploying a new fungible faucet.");

    // Faucet seed
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    // Faucet parameters
    let symbol = TokenSymbol::new("MID").unwrap();
    let decimals = 8;
    let max_supply = AssetAmount::new(1_000_000).unwrap();

    // Generate key pair
    let key_pair = AuthSecretKey::new_falcon512_poseidon2_with_rng(client.rng());

    // Build the faucet account.
    // The faucet is a `FungibleFaucet` component plus a `TokenPolicyManager`
    // that registers an "allow all" mint (and burn) policy; minting is rejected
    // unless an active mint policy is present.
    let faucet = FungibleFaucet::builder()
        .name(TokenName::new("MID").unwrap())
        .symbol(symbol)
        .decimals(decimals)
        .max_supply(max_supply)
        .build()
        .unwrap();
    let policies = TokenPolicyManager::builder()
        .active_mint_policy(MintPolicy::allow_all())
        .active_burn_policy(BurnPolicy::allow_all())
        .build();
    // The SDK factory includes BasicWallet so the faucet can receive the native fee asset.
    let faucet_account = create_singlesig_user_fungible_faucet(
        init_seed,
        faucet,
        AuthSingleSig::from_public_key(key_pair.public_key()),
        policies,
        AccountType::Public,
    )
    .unwrap();

    // Add the faucet to the client
    client.add_account(&faucet_account, false).await?;

    // Add the key pair to the keystore
    keystore
        .add_key(&key_pair, faucet_account.id())
        .await
        .unwrap();

    let faucet_account_id_bech32 = faucet_account.id().to_bech32(network.network_id());
    println!("Faucet account ID: {:?}", faucet_account_id_bech32);

    fund_account_for_fees(&mut client, faucet_account.id(), &fee_config).await?;

    // Resync to show newly deployed faucet
    client.sync_state().await?;
    tokio::time::sleep(Duration::from_secs(2)).await;

    //------------------------------------------------------------
    // STEP 3: Mint 5 notes of 100 tokens for Alice
    //------------------------------------------------------------
    println!("\n[STEP 3] Minting 5 notes of 100 tokens each for Alice.");

    let amount: u64 = 100;
    let fungible_asset = FungibleAsset::new(faucet_account.id(), amount).unwrap();

    let mut minted_note_ids = Vec::new();
    for i in 1..=5 {
        let transaction_request = TransactionRequestBuilder::new()
            .build_mint_fungible_asset(
                fungible_asset,
                alice_account.id(),
                NoteType::Public,
                client.rng(),
            )
            .unwrap();

        minted_note_ids.extend(
            transaction_request
                .expected_output_own_notes()
                .iter()
                .map(Note::id),
        );
        println!("tx request built");

        let tx_id = client
            .submit_tutorial_transaction(faucet_account.id(), transaction_request)
            .await?;
        println!(
            "Minted note #{} of {} tokens for Alice. TX: {:?}",
            i, amount, tx_id
        );
    }
    println!("All 5 notes minted for Alice successfully!");

    // Re-sync so minted notes become visible
    client.sync_state().await?;

    //------------------------------------------------------------
    // STEP 4: Alice consumes all her notes
    //------------------------------------------------------------
    println!("\n[STEP 4] Alice will now consume all of her notes to consolidate them.");

    // TX_FEE notes are also consumable. Select only the five P2ID notes we minted.
    let notes = rust_client::wait_for_notes_by_id(&mut client, &minted_note_ids).await?;
    assert_eq!(notes.len(), 5);
    let transaction_request = TransactionRequestBuilder::new().build_consume_notes(notes)?;
    let tx_id = client
        .submit_tutorial_transaction(alice_account.id(), transaction_request)
        .await?;
    println!(
        "All of Alice's notes consumed successfully. TX: {:?}",
        tx_id
    );

    //------------------------------------------------------------
    // STEP 5: Alice sends 5 notes of 50 tokens to 5 users
    //------------------------------------------------------------
    println!("\n[STEP 5] Alice sends 5 notes of 50 tokens each to 5 different users.");

    // Send 50 tokens to 4 accounts in one transaction
    println!("Creating multiple P2ID notes for 4 target accounts in one transaction...");
    let mut p2id_notes = vec![];

    // Creating 4 P2ID notes to 4 'dummy' AccountIds
    for _ in 1..=4 {
        let init_seed: [u8; 15] = {
            let mut init_seed = [0_u8; 15];
            client.rng().fill_bytes(&mut init_seed);
            init_seed
        };
        let target_account_id = AccountId::dummy(
            init_seed,
            AccountIdVersion::Version1,
            AccountType::Public,
            AssetCallbackFlag::Disabled,
        );

        let send_amount = 50;
        let fungible_asset = FungibleAsset::new(faucet_account.id(), send_amount).unwrap();

        let p2id_note: Note = P2idNote::builder()
            .sender(alice_account.id())
            .target(target_account_id)
            .asset(fungible_asset)
            .note_type(NoteType::Public)
            .generate_serial_number(client.rng())
            .build()?
            .into();
        p2id_notes.push(p2id_note);
    }

    // Specifying output notes and creating a tx request to create them
    let output_notes = p2id_notes;
    let transaction_request = TransactionRequestBuilder::new()
        .own_output_notes(output_notes)
        .build()
        .unwrap();

    let tx_id = client
        .submit_tutorial_transaction(alice_account.id(), transaction_request)
        .await?;

    println!("Submitted a transaction with 4 P2ID notes. TX: {:?}", tx_id);

    println!("Submitting one more single P2ID transaction...");
    let init_seed: [u8; 15] = {
        let mut init_seed = [0_u8; 15];
        client.rng().fill_bytes(&mut init_seed);
        init_seed
    };
    let target_account_id = AccountId::dummy(
        init_seed,
        AccountIdVersion::Version1,
        AccountType::Public,
        AssetCallbackFlag::Disabled,
    );

    let send_amount = 50;
    let fungible_asset = FungibleAsset::new(faucet_account.id(), send_amount).unwrap();

    let payment = PaymentNoteDescription::new(
        vec![fungible_asset.into()],
        alice_account.id(),
        target_account_id,
    );
    let transaction_request = TransactionRequestBuilder::new().build_pay_to_id(
        payment,
        NoteType::Public,
        client.rng(),
    )?;

    let tx_id = client
        .submit_tutorial_transaction(alice_account.id(), transaction_request)
        .await?;

    println!("Submitted final P2ID transaction. TX: {:?}", tx_id);
    let alice = client
        .get_account(alice_account.id())
        .await?
        .expect("Alice exists");
    let balance = alice
        .vault()
        .get_balance(AssetId::new_fungible(faucet_account.id()))?;
    assert_eq!(balance.as_u64(), 250, "Alice should retain 500 - 250 MID");

    println!("\nAll steps completed successfully!");
    println!("Alice created a wallet, a faucet was deployed,");
    println!("5 notes of 100 tokens were minted to Alice, those notes were consumed,");
    println!("and then Alice sent 5 separate 50-token notes to 5 different users.");

    Ok(())
}
