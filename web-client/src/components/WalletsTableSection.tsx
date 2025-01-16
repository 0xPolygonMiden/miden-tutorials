// src/components/WalletsTableSection.tsx

import React from 'react';
import { WalletDetails } from '../lib/getWalletDetails';

interface WalletsTableProps {
  wallets: WalletDetails[];
}

export function WalletsTableSection({ wallets }: WalletsTableProps) {
  return (
    <div style={{ marginTop: '2rem' }}>
      <h2>All Wallets</h2>
      <table className="wallets-table">
        <thead>
          <tr>
            <th>Account ID</th>
            <th>Nonce</th>
            <th>Balance</th>
          </tr>
        </thead>
        <tbody>
          {wallets.length > 0 ? (
            wallets.map((wallet) => (
              <tr key={wallet.accountId}>
                <td>{wallet.accountId}</td>
                <td>{wallet.nonce}</td>
                <td>{wallet.balance}</td>
              </tr>
            ))
          ) : (
            <tr>
              <td colSpan={3} style={{ textAlign: 'center' }}>
                No wallets created yet.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
