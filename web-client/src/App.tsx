import './App.css';

import { useMidenDemo } from './hooks/useMidenDemo';

import { ClientSection } from './components/ClientSection';
import { WalletSection } from './components/WalletSection';
import { AccountSelector } from './components/AccountSelector';
import { FaucetSection } from './components/FaucetSection';
import { MintTokensSection } from './components/MintTokensSection';
import { SendTokensSection } from './components/SendTokensSection';
import { WalletsTableSection } from './components/WalletsTableSection';
import { NotesSection } from './components/NotesSection';
import { SyncSection } from './components/SyncSection';
import { DatabaseSection } from './components/DatabaseSection';

function App() {
  const demo = useMidenDemo();

  const disableButtons =
    demo.isCreatingClient ||
    demo.isCreatingWallet ||
    demo.isSettingUpFaucet ||
    demo.isMintingTokens ||
    demo.isSyncingState ||
    demo.isClearingDb;

  return (
    <div className="container">
      <h1>Miden SDK Demo</h1>
      <div className="content">
        <ClientSection
          clientInitialized={demo.clientInitialized}
          disableButtons={disableButtons}
        />

        {/* Group Faucet and Mint Tokens sections side by side */}
        <div className="faucet-mint-group">
          <FaucetSection
            isSettingUpFaucet={demo.isSettingUpFaucet}
            faucetId={demo.faucetId}
            disableButtons={disableButtons}
            handleSetupFaucet={demo.handleSetupFaucet}
          />
          <MintTokensSection
            isMintingTokens={demo.isMintingTokens}
            noteId={demo.noteId}
            disableButtons={disableButtons}
            handleMintTokens={demo.handleMintTokens}
          />
        </div>

        {/* Group Wallet creation, account selector, and wallets table */}
        <div className="wallet-group">
          <WalletSection
            isCreatingWallet={demo.isCreatingWallet}
            walletId={demo.walletId}
            disableButtons={disableButtons}
            handleCreateWallet={demo.handleCreateWallet}
          />

          <AccountSelector
            wallets={demo.wallets}
            selectedWalletId={demo.walletId}
            onSelectWallet={demo.handleSelectWallet}
          />

          <WalletsTableSection wallets={demo.wallets} />
        </div>

        <SendTokensSection onSendTokens={demo.handleSendTokens} />

        <NotesSection
          isGettingNotes={demo.isGettingNotes}
          isConsumingNotes={demo.isConsumingNotes}
          walletId={demo.walletId}
          faucetId={demo.faucetId}
          noteId={demo.noteId}
          consumedNoteTxId={demo.consumedNoteTxId}
          disableButtons={disableButtons}
          handleGetNotes={demo.handleGetNotes}
          handleConsumeNotes={demo.handleConsumeNotes}
        />

        {/* Group Sync and Clear Database side by side */}
        <div className="sync-db-group">
          <SyncSection
            isSyncingState={demo.isSyncingState}
            syncResult={demo.syncResult}
            disableButtons={disableButtons}
            handleSyncState={demo.handleSyncState}
          />

          <DatabaseSection
            isClearingDb={demo.isClearingDb}
            disableButtons={disableButtons}
            handleClearDatabase={demo.handleClearDatabase}
          />
        </div>
      </div>

      {/* Error Display */}
      {demo.error && <p className="error">{demo.error}</p>}
    </div>
  );
}

export default App;
