# Miden Node Setup Tutorial

To run the Miden tutorial examples, you will need to set up a test enviorment and connect to a Miden node.

There are two ways to connect to a Miden node:

1. Run the Miden node locally
2. Connect to the Miden testnet

## Prerequisites

To run `miden-node` locally, you need to:  

1. Install the `miden-node` crate.  
2. Provide a `genesis.toml` file.  
3. Provide a `miden-node.toml` file.  

Example `genesis.toml` and `miden-node.toml` files can be found in the **miden-tutorials** repository:

- The `genesis.toml` file defines the **start timestamp** for the `miden-node` testnet and allows you to pre-deploy accounts and funding faucets.  
- The `miden-node.toml` file configures the **RPC endpoint** and other settings for the `miden-node`.

## Running the Miden node locally

### Step 1: Clone the miden-tutorials repository

In a terminal window, clone the miden-tutorials repository and navigate to the root of the repository using this command:

```bash
git clone git@github.com:0xPolygonMiden/miden-tutorials.git
cd miden-tutorials
```

### Step 2: Install the Miden node

Next, install the miden-node crate using this command:

```bash
cargo install miden-node --locked
```

### Step 3: Initializing the node

To start the node, we first need to generate the genesis file. To do so, navigate to the `/node` directory and create the genesis file using this command:

```bash
cd node
miden-node make-genesis \
  --inputs-path  config/genesis.toml \
  --output-path  storage/genesis.dat
```

Expected output:

```
Genesis input file: config/genesis.toml has successfully been loaded.
Creating fungible faucet account...
Account "faucet" has successfully been saved to: storage/accounts/faucet.mac
Miden node genesis successful: storage/genesis.dat has been created
```

### Step 4: Starting the node

Now, to start the node, navigate to the storage directory and run this command:

```bash
cd storage
miden-node start \
  --config node/config/miden-node.toml \
  node
```

Expected output:

```
2025-01-17T12:14:55.432445Z  INFO try_build_batches: miden-block-producer: /Users/username/.cargo/registry/src/index.crates.io-6f17d22bba15001f/miden-node-block-producer-0.6.0/src/txqueue/mod.rs:85: close, time.busy: 8.88µs, time.idle: 103µs
2025-01-17T12:14:57.433162Z  INFO try_build_batches: miden-block-producer: /Users/username/.cargo/registry/src/index.crates.io-6f17d22bba15001f/miden-node-block-producer-0.6.0/src/txqueue/mod.rs:85: new
2025-01-17T12:14:57.433256Z  INFO try_build_batches: miden-block-producer: /Users/username/.cargo/registry/src/index.crates.io-6f17d22bba15001f/miden-node-block-producer-0.6.0/src/txqueue/mod.rs:85: close, time.busy: 6.46µs, time.idle: 94.0µs
```

Congratulations, you now have a Miden node running locally. Now we can start creating a testing environment for building applications on Miden!

The endpoint of the Miden node running locally is:

```
http://localhost:57291
```

### Reseting the node

*If you need to reset the local state of the node and the rust-client, navigate to the root of the miden-tutorials repository and run this command:*

```bash 
rm -rf rust-client/store.sqlite3 
rm -rf node/storage/accounts
rm -rf node/storage/blocks
```

## Connecting to the Miden testnet

To run the tutorial examples using the Miden testnet, use this endpoint:

```bash
https://rpc.testnet.miden.io:443
```
