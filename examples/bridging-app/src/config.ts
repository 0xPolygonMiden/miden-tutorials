// Application display name (used by wallet adapter).
export const APP_NAME = "Miden x Epoch Bridge";

// Miden SDK configuration — override via environment variables.
export const MIDEN_NETWORK = import.meta.env.VITE_MIDEN_NETWORK || "testnet";
if (!["devnet", "testnet", "local"].includes(MIDEN_NETWORK)) {
  throw new Error("VITE_MIDEN_NETWORK must be devnet, testnet, or local");
}
export const MIDEN_RPC_URL =
  import.meta.env.VITE_MIDEN_RPC_URL || MIDEN_NETWORK;
export const MIDEN_PROVER =
  (import.meta.env.VITE_MIDEN_PROVER || MIDEN_NETWORK) as "devnet" | "testnet" | "local";

// Faucet IDs change across networks and resets. Configure the allocator-approved
// asset for the selected network.
export const MIDEN_USDC_FAUCET_ID = import.meta.env.VITE_MIDEN_USDC_FAUCET_ID?.trim();
