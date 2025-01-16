interface NotesSectionProps {
  isGettingNotes: boolean;
  isConsumingNotes: boolean;
  walletId: string | null;
  faucetId: string | null;
  noteId: string | null;
  consumedNoteTxId: string | null;
  disableButtons: boolean;
  handleGetNotes: () => void;
  handleConsumeNotes: () => void;
}

export function NotesSection({
  isGettingNotes,
  isConsumingNotes,
  walletId,
  faucetId,
  noteId,
  consumedNoteTxId,
  disableButtons,
  handleGetNotes,
  handleConsumeNotes,
}: NotesSectionProps) {
  return (
    <div style={{ marginBottom: '1rem' }}>
      {/* Get Notes */}
      <button
        onClick={handleGetNotes}
        disabled={disableButtons || isGettingNotes || !walletId}
        className="button"
      >
        {isGettingNotes ? 'Loading...' : 'Get Notes'}
      </button>

      {/* Consume Notes */}
      <button
        onClick={handleConsumeNotes}
        disabled={
          disableButtons ||
          isConsumingNotes ||
          !noteId ||
          !walletId ||
          !faucetId
        }
        className="button"
      >
        {isConsumingNotes ? 'Consuming...' : 'Consume Notes'}
      </button>

      {!noteId && <p>No notes to consume</p>}
      {consumedNoteTxId && (
        <p>Successfully consumed note. Transaction ID: {consumedNoteTxId}</p>
      )}
    </div>
  );
}
