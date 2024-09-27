export interface BenchmarksZkapp {
  payerPublicKey: string;
  payerPrivateKey: string;
  fee: number;
  nonce: string;
  memo?: string;
  accountUpdates: number;
}
