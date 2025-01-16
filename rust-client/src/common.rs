use figment::{
    providers::{Format, Toml},
    Figment,
};
use miden_client::accounts::AccountStorageMode;
use miden_client::{
    config::RpcConfig,
    crypto::RpoRandomCoin,
    rpc::TonicRpcClient,
    store::{
        sqlite_store::{config::SqliteStoreConfig, SqliteStore},
        StoreAuthenticator,
    },
    transactions::{LocalTransactionProver, ProvingOptions},
    Client, ClientError, Felt,
};
use miden_objects::{
    accounts::{Account, AccountBuilder, AccountComponent, AuthSecretKey},
    crypto::dsa::rpo_falcon512::{PublicKey, SecretKey},
    Word,
};
use serde::Deserialize;
use std::{path::Path, sync::Arc};

use miden_lib::accounts::auth::RpoFalcon512;
use rand::Rng;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

/// Name of your local TOML file containing client config.
/// Adjust this if you store it in another place/name.
const CLIENT_CONFIG_FILE_NAME: &str = "miden-client.toml";

/// Simple container for everything in your TOML file (RPC + store configs, etc.)
#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    /// Describes settings related to the RPC endpoint
    pub rpc: RpcConfig,
    /// Describes settings related to the store.
    pub store: SqliteStoreConfig,
}

impl ClientConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let figment = Figment::from(Toml::file(path));
        figment.extract().unwrap_or_else(|e| {
            panic!("Failed to load client config: {}", e);
        })
    }
}

/// This function initializes the `Client` using the parameters
/// from `miden-client.toml`. It loads the store, seeds the RNG,
/// sets up the authenticator, local prover, and returns a `Client`.
pub async fn initialize_client() -> Result<Client<RpoRandomCoin>, ClientError> {
    let client_config = ClientConfig::from_file(CLIENT_CONFIG_FILE_NAME);

    let store = SqliteStore::new(&client_config.store)
        .await
        .map_err(ClientError::StoreError)?;
    let arc_store = Arc::new(store);

    let mut seed_rng = rand::thread_rng();
    let coin_seed: [u64; 4] = seed_rng.gen();
    let rng_for_auth = RpoRandomCoin::new(coin_seed.map(Felt::new));
    let rng_for_client = RpoRandomCoin::new(coin_seed.map(Felt::new));

    let authenticator = StoreAuthenticator::new_with_rng(arc_store.clone(), rng_for_auth);

    let tx_prover = LocalTransactionProver::new(ProvingOptions::default());

    let rpc_client = Box::new(TonicRpcClient::new(&client_config.rpc));

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

pub fn get_new_pk_and_authenticator() -> (Word, AuthSecretKey) {
    let seed = [0_u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    let sec_key = SecretKey::with_rng(&mut rng);
    let pub_key: Word = sec_key.public_key().into();

    let auth_secret_key = AuthSecretKey::RpoFalcon512(sec_key);

    (pub_key, auth_secret_key)
}

pub fn create_new_account(
    account_component: AccountComponent,
) -> (Account, Option<Word>, AuthSecretKey) {
    let (pub_key, auth_secret_key) = get_new_pk_and_authenticator();

    let (account, seed) = AccountBuilder::new()
        .init_seed(ChaCha20Rng::from_entropy().gen())
        .storage_mode(AccountStorageMode::Public)
        .with_component(account_component)
        .with_component(RpoFalcon512::new(PublicKey::new(pub_key)))
        .build()
        .unwrap();
    // .build_testing()
    // .unwrap();

    (account, Some(seed), auth_secret_key)
}
