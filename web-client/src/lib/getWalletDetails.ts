// src/lib/getWalletDetails.ts
import { AccountId } from '@demox-labs/miden-sdk';
import ClientSingleton from './createClient';

export interface WalletDetails {
  accountId: string; // ID of the account
  nonce: string; // account nonce
  balance: number; // faucet token balance
}

interface GetWalletResult {
  mainWalletId: string | null; // first non-faucet wallet ID
  faucetId: string | null; // first faucet wallet ID
  wallets: WalletDetails[]; // all non-faucet wallet details
}

/**
 * Single function that:
 *   1) Discovers the first faucet account (if any).
 *   2) Discovers the first non-faucet account (mainWalletId).
 *   3) Returns an array of all non-faucet wallets with real or zero balances.
 */
export async function getWalletDetails(): Promise<GetWalletResult> {
  const webClient = await ClientSingleton.getInstance();

  // Fetch all accounts
  const accounts = await webClient.get_accounts();
  console.log('All accounts:', accounts);

  if (!accounts || accounts.length === 0) {
    console.log('No accounts found');
  }

  let mainWalletId: string | null = null;
  let discoveredFaucetId: string | null = null;

  // First, identify the faucet if it exists
  for (const acc of accounts) {
    const accObj = await webClient.get_account(acc.id());
    if (accObj.is_faucet()) {
      discoveredFaucetId = accObj.id().to_string();
      break; // only need the first faucet
    }
  }

  // Now gather non-faucet wallet details
  const wallets: WalletDetails[] = [];
  for (const acc of accounts) {
    const accObj = await webClient.get_account(acc.id());
    const currentIdString = accObj.id().to_string();
    const nonce = accObj.nonce().as_int().toString();
    const isFaucet = accObj.is_faucet();

    // Skip faucet accounts in the table (you only want non-faucet)
    if (isFaucet) {
      continue;
    }

    // The first non-faucet wallet is your 'mainWalletId'
    if (!mainWalletId) {
      mainWalletId = currentIdString;
    }

    // If we discovered a faucet, fetch real balance
    // Otherwise, default to 0
    let balance = 0;
    if (discoveredFaucetId) {
      const faucetAccountId = AccountId.from_hex(discoveredFaucetId);
      const bigBalance = accObj.vault().get_balance(faucetAccountId);
      balance = Number(bigBalance);
    }

    wallets.push({
      accountId: currentIdString,
      nonce,
      balance,
    });
  }

  console.log('Discovered faucet:', discoveredFaucetId);
  console.log('First non-faucet wallet (mainWalletId):', mainWalletId);

  return {
    mainWalletId,
    faucetId: discoveredFaucetId, // might remain null if none found
    wallets,
  };
}
