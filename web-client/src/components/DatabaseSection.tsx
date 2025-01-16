interface DatabaseSectionProps {
  isClearingDb: boolean;
  disableButtons: boolean;
  handleClearDatabase: () => void;
}

export function DatabaseSection({
  isClearingDb,
  disableButtons,
  handleClearDatabase,
}: DatabaseSectionProps) {
  return (
    <div style={{ marginBottom: '1rem' }}>
      <button
        onClick={handleClearDatabase}
        disabled={disableButtons || isClearingDb}
        className="button"
      >
        {isClearingDb ? 'Clearing...' : 'Clear Database'}
      </button>
    </div>
  );
}
