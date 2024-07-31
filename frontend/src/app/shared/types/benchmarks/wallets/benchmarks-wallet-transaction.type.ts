export interface BenchmarksWalletTransaction {
  amount: string;
  fee: string;
  memo: string;
  validUntil: string;
  from: string;
  to: string;
  nonce: string;
  privateKey: string;
  dateTime?: string;
}

/*
{
  "from": "B62qos2NxSras7juEwPVnkoV23YTFvWawyko8pgcf8S5nccTFCzVpdy",
  "nonce": "141",
  "to": "B62qqPz9DfFwDNJP4e2GPpb5aQ7gibaXAwcsLAyQhTb14gExwXCJypD",
  "fee": "1000000000",
  "amount": "1000000000",
  "memo": "1684402542938,1,4906249320019",
  "validUntil": "4294967295",
  "privateKey": "EKELeJwj9QPxGdwvsjAxmKXvYY2pUaEVE8Eufg5e8LoKCzTBNZNh",
}

B62qqPz9DfFwDNJP4e2GPpb5aQ7gibaXAwcsLAyQhTb14gExwXCJypD


 */
