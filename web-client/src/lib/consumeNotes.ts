// src/lib/consumeNotes.ts

import ClientSingleton from './createClient';
import { AccountId } from '@demox-labs/miden-sdk';

export async function consumeNotes(
  targetAccountId: string,
  _faucetId: string,
  noteId: string
): Promise<string> {
  try {
    const webClient = await ClientSingleton.getInstance();

    const _targetAccountId = AccountId.from_hex(targetAccountId);
    // const _faucetId = AccountId.from_hex(faucetId);

    await webClient.sync_state();

    await webClient.fetch_and_cache_account_auth_by_pub_key(_targetAccountId);
    const consumeTransactionResult = await webClient.new_consume_transaction(
      _targetAccountId,
      [noteId]
    );

    await webClient.sync_state();

    return consumeTransactionResult.executed_transaction().id().to_hex();
  } catch (error) {
    console.error('Error setting up faucet:', error);
    throw error;
  }
}
