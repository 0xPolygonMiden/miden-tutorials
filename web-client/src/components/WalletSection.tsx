interface WalletSectionProps {
  isCreatingWallet: boolean;
  walletId: string | null;
  disableButtons: boolean;
  handleCreateWallet: () => void;
}

export function WalletSection({
  isCreatingWallet,
  walletId,
  disableButtons,
  handleCreateWallet,
}: WalletSectionProps) {
  return (
    <div style={{ marginBottom: '1rem' }}>
      <button
        onClick={handleCreateWallet}
        disabled={disableButtons || isCreatingWallet}
        className="button"
      >
        {isCreatingWallet ? 'Loading...' : 'Create New Wallet'}
      </button>
      {walletId && <p>Wallet created with ID: {walletId}</p>}
    </div>
  );
}
