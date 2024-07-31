export interface BenchmarksWallet {
  publicKey: string;
  privateKey: string;
  minaTokens: number;
  nonce: number;
  lastTxCount: string;
  lastTxTime: string;
  lastTxMemo: string;
  lastTxStatus: string;
  successTx: number;
  failedTx: number;
  errorReason: string;
}

export enum BenchmarksWalletTransactionStatus {
  SENDING = 'sending',
  GENERATED = 'generated',
  ERROR = 'error',
  INCLUDED = 'included',
}
