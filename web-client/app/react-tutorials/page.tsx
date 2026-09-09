'use client';

import dynamic from 'next/dynamic';
import { useEffect, useState } from 'react';

const tutorials = {
  createMintConsume: dynamic(
    () => import('../../lib/react/createMintConsume'),
    { ssr: false },
  ),
  multiSendWithDelegatedProver: dynamic(
    () => import('../../lib/react/multiSendWithDelegatedProver'),
    { ssr: false },
  ),
  unauthenticatedNoteTransfer: dynamic(
    () => import('../../lib/react/unauthenticatedNoteTransfer'),
    { ssr: false },
  ),
};

export default function ReactTutorials() {
  const [selected, setSelected] = useState<keyof typeof tutorials | null>(null);
  useEffect(() => {
    const requested = new URLSearchParams(window.location.search).get(
      'tutorial',
    );
    if (requested && requested in tutorials)
      setSelected(requested as keyof typeof tutorials);
  }, []);
  const Tutorial = selected ? tutorials[selected] : null;
  return (
    <main>
      <h1>React SDK tutorials</h1>
      <nav>
        {Object.keys(tutorials).map((name) => (
          <p key={name}>
            <a href={`?tutorial=${name}`}>{name}</a>
          </p>
        ))}
      </nav>
      {Tutorial && <Tutorial />}
    </main>
  );
}
