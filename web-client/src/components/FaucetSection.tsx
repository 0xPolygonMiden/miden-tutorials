interface FaucetSectionProps {
  isSettingUpFaucet: boolean;
  faucetId: string | null;
  disableButtons: boolean;
  handleSetupFaucet: () => void;
}

export function FaucetSection({
  isSettingUpFaucet,
  faucetId,
  disableButtons,
  handleSetupFaucet,
}: FaucetSectionProps) {
  return (
    <div>
      <button
        onClick={handleSetupFaucet}
        disabled={disableButtons || isSettingUpFaucet}
        className="button"
      >
        {isSettingUpFaucet ? 'Loading...' : 'Create Faucet'}
      </button>
      {faucetId && <p>Faucet created with ID: {faucetId}</p>}
    </div>
  );
}
