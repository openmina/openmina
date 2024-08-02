export interface MempoolTransaction {
  kind: MempoolTransactionKind;
  txHash: string;
  sender: string;
  fee: number;
  amount: number;
  nonce: number;
  memo: string;
  transactionData: SignedCommand | ZkappCommand;
  sentFromStressingTool: boolean;
  sentByMyBrowser: boolean;
}

export enum MempoolTransactionKind {
  ZK_APP = 'ZKApp command',
  PAYMENT = 'Payment',
  DELEGATION = 'Delegation',
}


export interface SignedCommand {
  payload: Payload;
  signer: string;
  signature: string;
}

interface Payload {
  common: Common;
  body: Body;
}

interface Common {
  fee: string;
  fee_payer_pk: string;
  nonce: string;
  valid_until: ValidUntil;
  memo: string;
}

interface ValidUntil {
  SinceGenesis: string;
}

interface Body {
  Payment: Payment;
}

interface Payment {
  receiver_pk: string;
  amount: string;
}

export interface ZkappCommand {
  fee_payer: FeePayer;
  account_updates: AccountUpdate[];
  memo: string;
}

interface FeePayer {
  body: Body2;
  authorization: string;
}

interface Body2 {
  public_key: string;
  fee: string;
  valid_until: any;
  nonce: string;
}

interface AccountUpdate {
  elt: Elt;
  stack_hash: any;
}

interface Elt {
  account_update: AccountUpdate2;
  account_update_digest: any;
  calls: Call[];
}

interface AccountUpdate2 {
  body: Body3;
  authorization: Authorization;
}

interface Body3 {
  public_key: string;
  token_id: string;
  update: Update;
  balance_change: BalanceChange;
  increment_nonce: boolean;
  events: any[];
  actions: any[];
  call_data: string;
  preconditions: Preconditions;
  use_full_commitment: boolean;
  implicit_account_creation_fee: boolean;
  may_use_token: string;
  authorization_kind: string;
}

interface Update {
  app_state: string[];
  delegate: string;
  verification_key: any;
  permissions: string;
  zkapp_uri: string;
  token_symbol: string;
  timing: string;
  voting_for: string;
}

interface BalanceChange {
  magnitude: string;
  sgn: string[];
}

interface Preconditions {
  network: Network;
  account: Account;
  valid_while: string;
}

interface Network {
  snarked_ledger_hash: string;
  blockchain_length: string;
  min_window_density: string;
  total_currency: string;
  global_slot_since_genesis: string;
  staking_epoch_data: StakingEpochData;
  next_epoch_data: NextEpochData;
}

interface StakingEpochData {
  ledger: Ledger;
  seed: string;
  start_checkpoint: string;
  lock_checkpoint: string;
  epoch_length: string;
}

interface Ledger {
  hash: string;
  total_currency: string;
}

interface NextEpochData {
  ledger: Ledger2;
  seed: string;
  start_checkpoint: string;
  lock_checkpoint: string;
  epoch_length: string;
}

interface Ledger2 {
  hash: string;
  total_currency: string;
}

interface Account {
  balance: string;
  nonce: any;
  receipt_chain_hash: string;
  delegate: string;
  state: string[];
  action_state: string;
  proved_state: string;
  is_new: string;
}

interface Authorization {
  Signature: string;
}

interface Call {
  elt: Elt2;
  stack_hash: any;
}

interface Elt2 {
  account_update: AccountUpdate3;
  account_update_digest: any;
  calls: any[];
}

interface AccountUpdate3 {
  body: Body4;
  authorization: string;
}

interface Body4 {
  public_key: string;
  token_id: string;
  update: Update2;
  balance_change: BalanceChange2;
  increment_nonce: boolean;
  events: any[];
  actions: any[];
  call_data: string;
  preconditions: Preconditions2;
  use_full_commitment: boolean;
  implicit_account_creation_fee: boolean;
  may_use_token: string;
  authorization_kind: string;
}

interface Update2 {
  app_state: string[];
  delegate: string;
  verification_key: string;
  permissions: string;
  zkapp_uri: string;
  token_symbol: string;
  timing: string;
  voting_for: string;
}

interface BalanceChange2 {
  magnitude: string;
  sgn: string[];
}

interface Preconditions2 {
  network: Network2;
  account: Account2;
  valid_while: string;
}

interface Network2 {
  snarked_ledger_hash: string;
  blockchain_length: string;
  min_window_density: string;
  total_currency: string;
  global_slot_since_genesis: string;
  staking_epoch_data: StakingEpochData2;
  next_epoch_data: NextEpochData2;
}

interface StakingEpochData2 {
  ledger: Ledger3;
  seed: string;
  start_checkpoint: string;
  lock_checkpoint: string;
  epoch_length: string;
}

interface Ledger3 {
  hash: string;
  total_currency: string;
}

interface NextEpochData2 {
  ledger: Ledger4;
  seed: string;
  start_checkpoint: string;
  lock_checkpoint: string;
  epoch_length: string;
}

interface Ledger4 {
  hash: string;
  total_currency: string;
}

interface Account2 {
  balance: string;
  nonce: string;
  receipt_chain_hash: string;
  delegate: string;
  state: string[];
  action_state: string;
  proved_state: string;
  is_new: string;
}
