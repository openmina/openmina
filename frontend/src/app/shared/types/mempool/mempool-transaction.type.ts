export interface MempoolTransaction {
  status: MempoolTransactionStatus;
  date: string;
  timestamp: number;
  kind: MempoolTransactionKind;
  txHash: string;
  sender: string;
  fee: number;
  nonce: number;
  memo: string;
}

export enum MempoolTransactionStatus {
  Applicable = 'Applicable',
  NotApplicable = 'Not Applicable',
}

export enum MempoolTransactionKind {
  ZK_APP = 'ZKApp command',
  PAYMENT = 'Payment',
  DELEGATION = 'Delegation',
}
