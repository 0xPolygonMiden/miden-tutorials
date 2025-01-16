interface ClientSectionProps {
  clientInitialized: boolean;
  disableButtons: boolean;
}

export function ClientSection({ clientInitialized }: ClientSectionProps) {
  return (
    <div style={{ marginBottom: '1rem' }}>
      {clientInitialized && <p>Web Client Initialized.</p>}
    </div>
  );
}
