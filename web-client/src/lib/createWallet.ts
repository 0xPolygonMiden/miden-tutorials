// src/lib/createWallet.ts

import ClientSingleton from './createClient';
import { AccountStorageMode } from '@demox-labs/miden-sdk';

export async function createWallet(): Promise<string> {
  try {
    const webClient = await ClientSingleton.getInstance();

    const accountId = await webClient.new_wallet(
      AccountStorageMode.public(), // storage type
      true // updatable
    );

    await webClient.sync_state();

    return accountId.id().to_string();
  } catch (error) {
    console.error('Error setting up wallet:', error);
    throw error;
  }
}
