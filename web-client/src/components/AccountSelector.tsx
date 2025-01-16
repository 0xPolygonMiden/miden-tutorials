import React from 'react';
import { WalletDetails } from '../lib/getWalletDetails';

interface AccountSelectorProps {
  wallets: WalletDetails[];
  selectedWalletId: string | null;
  onSelectWallet: (walletId: string) => void;
}

export function AccountSelector({
  wallets,
  selectedWalletId,
  onSelectWallet,
}: AccountSelectorProps) {
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onSelectWallet(e.target.value);
  };

  return (
    <div>
      <h2>Switch Account</h2>
      <label htmlFor="walletSelector">Select Active Wallet: </label>
      <select
        id="walletSelector"
        value={selectedWalletId || ''}
        onChange={handleChange}
      >
        {/* Fallback option if no wallet is selected */}
        <option value="">-- choose wallet --</option>
        {wallets.map((w) => (
          <option key={w.accountId} value={w.accountId}>
            {w.accountId}
          </option>
        ))}
      </select>
    </div>
  );
}
