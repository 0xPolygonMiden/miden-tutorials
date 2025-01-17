# Miden Node Setup Tutorial

To run the Miden tutorial examples, you will need to connect to a running Miden node.

There are two ways to connect to a Miden node:
1) Run the Miden node locally
2) Connect to the Miden testnet


## Running the Miden Node Locally

### Step 1: Clone the miden-tutorials repository

In a terminal window, clone the miden-tutorials repository and navigate to the root of the repository using this command:
```
git clone git@github.com:0xPolygonMiden/miden-tutorials.git
cd miden-tutorials
```

### Step 2: Install the Miden Node
Next, install the miden-node crate using this command:
```bash
cargo install miden-node --locked --features testing
```

### Step 3: Initializing the Node

To start the node, we first need to generate the genesis file. To do so, navigate to the `/node` directory and create the genesis file using this command:
```bash
cd node
miden-node make-genesis \
  --inputs-path  config/genesis.toml \
  --output-path  storage/genesis.dat
```

### Step 4: Starting the Node
Now, to start the node, navigate to the storage directory and run this command:
```bash
cd storage
miden-node start \
  --config node/config/miden-node.toml \
  node
```

Congratulations, after running the start command, you should have a Miden node running locally. Now you have a fully fledged testing environment for building applications on Miden!

The endpoint of the Miden node running locally is:
```
http://localhost:57291
```

### Reseting the Node
*If you need to reset the local state of the node and the rust-client, navigate to the root of the miden-tutorials repository and run this command:*
```bash 
rm -rf rust-client/store.sqlite3 
rm -rf node/storage/accounts
rm -rf node/storage/blocks
```

## Specifying the Miden Node Endpoint 

To specify which miden node you are using with the examples in the rust-client, you can define the miden node endpoint in the `miden-client.toml` file:

```toml
[rpc.endpoint]
protocol = "http"
host = "localhost"        # localhost
# host = "18.203.155.106" # testnet
port = 57291
```

When using the web-client, you specify the miden-node endpoint when initializing the webclient, but we will cover this in later steps. 

## Connecting to the Miden Testnet
To run the tutorial examples using the Miden testnet, use this endpoint:
```bash
http://18.203.155.106:57291
```

