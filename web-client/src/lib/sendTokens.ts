import ClientSingleton from './createClient';
import { AccountId, NoteType } from '@demox-labs/miden-sdk';

export async function sendTokens(
  senderWalletId: string,
  faucetId: string,
  targetAccountId: string,
  amount: bigint
): Promise<string> {
  try {
    const webClient = await ClientSingleton.getInstance();

    const senderAccountId = AccountId.from_hex(senderWalletId);
    const faucetAccountId = AccountId.from_hex(faucetId);
    const targetAccId = AccountId.from_hex(targetAccountId);

    // Prepare the sender's account for sending tokens
    await webClient.fetch_and_cache_account_auth_by_pub_key(senderAccountId);

    // Send tokens using the SDK
    const sendResult = await webClient.new_send_transaction(
      senderAccountId,
      targetAccId,
      faucetAccountId,
      NoteType.private(),
      amount
    );

    // Return a representation of the transaction result
    return sendResult.block_num().toString();
  } catch (error) {
    console.error('Error sending tokens:', error);
    throw error;
  }
}
