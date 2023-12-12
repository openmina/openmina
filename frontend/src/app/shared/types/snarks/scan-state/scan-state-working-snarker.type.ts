import { ScanStateLeaf } from '@shared/types/snarks/scan-state/scan-state-leaf.type';

export interface ScanStateWorkingSnarker {
  hash: string;
  name: string;
  local: boolean;
  url: string;
  leafs: ScanStateLeaf[];
  error?: string;
}
