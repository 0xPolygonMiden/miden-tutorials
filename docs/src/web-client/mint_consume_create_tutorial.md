# Mint, Consume, and Create Notes

*Using the Miden WebClient in TypeScript to mint, consume, and create notes*

## Overview

In the previous section, we initialized our repository and covered how to create an account and deploy a faucet. In this section, we will mint tokens from the faucet for *Alice*, consume the newly created notes, and demonstrate how to send assets to other accounts.

## What we'll cover

- Minting assets from a faucet
- Consuming notes to fund an account
- Sending tokens to other users

## Step 1: Minting tokens from the faucet

To mint notes with tokens from the faucet we created, Alice can use the WebClient's `new_mint_transaction()` function.

Below is an example of a transaction request minting tokens from the faucet for Alice. 

Add this snippet to the end of the `webClient` function in the `src/webClient.ts` file that we created in the previous chapter:

```ts
await client.fetch_and_cache_account_auth_by_pub_key(
  AccountId.from_hex(faucetIdHex),
);
await client.sync_state();

console.log("Minting tokens to Alice...");
await client.new_mint_transaction(
  AccountId.from_hex(aliceIdHex),  // target wallet id
  AccountId.from_hex(faucetIdHex), // faucet id
  NoteType.public(),               // note type
  BigInt(1000),                    // amount
);

console.log("Waiting 15 seconds for transaction confirmation...");
await new Promise((resolve) => setTimeout(resolve, 15000));
await client.sync_state();
```

## Step 2: Identifying consumable notes

Once Alice has minted a note from the faucet, she will eventually want to spend the tokens that she received in the note created by the mint transaction. 

Minting a note from a faucet on Miden means a faucet account creates a new note targeted to the requesting account. The requesting account must consume this note for the assets to appear in their account.

To identify notes that are ready to consume, the Miden WebClient has a useful function `get_consumable_notes`. It is also important to sync the state of the client before calling the `get_consumable_notes` function. 

*Tip: If you know the expected number of notes after a transaction, use `await` or a loop condition to verify their availability before calling `get_consumable_notes`. This prevents unnecessary application idling.*

#### Identifying which notes are available:

```ts
consumable_notes = await client.get_consumable_notes(accountId);
```

## Step 3: Consuming multiple notes in a single transaction:

Now that we know how to identify notes ready to consume, let's consume the notes created by the faucet in a single transaction. After consuming the notes, Alice's wallet balance will be updated.

The following code snippet identifies and consumes notes in a single transaction.

Add this snippet to the end of the `webClient` function in the `src/webClient.ts` file:

```ts
await client.fetch_and_cache_account_auth_by_pub_key(
  AccountId.from_hex(aliceIdHex),
);

const mintedNotes = await client.get_consumable_notes(
  AccountId.from_hex(aliceIdHex),
);
const mintedNoteIds = mintedNotes.map((n) =>
  n.input_note_record().id().to_string(),
);
console.log("Minted note IDs:", mintedNoteIds);

console.log("Consuming minted notes...");
await client.new_consume_transaction(
  AccountId.from_hex(aliceIdHex), // account id
  mintedNoteIds,                  // array of note ids to consume
);
await client.sync_state();
console.log("Notes consumed.");
```

## Step 4: Sending tokens to other accounts

After consuming the notes, Alice has tokens in her wallet. Now, she wants to send tokens to her friends. She has two options: create a separate transaction for each transfer or batch multiple notes in a single transaction.

*The standard asset transfer note on Miden is the P2ID note (Pay to Id). There is also the P2IDR (Pay to Id Reclaimable) variant which allows the creator of the note to reclaim the note after a certain block height.*

In our example, Alice will now send 50 tokens to a different account.

### Basic P2ID transfer

Now as an example, Alice will send some tokens to an account in a single transaction.

Add this snippet to the end of your file in the `main()` function:

```ts
// send single P2ID note
const dummyIdHex = "0x599a54603f0cf9000000ed7a11e379";
console.log("Sending tokens to dummy account...");
await client.new_send_transaction(
  AccountId.from_hex(aliceIdHex),  // sender account id
  AccountId.from_hex(dummyIdHex),  // receiver account id
  AccountId.from_hex(faucetIdHex), // faucet account id
  NoteType.public(),               // note type
  BigInt(100),                     // amount
);
await client.sync_state();
```

## Summary

Your `src/webClient.ts` function should now look like this:

