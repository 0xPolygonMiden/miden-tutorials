# Running the Miden Node locally

### Setup
1)  **Install Miden Node:**
```
cargo install miden-node --locked --features testing
```

2) **In the root of the miden-tutorials directory, run the following:**
```
miden-node make-genesis \
  --inputs-path  node/config/genesis.toml \
  --output-path node/storage/genesis.dat

cd node/storage
miden-node start \
--config node/config/miden-node.toml \
node
```

### Resetting the Miden node:
```
rm -rf rust-client/store.sqlite3 
rm -rf node/storage/accounts
rm -rf node/storage/blocks
```