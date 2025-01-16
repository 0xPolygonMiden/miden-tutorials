import { useState } from 'react';

interface MintTokensSectionProps {
  isMintingTokens: boolean;
  noteId: string | null;
  disableButtons: boolean;
  handleMintTokens: (walletId: string, amount: bigint) => void;
}

export function MintTokensSection({
  isMintingTokens,
  noteId,
  disableButtons,
  handleMintTokens,
}: MintTokensSectionProps) {
  const [walletIdInput, setWalletIdInput] = useState('');
  const [amountInput, setAmountInput] = useState('');

  const onMint = () => {
    if (!walletIdInput || !amountInput) return;
    const amountBigInt = BigInt(amountInput);
    handleMintTokens(walletIdInput, amountBigInt);
  };

  return (
    <div style={{ marginBottom: '1rem' }}>
      <h2 style={{ marginBottom: '0.5rem' }}>Mint Tokens</h2>
      <input
        type="text"
        placeholder="Target Wallet ID"
        value={walletIdInput}
        onChange={(e) => setWalletIdInput(e.target.value)}
        style={{ display: 'block', marginBottom: '0.5rem', padding: '0.5rem' }}
        className="uniform-input"
      />
      <input
        type="number"
        placeholder="Amount"
        value={amountInput}
        onChange={(e) => setAmountInput(e.target.value)}
        style={{ display: 'block', marginBottom: '0.5rem', padding: '0.5rem' }}
        className="uniform-input"
      />
      <button
        onClick={onMint}
        disabled={
          disableButtons || isMintingTokens || !walletIdInput || !amountInput
        }
        className="button"
        style={{ display: 'block', marginBottom: '0.5rem', padding: '0.5rem' }}
      >
        {isMintingTokens ? 'Loading...' : 'Mint Tokens'}
      </button>
      {noteId && (
        <p style={{ marginTop: '0.5rem' }}>
          Tokens minted with Note ID: {noteId}
        </p>
      )}
    </div>
  );
}
