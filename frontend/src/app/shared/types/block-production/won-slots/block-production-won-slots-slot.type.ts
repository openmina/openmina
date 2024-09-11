export interface BlockProductionWonSlotsSlot {
  // slot related
  epoch: number;
  message: string;
  age: string;
  slotTime: number;
  globalSlot: number;
  slotInEpoch: number;
  vrfValueWithThreshold: [number, number];
  active: boolean;
  percentage: number;

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
  completedWorksCount?: number;

  // time related details
  times?: BlockProductionWonSlotTimes;

  // resulted statuses
  status?: BlockProductionWonSlotsStatus;
  discardReason?: BlockProductionWonSlotsDiscardReason;
  lastObservedConfirmations?: number;
  orphanedBy?: string;
}

export interface BlockProductionWonSlotTimes {
  scheduled: number;
  stagedLedgerDiffCreate: number;
  produced: number;
  proofCreate: number;
  blockApply: number;
  discarded: number;
  committed: number;

  stagedLedgerDiffCreateEnd: number;
  producedEnd: number;
  proofCreateEnd: number;
  blockApplyEnd: number;
}

export type BlockProductionWonSlotsDiscardReason =
  'BestTipStakingLedgerDifferent'
  | 'BestTipGlobalSlotHigher'
  | 'BestTipSuperior';

export enum BlockProductionWonSlotsStatus {
  Scheduled = 'Scheduled',
  StagedLedgerDiffCreatePending = 'StagedLedgerDiffCreatePending',
  StagedLedgerDiffCreateSuccess = 'StagedLedgerDiffCreateSuccess',
  Produced = 'Produced',
  ProofCreatePending = 'ProofCreatePending',
  ProofCreateSuccess = 'ProofCreateSuccess',
  BlockApplyPending = 'BlockApplyPending',
  BlockApplySuccess = 'BlockApplySuccess',
  Committed = 'Committed',
  Discarded = 'Discarded',
  Canonical = 'Canonical',
  Orphaned = 'Orphaned',
}
