import { AccountId, FungibleAsset, Note, NoteAssets, NoteAttachment, NoteType } from '@miden-sdk/miden-sdk';

/** Build the public, reclaimable note required by the current Epoch allocator. */
export function createEpochCollateralNote(params: {
  sender: string;
  allocator: string;
  faucet: string;
  amount: bigint;
  currentBlock: number;
  recallBlocks: number;
  bindingAttachmentFelts: bigint[];
}): Note {
  if (!Number.isSafeInteger(params.currentBlock) || params.currentBlock < 0 ||
      !Number.isSafeInteger(params.recallBlocks) || params.recallBlocks <= 0 ||
      params.currentBlock + params.recallBlocks > 0xffff_ffff) {
    throw new Error('Invalid Epoch reclaim window');
  }
  if (params.amount <= 0n) throw new Error('Epoch collateral amount must be positive');
  if (params.bindingAttachmentFelts.length === 0) {
    throw new Error('Epoch mandate-binding attachment is required');
  }
  if (params.bindingAttachmentFelts.some(value => value < 0n || value >= 18_446_744_069_414_584_321n)) {
    throw new Error('Epoch attachment values must be canonical field elements');
  }
  return Note.createP2IDENote(
    AccountId.fromHex(params.sender),
    AccountId.fromHex(params.allocator),
    new NoteAssets([new FungibleAsset(AccountId.fromHex(params.faucet), params.amount)]),
    params.currentBlock + params.recallBlocks,
    null,
    NoteType.Public,
    new NoteAttachment(BigUint64Array.from(params.bindingAttachmentFelts)),
  );
}
