import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

beforeEach(() => {
  for (const key of ['VITE_MIDEN_NETWORK', 'VITE_MIDEN_RPC_URL', 'VITE_MIDEN_PROVER', 'VITE_MIDENSCAN_URL', 'VITE_MIDEN_USDC_FAUCET_ID']) {
    vi.stubEnv(key, undefined);
  }
  vi.resetModules();
});

afterEach(() => { vi.unstubAllEnvs(); vi.resetModules(); });

describe('bridge network configuration', () => {
  it('defaults RPC, prover, and explorer to testnet', async () => {
    const config = await import('../../config');
    const explorers = await import('../explorers');
    expect(config.MIDEN_NETWORK).toBe('testnet');
    expect(config.MIDEN_RPC_URL).toBe('testnet');
    expect(config.MIDEN_PROVER).toBe('testnet');
    expect(config.MIDEN_USDC_FAUCET_ID).toBeUndefined();
    expect(explorers.midenscanNoteUrl('abc')).toBe('https://testnet.midenscan.com/note/abc');
  });

  it('uses explicitly selected devnet and explorer override', async () => {
    vi.stubEnv('VITE_MIDEN_NETWORK', 'devnet');
    vi.stubEnv('VITE_MIDENSCAN_URL', 'https://example.com/');
    vi.stubEnv('VITE_MIDEN_USDC_FAUCET_ID', '0xapproved');
    const config = await import('../../config');
    const explorers = await import('../explorers');
    expect(config.MIDEN_NETWORK).toBe('devnet');
    expect(config.MIDEN_RPC_URL).toBe('devnet');
    expect(config.MIDEN_PROVER).toBe('devnet');
    expect(config.MIDEN_USDC_FAUCET_ID).toBe('0xapproved');
    expect(explorers.midenscanNoteUrl('abc')).toBe('https://example.com/note/abc');
  });
});
