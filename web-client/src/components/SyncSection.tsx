interface SyncSectionProps {
  isSyncingState: boolean;
  syncResult: boolean | null;
  disableButtons: boolean;
  handleSyncState: () => void;
}

export function SyncSection({
  isSyncingState,
  syncResult,
  disableButtons,
  handleSyncState,
}: SyncSectionProps) {
  return (
    <div style={{ marginBottom: '1rem' }}>
      <button
        onClick={handleSyncState}
        disabled={disableButtons || isSyncingState}
        className="button"
      >
        {isSyncingState ? 'Loading...' : 'Sync State'}
      </button>
      {syncResult && <p>State sync complete</p>}
    </div>
  );
}
