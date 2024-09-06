export interface WebNodeDemoTransaction {
  from: string;
  to: string;
  amount: string | number;
  fee: string | number;
  memo: string;
  nonce: string;
  status: string;
  statusText: string;
  priv_key: string;
  hash?: string;
  height?: number;
  includedTime?: string;
  time: number;
}
