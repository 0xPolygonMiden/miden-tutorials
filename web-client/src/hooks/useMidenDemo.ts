import { useEffect, useState } from 'react';
import ClientSingleton from '../lib/createClient';
import { createWallet } from '../lib/createWallet';
import { setupFaucet } from '../lib/createFaucet';
import { mintTokens } from '../lib/mintTokens';
import { sendTokens } from '../lib/sendTokens';
// import { disperseTokens } from '../lib/disperseTokens';
import { syncState } from '../lib/syncState';
import { clearDatabase } from '../lib/clearDB';
import { consumeNotes } from '../lib/consumeNotes';
import { getConsumableNotes } from '../lib/getConsumableNotes';

// Import the new function we created
import { getWalletDetails, WalletDetails } from '../lib/getWalletDetails';

export function useMidenDemo() {
  const [walletId, setWalletId] = useState<string | null>(null);
  const [wallets, setWallets] = useState<WalletDetails[]>([]);
  const [faucetId, setFaucetId] = useState<string | null>(null);

  const [noteId, setNoteId] = useState<string | null>(null);
  const [syncResult, setSyncResult] = useState<boolean | null>(null);
  const [_isLoadingWallets, setIsLoadingWallets] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // Other loading states...
  const [isCreatingClient, setIsCreatingClient] = useState<boolean>(false);
  const [clientInitialized, setClientInitialized] = useState<boolean>(false);
  const [isCreatingWallet, setIsCreatingWallet] = useState<boolean>(false);
  const [isSettingUpFaucet, setIsSettingUpFaucet] = useState<boolean>(false);
  const [isMintingTokens, setIsMintingTokens] = useState<boolean>(false);
  const [isSyncingState, setIsSyncingState] = useState<boolean>(false);
  const [isClearingDb, setIsClearingDb] = useState<boolean>(false);

  // States for note operations
  const [isConsumingNotes, setIsConsumingNotes] = useState<boolean>(false);
  const [consumedNoteTxId, setConsumedNoteTxId] = useState<string | null>(null);
  const [isGettingNotes, setIsGettingNotes] = useState<boolean>(false);

  /* ------------------------------
   * Handlers
   * ------------------------------ */

  const handleGetAllWalletsOnLoad = async () => {
    setError(null);
    setIsLoadingWallets(true);

    setIsCreatingClient(true);
    await ClientSingleton.getInstance();
    setIsCreatingClient(false);
    setClientInitialized(true);

    try {
      const {
        mainWalletId,
        faucetId: discoveredFaucet,
        wallets: nonFaucetWallets,
      } = await getWalletDetails();

      setWalletId(mainWalletId); // first non-faucet
      setFaucetId(discoveredFaucet); // first faucet (if any)
      setWallets(nonFaucetWallets); // all non-faucet wallets
    } catch (err: any) {
      console.error('Error fetching all wallets:', err);
      setError(err.message || 'Error fetching all wallets');
    } finally {
      setIsLoadingWallets(false);
    }
  };

  // On first load, fetch everything
  useEffect(() => {
    handleGetAllWalletsOnLoad();
  }, []);

  const handleCreateWallet = async () => {
    setError(null);
    setIsCreatingWallet(true);
    try {
      const newWalletId = await createWallet();
      setWalletId(newWalletId);

      // Just re-run the fetch so we see the new wallet in the table
      // If a faucet already exists, we'll get real balances immediately
      await handleGetAllWalletsOnLoad();
    } catch (err: any) {
      console.error(err);
      setError(err.message || 'Error creating wallet');
    } finally {
      setIsCreatingWallet(false);
    }
  };

  const handleSelectWallet = (newWalletId: string) => {
    setWalletId(newWalletId);
  };

  const handleSetupFaucet = async () => {
    setError(null);
    setIsSettingUpFaucet(true);
    try {
      // create faucet
      const newFaucetId = await setupFaucet();
      setFaucetId(newFaucetId);
      console.log('Faucet ID:', newFaucetId);

      // Re-run the fetch so that the balances are correct now that we have a faucet
      await handleGetAllWalletsOnLoad();
    } catch (err: any) {
      console.error(err);
      setError(err.message || 'Error setting up faucet');
    } finally {
      setIsSettingUpFaucet(false);
    }
  };

  const handleMintTokens = async (inputWalletId: string, amount: bigint) => {
    setError(null);
    setNoteId(null);
    setIsMintingTokens(true);
    try {
      if (!inputWalletId || !faucetId) {
        throw new Error('Wallet ID and Faucet ID are required to mint tokens');
      }
      await mintTokens(inputWalletId, faucetId, amount);
      await handleGetAllWalletsOnLoad();
    } catch (err: any) {
      console.error(err);
      setError(err.message || 'Error minting tokens');
    } finally {
      setIsMintingTokens(false);
    }
  };

  const handleSendTokens = async (targetAccountId: string, amount: bigint) => {
    setError(null);
    try {
      if (!walletId || !faucetId) {
        throw new Error('Wallet ID and Faucet ID required to send tokens');
      }
      const sendResult = await sendTokens(
        walletId,
        faucetId,
        targetAccountId,
        amount
      );
      /*
      const sendResult = await disperseTokens(
        walletId,
        faucetId,
        targetAccountId,
        amount
      ); 
      */
      console.log('Send transaction result:', sendResult);
    } catch (err: any) {
      console.error(err);
      setError(err.message || 'Error sending tokens');
    }
  };

  const handleGetNotes = async () => {
    setError(null);
    setIsGettingNotes(true);
    try {
      if (!walletId) {
        throw new Error('No wallet to fetch notes from');
      }
      const notes = await getConsumableNotes(walletId);
      if (notes.length === 0) {
        setNoteId(null);
        setError('No notes found for this wallet');
      } else {
        setNoteId(notes[0].noteId);
        console.log('Consumable Notes:', notes);
      }
    } catch (err: any) {
      console.error(err);
      setError(err.message || 'Error fetching consumable notes');
    } finally {
      setIsGettingNotes(false);
    }
  };

  const handleConsumeNotes = async () => {
    setError(null);
    setConsumedNoteTxId(null);

    if (!noteId) {
      setError('No notes to consume');
      return;
    }
    if (!walletId || !faucetId) {
      setError('Wallet ID and Faucet ID are required to consume notes');
      return;
    }

    setIsConsumingNotes(true);
    try {
      const txId = await consumeNotes(walletId, faucetId, noteId);
      setConsumedNoteTxId(txId);
      console.log('Consumed Note TX ID:', txId);
      await getWalletDetails();
      setNoteId(null);
    } catch (err: any) {
      console.error(err);
      setError(err.message || 'Error consuming notes');
    } finally {
      setIsConsumingNotes(false);
    }
  };

  const handleSyncState = async () => {
    setError(null);
    setSyncResult(null);
    setIsSyncingState(true);
    try {
      await new Promise((resolve) => setTimeout(resolve, 300));
      const result = await syncState();
      setSyncResult(result);
      console.log('Sync result:', result);
    } catch (err: any) {
      console.error(err);
      setError(err.message || 'Error syncing state');
    } finally {
      setIsSyncingState(false);
    }
  };

  const handleClearDatabase = async () => {
    setError(null);
    setIsClearingDb(true);
    try {
      await new Promise((resolve) => setTimeout(resolve, 300));
      await clearDatabase();
      console.log('Database cleared');
    } catch (err: any) {
      console.error(err);
      setError(err.message || 'Error clearing database');
    } finally {
      setIsClearingDb(false);
    }
  };

  /* ------------------------------
   * Return states and handlers
   * ------------------------------ */
  return {
    // States
    walletId,
    faucetId,
    noteId,
    syncResult,
    error,
    isCreatingClient,
    clientInitialized,
    isCreatingWallet,
    isSettingUpFaucet,
    isMintingTokens,
    isSyncingState,
    isClearingDb,
    isConsumingNotes,
    consumedNoteTxId,
    isGettingNotes,

    // The array of all wallets we have
    wallets,

    // Handlers
    handleCreateWallet,
    handleSelectWallet,
    handleSetupFaucet,
    handleMintTokens,
    handleSendTokens,
    handleSyncState,
    handleClearDatabase,
    handleGetNotes,
    handleConsumeNotes,
  };
}
