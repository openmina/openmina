import { ScanStateLeaf } from '@shared/types/snarks/scan-state/scan-state-leaf.type';

export interface ScanStateTree {
  leafs: ScanStateLeaf[];
  availableJobs: number;
  ongoing: number;
  notIncludedSnarks: number;
  completedSnarks: number;
  empty: number;
  coinbase: number;
  payment: number;
  zkApp: number;
  feeTransfer: number;
  merge: number;
}
