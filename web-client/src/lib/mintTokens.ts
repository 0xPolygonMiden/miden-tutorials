import ClientSingleton from './createClient';
import { AccountId, NoteType } from '@demox-labs/miden-sdk';

export async function mintTokens(
  walletId: string,
  faucetId: string,
  amount: bigint
): Promise<string> {
  try {
    const webClient = await ClientSingleton.getInstance();

    const _walletId = AccountId.from_hex(walletId);
    const _faucetId = AccountId.from_hex(faucetId);

    console.log('wallet and faucet', _walletId, _faucetId);
    console.log('hex:', _walletId.to_string(), _faucetId.to_string());

    await webClient.fetch_and_cache_account_auth_by_pub_key(_faucetId);
    await webClient.sync_state();

    console.log('calling mint');

    const newTxnResult = await webClient.new_mint_transaction(
      _walletId, // target wallet id
      _faucetId, // faucet id
      NoteType.public(), // Note Type
      amount // use the passed amount
    );

    await new Promise((r) => setTimeout(r, 20000));
    await webClient.sync_state();

    return newTxnResult.created_notes().notes().toString();
  } catch (error) {
    console.error('Error minting tokens:', error);
    throw error;
  }
}
