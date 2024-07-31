export interface BenchmarksTransaction {
  date: string;
  from: string;
  to: string;
  amount: number;
  fee: number;
  memo: string;
  nonce: number;
  validUntil: string;
  privateKey: string;
}
