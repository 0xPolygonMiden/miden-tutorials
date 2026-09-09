'use client';

import {
  MidenProvider,
  useMiden,
  useCreateWallet,
  useCreateFaucet,
  useMint,
  useConsume,
  useMultiSend,
} from '@miden-sdk/react/lazy';
import { NoteVisibility, StorageMode } from '@miden-sdk/miden-sdk/lazy';
import { tutorialNetwork } from '../feeSupport';
import {
  TutorialButton,
  tutorialAuthScheme,
  useTutorialSupport,
} from './tutorialSupport';

function MultiSendInner() {
  const { sync } = useMiden();
  const { createWallet } = useCreateWallet();
  const { createFaucet } = useCreateFaucet();
  const { mint } = useMint();
  const { consume } = useConsume();
  const { sendMany } = useMultiSend();
  const { fundAccount, committed, waitForTokenNotes, assertBalance } =
    useTutorialSupport();

  const run = async () => {
    await sync();
    const authScheme = await tutorialAuthScheme();
    const alice = await createWallet({
      storageMode: StorageMode.Public,
      authScheme,
    });
    console.log('Alice ID:', alice.id().toString());
    await fundAccount(alice);
    const faucet = await createFaucet({
      tokenSymbol: 'MID',
      decimals: 8,
      maxSupply: BigInt(1_000_000),
      storageMode: StorageMode.Public,
      authScheme,
    });
    console.log('Faucet ID:', faucet.id().toString());
    await fundAccount(faucet);

    await sync();
    const minted = await mint({
      faucetId: faucet,
      targetAccountId: alice,
      amount: BigInt(10_000),
      noteType: NoteVisibility.Public,
    });
    await committed(minted.transactionId);
    const notes = await waitForTokenNotes(alice, faucet);
    const consumed = await consume({ accountId: alice.id().toString(), notes });
    await committed(consumed.transactionId);

    const recipients = [];
    for (let index = 0; index < 3; index += 1) {
      recipients.push(
        await createWallet({ storageMode: StorageMode.Public, authScheme }),
      );
    }
    const sent = await sendMany({
      from: alice,
      assetId: faucet,
      recipients: recipients.map((account) => ({
        to: account,
        amount: BigInt(100),
      })),
      noteType: NoteVisibility.Public,
    });
    await committed(sent.transactionId);
    for (const recipient of recipients) {
      const outputs = await waitForTokenNotes(recipient, faucet);
      if (
        outputs.length !== 1 ||
        outputs[0].details().assets().fungibleAssets()[0]?.amount() !==
          BigInt(100)
      ) {
        throw new Error(`Expected one 100 MID note for ${recipient.id()}`);
      }
    }
    await assertBalance(alice, faucet, BigInt(9700));
    console.log('All notes created ✅');
  };

  return (
    <TutorialButton
      name="multiSendWithDelegatedProver"
      label="Run: Multi-Send with Delegated Proving"
      run={run}
    />
  );
}

export default function MultiSendWithDelegatedProver() {
  return (
    <MidenProvider
      config={{
        rpcUrl: tutorialNetwork(),
        prover: tutorialNetwork(),
        autoSyncInterval: 0,
      }}
    >
      <MultiSendInner />
    </MidenProvider>
  );
}
