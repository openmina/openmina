export interface BlockProductionWonSlotsSlot {
  // slot related
  message: string;
  age: string;
  slotTime: number;
  globalSlot: number;
  vrfValueWithThreshold: [number, number];
  active: boolean;

  // block related
  height?: number;
  hash?: string;
  transactionsTotal?: number;
  payments?: number;
  delegations?: number;
  zkapps?: number;
  snarkFees?: number;
  coinbaseRewards?: number;
  txFeesRewards?: number;

  // time related details
  times?: BlockProductionWonSlotTimes;
  // creatingStagedLedgerDiffElapsedTime: number;
  // creatingBlockProofElapsedTime: number;
  // applyingBlockElapsedTime: number;
  // broadcastedBlockElapsedTime: number;

  status?: string;
  discardReason: BlockProductionWonSlotsDiscardReason;
}

export interface BlockProductionWonSlotTimes {
  scheduled: number;
  stagedLedgerDiffCreate: number;
  produced: number;
  proofCreate: number;
  blockApply: number;
  discarded: number;
}

export enum BlockProductionWonSlotsDiscardReason {
  BestTipStakingLedgerDifferent = 'BestTipStakingLedgerDifferent',
  BestTipGlobalSlotHigher = 'BestTipGlobalSlotHigher',
  BestTipSuperior = 'BestTipSuperior',
}
