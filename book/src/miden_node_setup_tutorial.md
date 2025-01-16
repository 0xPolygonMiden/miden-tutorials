# Miden Node Setup Tutorial

To run the Miden tutorial examples, you will need to connect to a running Miden node. 

There are two ways of connecting to a Miden node:
1) Connect to the Miden Testnet 
2) Run the Miden node locally

The quickest way is to connect to the Miden testnet. The other way is to spin up the Miden node locally.

## Miden Testnet Node Endpoint

This is the Miden testnet node endpoint:
```bash
http://18.203.155.106:57291
```
## Running the Miden Node Locally

If you want to run your tests completely locally, you can start the miden node by executing the following commands:

Installing the Miden node: 
```bash
cargo install miden-node --locked --features testing
```

cd into the `miden-tutorials` directory and run the following commands:
```bash
miden-node make-genesis \
  --inputs-path  node/config/genesis.toml \
  --output-path node/storage/genesis.dat

cd node/storage
miden-node start \
  --config node/config/miden-node.toml \
  node
```

*If you need to reset the local state of the node:*
```bash 
rm -rf rust-client/store.sqlite3 
rm -rf node/storage/accounts
rm -rf node/storage/blocks
```

## Specifying the Miden Node Endpoint 

To specify which miden node you are using with the examples in the rust-client, you can specify the miden node endpoint in the `miden-client.toml` file:

```toml
[rpc.endpoint]
protocol = "http"
host = "18.203.155.106" # testnet
# host = "localhost"    # localhost
port = 57291
```