```ts
import {
  WebClient,
  AccountStorageMode,
  AccountId,
  NoteType,
} from "@demox-labs/miden-sdk";

const nodeEndpoint = "http://localhost:57291";

export async function webClient(): Promise<void> {
  try {
    // 1. Create client
    const client = new WebClient();
    await client.create_client(nodeEndpoint);

    // 2. Sync and log block
    const state = await client.sync_state();
    console.log("Latest block number:", state.block_num());

    // 3. Create Alice account (public, updatable)
    console.log("Creating account for Alice");
    const aliceAccount = await client.new_wallet(
      AccountStorageMode.public(), // account type
      true                         // mutability
    );
    const aliceIdHex = aliceAccount.id().to_string();
    console.log("Alice's account ID:", aliceIdHex);

    // 4. Create faucet
    console.log("Creating faucet...");
    const faucetAccount = await client.new_faucet(
      AccountStorageMode.public(),  // account type
      false,                        // fungible
      "MID",                        // symbol
      8,                            // decimals
      BigInt(1_000_000)             // max supply
    );
    const faucetIdHex = faucetAccount.id().to_string();
    console.log("Faucet account ID:", faucetIdHex);

    // 5. Mint tokens to Alice
    await client.fetch_and_cache_account_auth_by_pub_key(
      AccountId.from_hex(faucetIdHex),
    );
    await client.sync_state();

    console.log("Minting tokens to Alice...");
    await client.new_mint_transaction(
      AccountId.from_hex(aliceIdHex),  // target wallet id
      AccountId.from_hex(faucetIdHex), // faucet id
      NoteType.public(),               // note type
      BigInt(1000),                    // amount
    );

    console.log("Waiting 15 seconds for transaction confirmation...");
    await new Promise((resolve) => setTimeout(resolve, 15000));
    await client.sync_state();

    // 6. Fetch minted notes
    await client.fetch_and_cache_account_auth_by_pub_key(
      AccountId.from_hex(aliceIdHex),
    );

    const mintedNotes = await client.get_consumable_notes(
      AccountId.from_hex(aliceIdHex),
    );
    const mintedNoteIds = mintedNotes.map((n) =>
      n.input_note_record().id().to_string(),
    );
    console.log("Minted note IDs:", mintedNoteIds);

    // 7. Consume minted notes
    console.log("Consuming minted notes...");
    await client.new_consume_transaction(
      AccountId.from_hex(aliceIdHex), // account id
      mintedNoteIds,                  // array of note ids to consume
    );
    await client.sync_state();
    console.log("Notes consumed.");

    // 8. Send tokens to a dummy account
    const dummyIdHex = "0x599a54603f0cf9000000ed7a11e379";
    console.log("Sending tokens to dummy account...");
    await client.new_send_transaction(
      AccountId.from_hex(aliceIdHex),  // sender account id
      AccountId.from_hex(dummyIdHex),  // receiver account id
      AccountId.from_hex(faucetIdHex), // faucet account id
      NoteType.public(),               // note type
      BigInt(100),                     // amount
    );
    await client.sync_state();
    console.log("Tokens sent.");
  } catch (error) {
    console.error("Error:", error);
    throw error;
  }
}
```

Let's run the `src/webClient.ts` function again. Reload the page and click "Start WebClient". 

**Note**: *Currently there is a minor bug in the WebClient that produces a warning message, "Error inserting code with root" when creating multiple accounts. This is currently being fixed.*

The output will look like this:

```
Latest block number: 4807
Alice's account ID: 0x1a20f4d1321e681000005020e69b1a
Creating faucet...
Faucet account ID: 0xaa86a6f05ae40b2000000f26054d5d
Minting tokens to Alice...
Waiting 15 seconds for transaction confirmation...
Minted note IDs: ['0x4edbb3d5dbdf6944f229a4711533114e0602ad48b70cda400993925c61f5bfaa']
Consuming minted notes...
Notes consumed.
Sending tokens to dummy account...
Tokens sent.
```

### Resetting the `MidenClientDB`

The Miden webclient stores account and note data in the browser. To clear the account and node data in the browser, paste this code snippet into the browser console:

```javascript
(async () => {
  const dbs = await indexedDB.databases(); // Get all database names
  for (const db of dbs) {
    await indexedDB.deleteDatabase(db.name);
    console.log(`Deleted database: ${db.name}`);
  }
  console.log("All databases deleted.");
})();
```

### Running the example

To run a full working example navigate to the `web-client` directory in the [miden-tutorials](https://github.com/0xPolygonMiden/miden-tutorials/) repository and run the web application example:

```bash
cd web-client
pnpm i
pnpm run dev
```
