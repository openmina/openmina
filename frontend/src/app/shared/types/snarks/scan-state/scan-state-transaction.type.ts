export interface ScanStateTransaction {
  hash: string | null;
  kind: 'Payment' | 'Zkapp';
  status: {
    [p: string]: [string][];
  };
}
