import { useState } from 'react';

interface SendTokensSectionProps {
  onSendTokens: (targetAccountId: string, amount: bigint) => Promise<void>;
}

export function SendTokensSection({ onSendTokens }: SendTokensSectionProps) {
  const [targetAccountId, setTargetAccountId] = useState('');
  const [amount, setAmount] = useState('');

  const handleSubmit = async () => {
    if (!targetAccountId || !amount) return;
    try {
      const amtBigInt = BigInt(amount);
      await onSendTokens(targetAccountId, amtBigInt);
      alert('Tokens sent successfully!');
    } catch (err: any) {
      alert('Error sending tokens: ' + err.message);
    }
    setTargetAccountId('');
    setAmount('');
  };

  return (
    <div style={{ marginBottom: '1rem' }}>
      <input
        type="text"
        placeholder="Target Account ID"
        value={targetAccountId}
        onChange={(e) => setTargetAccountId(e.target.value)}
        style={{ display: 'block', marginBottom: '0.5rem', padding: '0.5rem' }}
        className="uniform-input"
      />
      <input
        type="number"
        placeholder="Amount"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        style={{ display: 'block', marginBottom: '0.5rem', padding: '0.5rem' }}
        className="uniform-input"
      />
      <button
        onClick={handleSubmit}
        style={{ display: 'block', marginBottom: '0.5rem', padding: '0.5rem' }}
        className="button"
      >
        Send Tokens
      </button>
    </div>
  );
}
